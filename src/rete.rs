//! Ricerca dei telefoni col Debug wireless attivo, via mDNS.
//!
//! Si chiede il servizio `_adb-tls-connect._tcp.local` con il bit QU (risposta
//! diretta al nostro socket): è il metodo che ha trovato il telefono quando
//! `adb mdns` non vedeva niente. L'istanza si chiama `adb-<seriale>-<suffisso>`
//! e la porta cambia a ogni attivazione del debug, quindi va sempre riletta.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::{Duration, Instant};

use anyhow::Result;

const SERVIZIO: &str = "_adb-tls-connect._tcp.local";
/// Annunciato solo mentre sul telefono è aperta la schermata «Associa
/// dispositivo con codice di associazione» (prove §37).
const SERVIZIO_ABBINAMENTO: &str = "_adb-tls-pairing._tcp.local";
const GRUPPO_MDNS: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(224, 0, 0, 251), 5353);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelefonoInRete {
    pub seriale: String,
    pub indirizzo: SocketAddr,
    pub istanza: String,
}

fn codifica_nome(nome: &str) -> Vec<u8> {
    let mut v = Vec::new();
    for parte in nome.split('.').filter(|p| !p.is_empty()) {
        v.push(parte.len() as u8);
        v.extend_from_slice(parte.as_bytes());
    }
    v.push(0);
    v
}

/// Legge un nome DNS (con i puntatori di compressione); restituisce il nome e
/// la posizione subito dopo il nome nel messaggio.
fn leggi_nome(d: &[u8], mut o: usize) -> Option<(String, usize)> {
    let mut parti = Vec::new();
    let mut dopo = None;
    for _ in 0..128 {
        let l = *d.get(o)? as usize;
        if l == 0 {
            o += 1;
            break;
        }
        if l & 0xC0 == 0xC0 {
            dopo.get_or_insert(o + 2);
            o = ((l & 0x3F) << 8) | *d.get(o + 1)? as usize;
            continue;
        }
        parti.push(String::from_utf8_lossy(d.get(o + 1..o + 1 + l)?).to_string());
        o += 1 + l;
    }
    Some((parti.join("."), dopo.unwrap_or(o)))
}

fn u16_be(d: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*d.get(o)?, *d.get(o + 1)?]))
}

/// Estrae dalle risposte i record SRV (istanza → porta, destinazione) e A.
fn analizza(d: &[u8], srv: &mut HashMap<String, (u16, String)>, a: &mut HashMap<String, Ipv4Addr>) -> Option<()> {
    let domande = u16_be(d, 4)?;
    let record = u16_be(d, 6)? as usize + u16_be(d, 8)? as usize + u16_be(d, 10)? as usize;
    let mut o = 12;
    for _ in 0..domande {
        o = leggi_nome(d, o)?.1 + 4;
    }
    for _ in 0..record {
        let (nome, dopo) = leggi_nome(d, o)?;
        let tipo = u16_be(d, dopo)?;
        let durata = u32::from_be_bytes(d.get(dopo + 4..dopo + 8)?.try_into().ok()?);
        let lunghezza = u16_be(d, dopo + 8)? as usize;
        let dati = dopo + 10;
        match tipo {
            // Durata 0 = «addio»: il servizio su quella porta si è chiuso
            // (succede quando il debug si riavvia, es. staccando il cavo).
            33 if durata == 0 => {
                srv.remove(&nome);
            }
            33 => {
                let porta = u16_be(d, dati + 4)?;
                let destinazione = leggi_nome(d, dati + 6)?.0;
                srv.insert(nome, (porta, destinazione));
            }
            1 if lunghezza == 4 => {
                let b = d.get(dati..dati + 4)?;
                a.insert(nome, Ipv4Addr::new(b[0], b[1], b[2], b[3]));
            }
            _ => {}
        }
        o = dati + lunghezza;
    }
    Some(())
}

/// `adb-R5CT0000000-aBcDeF._adb-tls-connect._tcp.local` → `R5CT0000000`.
fn seriale_da_istanza(istanza: &str) -> Option<String> {
    let etichetta = istanza.split('.').next()?;
    Some(etichetta.strip_prefix("adb-")?.rsplit_once('-')?.0.to_string())
}

/// Cerca i telefoni per `durata` e li restituisce, uno per numero di serie.
pub fn cerca(durata: Duration) -> Result<Vec<TelefonoInRete>> {
    cerca_servizio(SERVIZIO, durata)
}

/// Cerca i telefoni con la schermata del codice di associazione aperta:
/// l'indirizzo è quello a cui mandare il codice (`adb::abbina`).
pub fn cerca_abbinamento(durata: Duration) -> Result<Vec<TelefonoInRete>> {
    cerca_servizio(SERVIZIO_ABBINAMENTO, durata)
}

fn cerca_servizio(servizio: &str, durata: Duration) -> Result<Vec<TelefonoInRete>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(300)))?;
    let mut domanda = vec![0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    domanda.extend(codifica_nome(servizio));
    domanda.extend([0, 12, 0x80, 0x01]); // tipo PTR, classe IN con bit QU
    socket.send_to(&domanda, GRUPPO_MDNS)?;

    let mut srv = HashMap::new();
    let mut a = HashMap::new();
    let mut buffer = [0u8; 9000];
    let fine = Instant::now() + durata;
    let mut ultimo_invio = Instant::now();
    while Instant::now() < fine {
        // Qualche telefono risponde solo alla seconda domanda.
        if ultimo_invio.elapsed() > Duration::from_secs(1) {
            socket.send_to(&domanda, GRUPPO_MDNS)?;
            ultimo_invio = Instant::now();
        }
        if let Ok((n, _)) = socket.recv_from(&mut buffer) {
            let _ = analizza(&buffer[..n], &mut srv, &mut a);
        }
    }

    let mut trovati: Vec<TelefonoInRete> = srv
        .into_iter()
        .filter_map(|(istanza, (porta, destinazione))| {
            let ip = *a.get(&destinazione)?;
            let seriale = seriale_da_istanza(&istanza)?;
            Some(TelefonoInRete { seriale, indirizzo: SocketAddr::from((ip, porta)), istanza })
        })
        .collect();
    trovati.sort_by(|x, y| x.seriale.cmp(&y.seriale));
    trovati.dedup_by(|x, y| x.seriale == y.seriale);
    Ok(trovati)
}

/// Indirizzo del telefono `seriale` che accetta davvero collegamenti: la porta
/// annunciata può essere già vecchia (debug riavviato), quindi si prova a
/// collegarsi e, se rifiuta, si ripete la ricerca.
pub fn indirizzo_attivo(seriale: &str, ultimo: Option<SocketAddr>) -> Result<SocketAddr> {
    // Via rapida: l'ultimo indirizzo buono (la porta cambia solo quando il
    // debug si riavvia). Se risponde non serve la ricerca.
    if let Some(a) = ultimo
        && std::net::TcpStream::connect_timeout(&a, Duration::from_millis(800)).is_ok()
    {
        return Ok(a);
    }
    for _ in 0..3 {
        if let Some(t) = cerca(Duration::from_secs(3))?.into_iter().find(|t| t.seriale == seriale)
            && std::net::TcpStream::connect_timeout(&t.indirizzo, Duration::from_secs(2)).is_ok()
        {
            return Ok(t.indirizzo);
        }
    }
    anyhow::bail!("telefono non raggiungibile: acceso, sbloccato, stessa rete Wi-Fi, Debug wireless attivo?")
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn nome_codificato_e_riletto() {
        let codificato = codifica_nome(SERVIZIO);
        assert_eq!(leggi_nome(&codificato, 0).unwrap(), (SERVIZIO.trim_end_matches('.').to_string(), codificato.len()));
    }

    #[test]
    fn seriale_dal_nome_del_servizio() {
        // Nome reale annunciato dal Galaxy S23+ il 26 set 2026.
        let istanza = "adb-R5CT0000000-aBcDeF._adb-tls-connect._tcp.local";
        assert_eq!(seriale_da_istanza(istanza).as_deref(), Some("R5CT0000000"));
        assert_eq!(seriale_da_istanza("stampante._ipp._tcp.local"), None);
    }
}
