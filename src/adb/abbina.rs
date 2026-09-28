//! Associazione col codice del Debug wireless (Android 11+, «Associa
//! dispositivo con codice di associazione»): lo stesso protocollo di
//! `adb pair`, riprodotto dai sorgenti di Android (`packages/modules/adb`:
//! `pairing_connection`, `pairing_auth`, `aes_128_gcm`) e di BoringSSL
//! (`crypto/curve25519/spake25519`).
//!
//! 1. TLS direttamente sulla porta di associazione, col certificato di Phonestra;
//! 2. password = le 6 cifre + 64 byte esportati dal TLS (etichetta
//!    «adb-label\0»): nessuno può mettersi in mezzo alla connessione;
//! 3. SPAKE2 su Ed25519, noi nel ruolo «alice»;
//! 4. dalla chiave comune, con HKDF-SHA256, una chiave AES-128-GCM;
//! 5. ci si scambiano cifrati i PeerInfo: noi la chiave pubblica ADB di
//!    Phonestra, il telefono il suo identificativo.
//!
//! Da quel momento il telefono accetta la chiave di Phonestra nei collegamenti
//! Wi-Fi (`Adb::wifi`), come dopo un «Consenti sempre» col cavo.

use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use base64::Engine;
use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use ring::{aead, digest, hkdf};
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use super::tls;

/// Nomi dei due ruoli, col terminatore: Android usa `sizeof` sulle stringhe C.
const NOME_CLIENT: &[u8] = b"adb pair client\0";
const NOME_SERVER: &[u8] = b"adb pair server\0";
const ETICHETTA_TLS: &[u8] = b"adb-label\0";
const INFO_HKDF: &[u8] = b"adb pairing_auth aes-128-gcm key";

/// Punti M e N di BoringSSL (`spake25519.cc`), codificati.
const PUNTO_M: &str = "5ada7e4bf6ddd9adb6626d32131c6b5c51a1e347a3478f53cfcf441b88eed12e";
const PUNTO_N: &str = "10e3df0ae37d8e7a99b5fe74b44672103dbddcbd06af680d71329a11693bc778";

const VERSIONE_PACCHETTO: u8 = 1;
const TIPO_SPAKE2: u8 = 0;
const TIPO_PEER_INFO: u8 = 1;
/// `PeerInfo`: un byte di tipo e 8191 di dati, sempre 8192 in tutto.
const DIMENSIONE_PEER_INFO: usize = 8192;
const PEER_INFO_CHIAVE_RSA: u8 = 0;
const PEER_INFO_GUID: u8 = 1;

fn punto(esadecimale: &str) -> EdwardsPoint {
    let mut byte = [0u8; 32];
    for (i, b) in byte.iter_mut().enumerate() {
        *b = u8::from_str_radix(&esadecimale[2 * i..2 * i + 2], 16).expect("costante esadecimale");
    }
    CompressedEdwardsY(byte).decompress().expect("punto della curva")
}

/// La parte di ordine primo di un punto. BoringSSL moltiplica M e N per la
/// password ridotta più multipli dell'ordine, in modo che il risultato sia
/// multiplo di 8: equivale a togliere la parte di ordine piccolo.
fn parte_prima(p: EdwardsPoint) -> EdwardsPoint {
    p.mul_by_cofactor() * Scalar::from(8u64).invert()
}

fn sha512(parti: &[&[u8]]) -> [u8; 64] {
    let mut c = digest::Context::new(&digest::SHA512);
    for p in parti {
        c.update(p);
    }
    c.finish().as_ref().try_into().expect("SHA-512 da 64 byte")
}

/// SPAKE2 come `SPAKE2_generate_msg` / `SPAKE2_process_msg` di BoringSSL.
struct Spake2 {
    alice: bool,
    /// `x`: la chiave privata di BoringSSL è 8·x (multipla del cofattore).
    privata: Scalar,
    /// Password ridotta modulo l.
    password: Scalar,
    password_hash: [u8; 64],
    messaggio: [u8; 32],
}

impl Spake2 {
    fn new(alice: bool, password: &[u8], casuale: [u8; 64]) -> Self {
        let privata = Scalar::from_bytes_mod_order_wide(&casuale);
        let p = (ED25519_BASEPOINT_POINT * privata).mul_by_cofactor();
        let password_hash = sha512(&[password]);
        let password = Scalar::from_bytes_mod_order_wide(&password_hash);
        let maschera = parte_prima(punto(if alice { PUNTO_M } else { PUNTO_N })) * password;
        let messaggio = (p + maschera).compress().to_bytes();
        Self { alice, privata, password, password_hash, messaggio }
    }

    fn chiave(&self, loro: &[u8]) -> Result<[u8; 64]> {
        let loro: [u8; 32] = loro.try_into().map_err(|_| anyhow!("messaggio SPAKE2 di {} byte invece di 32", loro.len()))?;
        let q = CompressedEdwardsY(loro).decompress().context("il messaggio SPAKE2 del telefono non è un punto della curva")?;
        let maschera = parte_prima(punto(if self.alice { PUNTO_N } else { PUNTO_M })) * self.password;
        let comune = ((q - maschera) * self.privata).mul_by_cofactor().compress().to_bytes();
        // Ogni parte preceduta dalla lunghezza (8 byte little-endian).
        let lunghezza = |b: &[u8]| (b.len() as u64).to_le_bytes();
        let (nome_a, nome_b, msg_a, msg_b) = if self.alice {
            (NOME_CLIENT, NOME_SERVER, &self.messaggio[..], &loro[..])
        } else {
            (NOME_CLIENT, NOME_SERVER, &loro[..], &self.messaggio[..])
        };
        Ok(sha512(&[
            &lunghezza(nome_a), nome_a,
            &lunghezza(nome_b), nome_b,
            &lunghezza(msg_a), msg_a,
            &lunghezza(msg_b), msg_b,
            &lunghezza(&comune), &comune,
            &lunghezza(&self.password_hash), &self.password_hash,
        ]))
    }
}

/// AES-128-GCM con contatori separati per i due versi (nonce: contatore a
/// 64 bit little-endian nei primi 8 byte, zeri negli altri 4).
struct Cifrario {
    chiave: aead::LessSafeKey,
    inviati: u64,
    ricevuti: u64,
}

impl Cifrario {
    fn new(chiave_spake2: &[u8; 64]) -> Result<Self> {
        let prk = hkdf::Salt::new(hkdf::HKDF_SHA256, &[]).extract(chiave_spake2);
        let materiale = prk.expand(&[INFO_HKDF], &aead::AES_128_GCM).map_err(|_| anyhow!("HKDF non riuscito"))?;
        Ok(Self { chiave: aead::LessSafeKey::new(aead::UnboundKey::from(materiale)), inviati: 0, ricevuti: 0 })
    }

    fn nonce(contatore: u64) -> aead::Nonce {
        let mut n = [0u8; 12];
        n[..8].copy_from_slice(&contatore.to_le_bytes());
        aead::Nonce::assume_unique_for_key(n)
    }

    fn cifra(&mut self, mut dati: Vec<u8>) -> Result<Vec<u8>> {
        self.chiave
            .seal_in_place_append_tag(Self::nonce(self.inviati), aead::Aad::empty(), &mut dati)
            .map_err(|_| anyhow!("cifratura non riuscita"))?;
        self.inviati += 1;
        Ok(dati)
    }

    fn decifra(&mut self, mut dati: Vec<u8>) -> Result<Vec<u8>> {
        let chiari = self
            .chiave
            .open_in_place(Self::nonce(self.ricevuti), aead::Aad::empty(), &mut dati)
            .map_err(|_| anyhow!("risposta del telefono non decifrabile"))?
            .len();
        self.ricevuti += 1;
        dati.truncate(chiari);
        Ok(dati)
    }
}

/// La chiave pubblica nel formato di Android (`android_pubkey_encode`):
/// base64 della struttura RSAPublicKey (modulo e R² in parole da 32 bit
/// little-endian, lunghezza fissa) seguita da « nome@computer».
pub fn chiave_pubblica(chiave: &Path) -> Result<String> {
    let pem = std::fs::read_to_string(chiave).with_context(|| format!("chiave {} illeggibile", chiave.display()))?;
    let privata = rsa::RsaPrivateKey::from_pkcs8_pem(&pem).context("chiave ADB non valida")?;
    let n = privata.n();
    let parole = n.bits().div_ceil(32);
    let a_lunghezza = |mut b: Vec<u8>| {
        b.resize(parole * 4, 0);
        b
    };
    let n0 = u32::from_le_bytes(a_lunghezza(n.to_bytes_le())[..4].try_into()?);
    // Inverso di n0 modulo 2³² (Newton: ogni passo raddoppia i bit giusti).
    let mut inverso = n0;
    for _ in 0..5 {
        inverso = inverso.wrapping_mul(2u32.wrapping_sub(n0.wrapping_mul(inverso)));
    }
    let rr = (rsa::BigUint::from(1u8) << (parole * 64)) % n;
    let mut e = privata.e().to_bytes_le();
    e.resize(4, 0);
    let esponente = u32::from_le_bytes(e[..4].try_into()?);
    let mut struttura = Vec::with_capacity(12 + parole * 8);
    struttura.extend((parole as u32).to_le_bytes());
    struttura.extend(inverso.wrapping_neg().to_le_bytes());
    struttura.extend(a_lunghezza(n.to_bytes_le()));
    struttura.extend(a_lunghezza(rr.to_bytes_le()));
    struttura.extend(esponente.to_le_bytes());
    let computer = std::fs::read_to_string("/proc/sys/kernel/hostname").unwrap_or_default();
    Ok(format!("{} Phonestra@{}", base64::engine::general_purpose::STANDARD.encode(struttura), computer.trim()))
}

async fn scrivi_pacchetto<T: AsyncWrite + Unpin>(flusso: &mut T, tipo: u8, dati: &[u8]) -> Result<()> {
    let mut pacchetto = vec![VERSIONE_PACCHETTO, tipo];
    pacchetto.extend((dati.len() as u32).to_be_bytes());
    pacchetto.extend_from_slice(dati);
    flusso.write_all(&pacchetto).await?;
    flusso.flush().await?;
    Ok(())
}

async fn leggi_pacchetto<T: AsyncRead + Unpin>(flusso: &mut T, tipo_atteso: u8) -> Result<Vec<u8>> {
    let mut testa = [0u8; 6];
    flusso.read_exact(&mut testa).await.context("il telefono ha chiuso l'associazione (codice sbagliato o scaduto?)")?;
    if testa[0] != VERSIONE_PACCHETTO {
        bail!("versione del protocollo di associazione {} non supportata", testa[0]);
    }
    if testa[1] != tipo_atteso {
        bail!("pacchetto di tipo {} invece di {tipo_atteso}", testa[1]);
    }
    let lunghezza = u32::from_be_bytes(testa[2..].try_into()?) as usize;
    if lunghezza == 0 || lunghezza > 2 * DIMENSIONE_PEER_INFO {
        bail!("pacchetto di associazione di {lunghezza} byte");
    }
    let mut dati = vec![0u8; lunghezza];
    flusso.read_exact(&mut dati).await?;
    Ok(dati)
}

/// Associa Phonestra al telefono che mostra `codice` all'indirizzo della
/// schermata di associazione (`rete::cerca_abbinamento`). Restituisce
/// l'identificativo che il telefono dà di sé (es. `adb-R5CT0000000-aBcDeF`).
pub async fn abbina(indirizzo: SocketAddr, codice: &str, chiave: &Path) -> Result<String> {
    let config = tls::configurazione_client(chiave)?;
    let tcp = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(indirizzo))
        .await
        .context("tempo scaduto")?
        .with_context(|| format!("collegamento a {indirizzo} non riuscito"))?;
    tcp.set_nodelay(true)?;
    let nome = rustls::pki_types::ServerName::IpAddress(indirizzo.ip().into());
    let mut flusso = TlsConnector::from(config).connect(nome, tcp).await.context("handshake TLS dell'associazione non riuscito")?;

    let esportati = flusso.get_ref().1.export_keying_material([0u8; 64], ETICHETTA_TLS, None)?;
    let mut password = codice.trim().as_bytes().to_vec();
    password.extend_from_slice(&esportati);
    let mut casuale = [0u8; 64];
    ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(), &mut casuale).map_err(|_| anyhow!("generatore casuale non disponibile"))?;
    let spake2 = Spake2::new(true, &password, casuale);

    scrivi_pacchetto(&mut flusso, TIPO_SPAKE2, &spake2.messaggio).await?;
    let loro = leggi_pacchetto(&mut flusso, TIPO_SPAKE2).await?;
    let mut cifrario = Cifrario::new(&spake2.chiave(&loro)?)?;

    let pubblica = chiave_pubblica(chiave)?;
    if pubblica.len() >= DIMENSIONE_PEER_INFO - 1 {
        bail!("chiave pubblica troppo lunga ({} byte)", pubblica.len());
    }
    let mut nostro = vec![0u8; DIMENSIONE_PEER_INFO];
    nostro[0] = PEER_INFO_CHIAVE_RSA;
    nostro[1..1 + pubblica.len()].copy_from_slice(pubblica.as_bytes());
    scrivi_pacchetto(&mut flusso, TIPO_PEER_INFO, &cifrario.cifra(nostro)?).await?;

    // Se il codice è sbagliato il telefono non riesce a decifrare e chiude.
    let loro = cifrario.decifra(leggi_pacchetto(&mut flusso, TIPO_PEER_INFO).await?)?;
    if loro.len() != DIMENSIONE_PEER_INFO || loro[0] != PEER_INFO_GUID {
        bail!("risposta del telefono inattesa ({} byte, tipo {})", loro.len(), loro.first().copied().unwrap_or(255));
    }
    let fine = loro[1..].iter().position(|&b| b == 0).map_or(loro.len(), |p| p + 1);
    Ok(String::from_utf8_lossy(&loro[1..fine]).to_string())
}

#[cfg(test)]
mod prove {
    use super::*;

    /// Client e server con la stessa password arrivano alla stessa chiave;
    /// con password diverse no.
    #[test]
    fn spake2_stessa_chiave() {
        let alice = Spake2::new(true, b"229325 e il resto", [7u8; 64]);
        let bob = Spake2::new(false, b"229325 e il resto", [9u8; 64]);
        assert_eq!(alice.chiave(&bob.messaggio).unwrap(), bob.chiave(&alice.messaggio).unwrap());
        let intruso = Spake2::new(false, b"000000 e il resto", [9u8; 64]);
        assert_ne!(alice.chiave(&intruso.messaggio).unwrap(), intruso.chiave(&alice.messaggio).unwrap());
    }

    /// I messaggi sono nel sottogruppo di ordine primo, come quelli di
    /// BoringSSL dopo la correzione della password.
    #[test]
    fn spake2_messaggio_senza_torsione() {
        let alice = Spake2::new(true, b"123456", [1u8; 64]);
        assert!(CompressedEdwardsY(alice.messaggio).decompress().unwrap().is_torsion_free());
        assert!(parte_prima(punto(PUNTO_M)).is_torsion_free());
    }

    /// La struttura di `android_pubkey_encode`: 524 byte per una chiave da
    /// 2048 bit, n0inv tale che n0·n0inv ≡ −1 (mod 2³²), R² = 2⁴⁰⁹⁶ mod n.
    #[test]
    fn chiave_pubblica_formato_android() {
        use rsa::pkcs8::{EncodePrivateKey, LineEnding};
        let privata = rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap();
        let file = std::env::temp_dir().join(format!("phonestra-prova-chiave-{}", std::process::id()));
        std::fs::write(&file, privata.to_pkcs8_pem(LineEnding::LF).unwrap().as_bytes()).unwrap();
        let testo = chiave_pubblica(&file).unwrap();
        std::fs::remove_file(&file).unwrap();
        let b = base64::engine::general_purpose::STANDARD.decode(testo.split(' ').next().unwrap()).unwrap();
        assert_eq!(b.len(), 524);
        let parola = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        assert_eq!(parola(0), 64);
        assert_eq!(rsa::BigUint::from_bytes_le(&b[8..264]), *privata.n());
        assert_eq!(parola(8).wrapping_mul(parola(4)), u32::MAX);
        assert_eq!(rsa::BigUint::from_bytes_le(&b[264..520]), (rsa::BigUint::from(1u8) << 4096usize) % privata.n());
        assert_eq!(parola(520), 65537);
    }

    #[test]
    fn cifrario_andata_e_ritorno() {
        let mut a = Cifrario::new(&[3u8; 64]).unwrap();
        let mut b = Cifrario::new(&[3u8; 64]).unwrap();
        let cifrato = a.cifra(b"ciao".to_vec()).unwrap();
        assert_eq!(cifrato.len(), 4 + 16);
        assert_eq!(b.decifra(cifrato).unwrap(), b"ciao");
    }
}
