//! Una sessione con il componente sul telefono (per ora il server ufficiale di
//! scrcpy 4.1, Apache 2.0, in `telefono/`): display virtuale, video e comandi.
//!
//! Protocollo (scrcpy 4.1, `doc/develop.md` e sorgenti del server):
//! - avvio: `CLASSPATH=<jar> app_process / com.genymobile.scrcpy.Server 4.1 chiave=valore…`;
//! - con `tunnel_forward=true` il server ascolta su `localabstract:scrcpy_<scid esadecimale>`
//!   e noi apriamo i canali in ordine: video, poi comandi;
//! - sul primo canale: 1 byte fittizio, poi 64 byte col nome del dispositivo;
//! - sul canale video: id del codec (u32 BE), poi pacchetti con intestazione di 12 byte:
//!   sessione (bit 63) = larghezza e altezza; dati = pts/flag (u64) + lunghezza (u32).

use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::adb::{Adb, Canale, sync};

/// Il componente sul telefono, incluso nell'eseguibile.
const SERVER: &[u8] = include_bytes!("../telefono/scrcpy-server-v4.1");
const VERSIONE_SERVER: &str = "4.1";

const FLAG_SESSIONE: u64 = 1 << 63;
const FLAG_CONFIG: u64 = 1 << 62;
const FLAG_CHIAVE: u64 = 1 << 61;

/// Codec video annunciati dal server.
pub fn nome_codec(id: u32) -> &'static str {
    match id {
        0x6832_3634 => "h264",
        0x6832_3635 => "h265",
        0x0061_7631 => "av1",
        _ => "sconosciuto",
    }
}

pub struct Opzioni {
    /// Dimensione del display virtuale: larghezza, altezza, densità.
    pub display: (u32, u32, u32),
    /// Display ridimensionabile con [`Comandi::ridimensiona`] (`flex_display`).
    pub ridimensionabile: bool,
    pub codec: &'static str,
    /// Schermo principale del telefono invece di un display virtuale.
    pub specchio: bool,
}

impl Default for Opzioni {
    fn default() -> Self {
        Self { display: (720, 1280, 320), ridimensionabile: true, codec: "h264", specchio: false }
    }
}

pub struct Sessione {
    pub nome_dispositivo: String,
    pub codec: u32,
    /// Flusso video: va letto da un solo compito con [`leggi_pacchetto`].
    pub video: Canale,
    pub comandi: Comandi,
    /// Il canale su cui gira il server: ne escono i suoi messaggi; chiuderlo lo ferma.
    pub server: Canale,
}

/// Canale dei comandi verso il componente (tocchi, tasti, avvio app…).
pub struct Comandi(Canale);

/// Un pacchetto del flusso video.
pub enum Pacchetto {
    /// Nuova sessione di cattura (inizio o cambio di dimensione).
    Dimensione { larghezza: u32, altezza: u32 },
    /// Dati del codec: parametri (`config`) o un fotogramma.
    Dati { pts: u64, config: bool, chiave: bool, dati: Vec<u8> },
}

/// Messaggi di diagnosi, solo con `PHONESTRA_DEBUG=1`.
pub(crate) fn diagnosi(testo: &str) {
    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
        eprintln!("[sessione] {testo}");
    }
}

pub(crate) fn u32_be(b: &[u8]) -> u32 {
    u32::from_be_bytes(b[0..4].try_into().unwrap())
}

/// Copia il componente sul telefono e lo avvia con i `parametri` scrcpy
/// (oltre a `scid`, `log_level` e `tunnel_forward`). Restituisce l'id della
/// sessione e il canale su cui gira (ne escono i suoi messaggi; chiuderlo lo
/// ferma). Ogni avvio usa un file suo: il componente lo cancella appena
/// partito, e due avvii contemporanei sullo stesso file si pesterebbero.
pub(crate) async fn lancia(adb: &Adb, parametri: &str) -> Result<(u32, Canale)> {
    let scid: u32 = rand::random::<u32>() & 0x7fff_ffff;
    let percorso = format!("/data/local/tmp/phonestra-server-{scid:08x}.jar");
    diagnosi("copia del componente…");
    sync::invia(adb, SERVER, &percorso, 0o644).await.context("copia del componente sul telefono")?;
    let comando = format!(
        "CLASSPATH={percorso} app_process / com.genymobile.scrcpy.Server {VERSIONE_SERVER} \
         scid={scid:08x} log_level={} tunnel_forward=true {parametri}",
        if std::env::var_os("PHONESTRA_DEBUG").is_some() { "verbose" } else { "info" }
    );
    diagnosi(&format!("avvio: {comando}"));
    let server = adb.apri(&format!("shell:{comando}")).await?;
    Ok((scid, server))
}

/// Apre il prossimo canale del componente `scid`. Il primo tentativo può
/// arrivare prima che il componente ascolti: si riprova per qualche secondo.
pub(crate) async fn apri_canale(adb: &Adb, scid: u32) -> Result<Canale> {
    let nome_socket = format!("localabstract:scrcpy_{scid:08x}");
    for tentativo in 0..50 {
        match adb.apri(&nome_socket).await {
            Ok(c) => return Ok(c),
            Err(e) => diagnosi(&format!("tentativo {tentativo}: {e:#}")),
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    bail!("il componente sul telefono non è partito")
}

impl Sessione {
    pub async fn avvia(adb: &Adb, opzioni: &Opzioni) -> Result<Self> {
        let (l, a, d) = opzioni.display;
        let parametri = if opzioni.specchio {
            format!("audio=false control=true video_codec={} max_size=1920 clipboard_autosync=false", opzioni.codec)
        } else {
            format!(
                "audio=false control=true video_codec={} new_display={l}x{a}/{d} vd_system_decorations=false \
                 flex_display={} clipboard_autosync=false",
                opzioni.codec, opzioni.ridimensionabile,
            )
        };
        let (scid, server) = lancia(adb, &parametri).await?;
        let mut video = apri_canale(adb, scid).await?;
        // Il nome del dispositivo arriva solo quando tutti i canali sono aperti.
        let comandi = apri_canale(adb, scid).await.context("canale dei comandi")?;
        let intestazione = tokio::time::timeout(Duration::from_secs(10), video.leggi_esatti(1 + 64 + 4))
            .await
            .context("il componente non ha mandato l'intestazione video")??;
        diagnosi("intestazione video ricevuta");
        let nome = String::from_utf8_lossy(&intestazione[1..65]).trim_end_matches('\0').to_string();
        let codec = u32_be(&intestazione[65..69]);
        if codec <= 1 {
            bail!("il telefono non ha potuto avviare il video (codice {codec})");
        }
        Ok(Self { nome_dispositivo: nome, codec, video, comandi: Comandi(comandi), server })
    }
}

/// Numero del display virtuale dal messaggio del componente
/// «New display: 400x714/160 (id=57)».
pub fn display_da_messaggio(riga: &str) -> Option<i32> {
    let dopo = riga.split_once("New display:")?.1;
    dopo.split_once("(id=")?.1.split(')').next()?.trim().parse().ok()
}

/// Toglie dalle recenti le app del display virtuale `display`, come
/// scorrerle via dal telefono: si chiudono senza arresto forzato (che
/// bloccherebbe le loro notifiche). Senza, alla chiusura del display Samsung
/// le sposta sullo schermo del telefono e continuano lì (SPECIFICHE §7.4).
pub async fn togli_dalle_recenti(adb: &Adb, display: i32) -> Result<()> {
    let elenco = adb.esegui("am stack list").await?;
    for riga in elenco.lines().filter(|r| r.starts_with("RootTask id=")) {
        let campo = |nome: &str| riga.split_whitespace().find_map(|p| p.strip_prefix(nome)).and_then(|v| v.parse::<i32>().ok());
        if campo("displayId=") == Some(display)
            && let Some(id) = campo("id=")
        {
            diagnosi(&format!("tolgo il task {id} del display {display}"));
            adb.esegui(&format!("am stack remove {id}")).await?;
        }
    }
    Ok(())
}

/// Legge il prossimo pacchetto video. Non va interrotta a metà (es. dentro
/// `select!`): i byte già letti andrebbero persi. Usarla in un compito dedicato.
pub async fn leggi_pacchetto(video: &mut Canale) -> Result<Pacchetto> {
    let h = video.leggi_esatti(12).await?;
    let testa = u64::from_be_bytes(h[0..8].try_into().unwrap());
    if testa & FLAG_SESSIONE != 0 {
        return Ok(Pacchetto::Dimensione { larghezza: u32_be(&h[4..8]), altezza: u32_be(&h[8..12]) });
    }
    let lunghezza = u32_be(&h[8..12]) as usize;
    let dati = video.leggi_esatti(lunghezza).await?;
    Ok(Pacchetto::Dati {
        pts: testa & !(FLAG_CONFIG | FLAG_CHIAVE),
        config: testa & FLAG_CONFIG != 0,
        chiave: testa & FLAG_CHIAVE != 0,
        dati,
    })
}

impl Comandi {
    /// Avvia un'app sul display virtuale (messaggio START_APP).
    pub async fn avvia_app(&mut self, pacchetto: &str) -> Result<()> {
        let nome = pacchetto.as_bytes();
        if nome.len() > 255 {
            bail!("nome del pacchetto troppo lungo");
        }
        let mut m = vec![16u8, nome.len() as u8];
        m.extend_from_slice(nome);
        self.0.scrivi(&m).await
    }

    /// Accende o spegne il pannello fisico del telefono (SET_DISPLAY_POWER).
    pub async fn pannello(&mut self, acceso: bool) -> Result<()> {
        self.0.scrivi(&[10u8, acceso as u8]).await
    }

    /// Tocco o clic (INJECT_TOUCH_EVENT). `azione`: 0 giù, 1 su, 2 movimento.
    pub async fn tocco(&mut self, azione: u8, x: i32, y: i32, larghezza: u16, altezza: u16) -> Result<()> {
        let mut m = vec![2u8, azione];
        m.extend_from_slice(&(-1i64 as u64).to_be_bytes()); // puntatore del mouse
        m.extend_from_slice(&x.to_be_bytes());
        m.extend_from_slice(&y.to_be_bytes());
        m.extend_from_slice(&larghezza.to_be_bytes());
        m.extend_from_slice(&altezza.to_be_bytes());
        let pressione: u16 = if azione == 1 { 0 } else { 0xffff };
        m.extend_from_slice(&pressione.to_be_bytes());
        m.extend_from_slice(&1u32.to_be_bytes()); // pulsante principale
        m.extend_from_slice(&(if azione == 1 { 0u32 } else { 1 }).to_be_bytes());
        self.0.scrivi(&m).await
    }

    /// Tocco di un dito (non del mouse): serve per la pressione lunga, che
    /// Android col mouse non considera (selezione del testo, menu).
    pub async fn dito(&mut self, azione: u8, x: i32, y: i32, larghezza: u16, altezza: u16) -> Result<()> {
        self.dito_numero(-2, azione, x, y, larghezza, altezza).await // dito generico
    }

    /// Tocco del dito `numero` (per i gesti a più dita, come il pizzico).
    pub async fn dito_numero(&mut self, numero: i64, azione: u8, x: i32, y: i32, larghezza: u16, altezza: u16) -> Result<()> {
        let mut m = vec![2u8, azione];
        m.extend_from_slice(&(numero as u64).to_be_bytes());
        m.extend_from_slice(&x.to_be_bytes());
        m.extend_from_slice(&y.to_be_bytes());
        m.extend_from_slice(&larghezza.to_be_bytes());
        m.extend_from_slice(&altezza.to_be_bytes());
        let pressione: u16 = if azione == 1 { 0 } else { 0xffff };
        m.extend_from_slice(&pressione.to_be_bytes());
        m.extend_from_slice(&0u32.to_be_bytes()); // nessun pulsante
        m.extend_from_slice(&0u32.to_be_bytes());
        self.0.scrivi(&m).await
    }

    /// Più tocchi di dita (`(numero, azione, x, y)`) in un solo invio: una sola
    /// attesa della conferma del telefono invece di una per messaggio.
    pub async fn dita_insieme(&mut self, tocchi: &[(i64, u8, i32, i32)], larghezza: u16, altezza: u16) -> Result<()> {
        let mut m = Vec::with_capacity(32 * tocchi.len());
        for &(numero, azione, x, y) in tocchi {
            m.extend_from_slice(&[2u8, azione]);
            m.extend_from_slice(&(numero as u64).to_be_bytes());
            m.extend_from_slice(&x.to_be_bytes());
            m.extend_from_slice(&y.to_be_bytes());
            m.extend_from_slice(&larghezza.to_be_bytes());
            m.extend_from_slice(&altezza.to_be_bytes());
            let pressione: u16 = if azione == 1 { 0 } else { 0xffff };
            m.extend_from_slice(&pressione.to_be_bytes());
            m.extend_from_slice(&0u32.to_be_bytes());
            m.extend_from_slice(&0u32.to_be_bytes());
        }
        self.0.scrivi(&m).await
    }

    /// Nuova dimensione del display virtuale (RESIZE_DISPLAY, solo con
    /// `ridimensionabile`): la densità resta quella iniziale. Il telefono la
    /// applica dopo un attimo e manda un pacchetto [`Pacchetto::Dimensione`].
    pub async fn ridimensiona(&mut self, larghezza: u16, altezza: u16) -> Result<()> {
        let mut m = vec![21u8];
        m.extend_from_slice(&larghezza.to_be_bytes());
        m.extend_from_slice(&altezza.to_be_bytes());
        self.0.scrivi(&m).await
    }

    /// Testo già composto dal PC (INJECT_TEXT): Android lo accetta solo per i
    /// caratteri della sua mappa virtuale (in pratica ASCII); per gli altri
    /// usare [`Comandi::incolla`].
    pub async fn testo(&mut self, testo: &str) -> Result<()> {
        let mut m = vec![1u8];
        m.extend_from_slice(&(testo.len() as u32).to_be_bytes());
        m.extend_from_slice(testo.as_bytes());
        self.0.scrivi(&m).await
    }

    /// Tasto Android (INJECT_KEYCODE). `azione`: 0 giù, 1 su; `meta`: modificatori
    /// (`KeyEvent.META_*`).
    pub async fn tasto(&mut self, azione: u8, codice: u32, meta: u32) -> Result<()> {
        let mut m = vec![0u8, azione];
        m.extend_from_slice(&codice.to_be_bytes());
        m.extend_from_slice(&0u32.to_be_bytes()); // ripetizione
        m.extend_from_slice(&meta.to_be_bytes());
        self.0.scrivi(&m).await
    }

    /// Mette il testo negli appunti del telefono e lo incolla (SET_CLIPBOARD
    /// con «incolla»): funziona con qualsiasi carattere.
    pub async fn incolla(&mut self, testo: &str) -> Result<()> {
        let mut m = vec![9u8];
        m.extend_from_slice(&0u64.to_be_bytes()); // nessuna conferma richiesta
        m.push(1);
        m.extend_from_slice(&(testo.len() as u32).to_be_bytes());
        m.extend_from_slice(testo.as_bytes());
        self.0.scrivi(&m).await
    }

    /// Chiude il canale: il componente sul telefono termina.
    pub async fn chiudi(self) -> Result<()> {
        self.0.chiudi().await
    }

    /// Rotellina del mouse (INJECT_SCROLL_EVENT) nel punto `x`,`y`: scatti
    /// verticali (positivo = su, come Android) e orizzontali (positivo = destra).
    pub async fn scorri(&mut self, x: i32, y: i32, larghezza: u16, altezza: u16, orizzontale: f32, verticale: f32) -> Result<()> {
        // Il server legge valori a virgola fissa in [-1, 1] e li moltiplica per 16.
        let fisso = |v: f32| ((v / 16.0).clamp(-1.0, 1.0) * 32767.0) as i16;
        let mut m = vec![3u8];
        m.extend_from_slice(&x.to_be_bytes());
        m.extend_from_slice(&y.to_be_bytes());
        m.extend_from_slice(&larghezza.to_be_bytes());
        m.extend_from_slice(&altezza.to_be_bytes());
        m.extend_from_slice(&fisso(orizzontale).to_be_bytes());
        m.extend_from_slice(&fisso(verticale).to_be_bytes());
        m.extend_from_slice(&0u32.to_be_bytes()); // nessun pulsante premuto
        self.0.scrivi(&m).await
    }

    /// Riavvia la codifica video (RESET_VIDEO): arrivano subito parametri e
    /// un fotogramma completo, da cui può partire una registrazione.
    pub async fn ricomincia_video(&mut self) -> Result<()> {
        self.0.scrivi(&[17u8]).await
    }

    /// Tasto Indietro (BACK_OR_SCREEN_ON, giù e su).
    pub async fn indietro(&mut self) -> Result<()> {
        self.0.scrivi(&[4u8, 0]).await?;
        self.0.scrivi(&[4u8, 1]).await
    }
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn numero_del_display() {
        assert_eq!(display_da_messaggio("[server] INFO: New display: 400x714/160 (id=57)"), Some(57));
        assert_eq!(display_da_messaggio("[server] INFO: Device display turned off"), None);
    }
}
