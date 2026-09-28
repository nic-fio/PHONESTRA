//! Il modulo input del componente nostro, lato PC: tocchi, rotellina, tasti,
//! testo, «indietro» e appunti mandati al servizio sul canale comandi
//! (`Input.java` sul telefono; formato in `memoria/componente.md`, «Input»).
//!
//! [`InputNostro`] lavora su uno schermo (principale o virtuale). I messaggi
//! non aspettano il telefono (vanno in coda al canale comandi), la rotellina
//! passa i valori frazionari così come sono (entro ±16 scatti) e più dita
//! vanno in un solo messaggio.
//!
//! Le copie fatte sul telefono arrivano come messaggi spontanei
//! [`tipo::APPUNTI_CAMBIATI`] a [`crate::componente::Componente::ricevi`] (col
//! servizio condiviso: a chi si iscrive con
//! [`crate::componente::Condiviso::iscrivi`]): si leggono con [`Appunti::leggi`].
//!
//! In Phonestra ogni finestra ha il suo `InputNostro`, col mittente del
//! servizio condiviso del collegamento (`finestra.rs`).

use anyhow::{Result, bail};

use crate::componente::{Componente, Messaggio, Mittente};

/// Tipi dei messaggi del modulo input: fascia 0x50–0x5f del canale comandi
/// (0x40–0x4f è del video).
pub mod tipo {
    /// PC → servizio: tocchi di una o più dita.
    pub const TOCCHI: u8 = 0x50;
    /// PC → servizio: rotellina.
    pub const ROTELLINA: u8 = 0x51;
    /// PC → servizio: un tasto Android.
    pub const TASTO: u8 = 0x52;
    /// PC → servizio: testo coi tasti virtuali.
    pub const TESTO: u8 = 0x53;
    /// PC → servizio: «indietro».
    pub const INDIETRO: u8 = 0x54;
    /// PC → servizio: testo negli appunti (e incolla); risposta se l'id non è 0.
    pub const APPUNTI_SCRIVI: u8 = 0x55;
    /// PC → servizio, domanda: gli appunti attuali.
    pub const APPUNTI_LEGGI: u8 = 0x56;
    /// PC → servizio: avviso delle copie acceso o spento; risposta se l'id non è 0.
    pub const APPUNTI_ASCOLTA: u8 = 0x57;
    /// Servizio → PC, spontaneo: copia fatta sul telefono.
    pub const APPUNTI_CAMBIATI: u8 = 0x58;
    /// PC → servizio, domanda di diagnosi: eventi iniettati, falliti, scartati.
    pub const CONTEGGI: u8 = 0x5c;
    /// PC → servizio, domanda: comandi delle prove.
    pub const PROVA: u8 = 0x5d;
}

/// Azioni di tocchi e tasti.
pub const GIU: u8 = 0;
pub const SU: u8 = 1;
pub const MOVIMENTO: u8 = 2;

/// Identificativi dei puntatori usati da Phonestra.
pub const DITO_MOUSE: i64 = -1;
pub const DITO_GENERICO: i64 = -2;

/// Scatti massimi della rotellina in un messaggio.
const SCATTI_MASSIMI: f32 = 16.0;

/// Un dito in un messaggio [`tipo::TOCCHI`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tocco {
    /// Identificativo del PC: −1 mouse, −2 dito generico, altri per i gesti.
    pub dito: i64,
    pub azione: u8,
    pub x: i32,
    pub y: i32,
    /// 0–1: 1 appoggiato, 0 alzato.
    pub pressione: f32,
}

impl Tocco {
    /// Come oggi: pressione piena in giù e movimento, zero quando si alza.
    pub fn new(dito: i64, azione: u8, x: i32, y: i32) -> Self {
        Self { dito, azione, x, y, pressione: if azione == SU { 0.0 } else { 1.0 } }
    }
}

/// `display i32 · larghezza u16 · altezza u16 · n u8 · n × (dito i64 · azione u8
/// · x i32 · y i32 · pressione f32)`. La misura è quella dell'immagine su cui il
/// PC ha calcolato le coordinate: il telefono scala e scarta le misure vecchie.
/// Al massimo 255 dita per messaggio.
pub fn codifica_tocchi(display: i32, larghezza: u16, altezza: u16, tocchi: &[Tocco]) -> Result<Vec<u8>> {
    if tocchi.len() > 255 {
        bail!("troppi tocchi in un messaggio: {}", tocchi.len());
    }
    let mut m = Vec::with_capacity(9 + 21 * tocchi.len());
    m.extend_from_slice(&display.to_be_bytes());
    m.extend_from_slice(&larghezza.to_be_bytes());
    m.extend_from_slice(&altezza.to_be_bytes());
    m.push(tocchi.len() as u8);
    for t in tocchi {
        m.extend_from_slice(&t.dito.to_be_bytes());
        m.push(t.azione);
        m.extend_from_slice(&t.x.to_be_bytes());
        m.extend_from_slice(&t.y.to_be_bytes());
        m.extend_from_slice(&t.pressione.to_be_bytes());
    }
    Ok(m)
}

/// `display i32 · x i32 · y i32 · larghezza u16 · altezza u16 · orizzontale f32 ·
/// verticale f32`, scatti entro ±16 (come oggi).
pub fn codifica_rotellina(display: i32, x: i32, y: i32, larghezza: u16, altezza: u16, orizzontale: f32, verticale: f32) -> Vec<u8> {
    let limita = |v: f32| if v.is_finite() { v.clamp(-SCATTI_MASSIMI, SCATTI_MASSIMI) } else { 0.0 };
    let mut m = Vec::with_capacity(24);
    m.extend_from_slice(&display.to_be_bytes());
    m.extend_from_slice(&x.to_be_bytes());
    m.extend_from_slice(&y.to_be_bytes());
    m.extend_from_slice(&larghezza.to_be_bytes());
    m.extend_from_slice(&altezza.to_be_bytes());
    m.extend_from_slice(&limita(orizzontale).to_be_bytes());
    m.extend_from_slice(&limita(verticale).to_be_bytes());
    m
}

/// `display i32 · azione u8 · codice u32 · ripetizione u32 · meta u32`.
pub fn codifica_tasto(display: i32, azione: u8, codice: u32, ripetizione: u32, meta: u32) -> Vec<u8> {
    let mut m = Vec::with_capacity(17);
    m.extend_from_slice(&display.to_be_bytes());
    m.push(azione);
    m.extend_from_slice(&codice.to_be_bytes());
    m.extend_from_slice(&ripetizione.to_be_bytes());
    m.extend_from_slice(&meta.to_be_bytes());
    m
}

/// `display i32 · testo UTF-8` (per [`tipo::TESTO`]).
pub fn codifica_testo(display: i32, testo: &str) -> Vec<u8> {
    let mut m = display.to_be_bytes().to_vec();
    m.extend_from_slice(testo.as_bytes());
    m
}

/// `display i32 · azione u8` (per [`tipo::INDIETRO`]).
pub fn codifica_indietro(display: i32, azione: u8) -> Vec<u8> {
    let mut m = display.to_be_bytes().to_vec();
    m.push(azione);
    m
}

/// `display i32 · incolla u8 · testo UTF-8` (per [`tipo::APPUNTI_SCRIVI`]):
/// con `incolla` il tasto PASTE va allo schermo `display`.
pub fn codifica_appunti(display: i32, incolla: bool, testo: &str) -> Vec<u8> {
    let mut m = display.to_be_bytes().to_vec();
    m.push(incolla as u8);
    m.extend_from_slice(testo.as_bytes());
    m
}

/// Gli appunti del telefono, come li descrive il servizio (`stato u8 · testo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Appunti {
    /// Vuoti, o non di testo.
    Vuoti,
    Testo(String),
    /// Segnati come sensibili (password): il testo non esce dal telefono.
    Sensibili,
    /// Il servizio non è riuscito a leggerli: nel dubbio non passano (come oggi).
    Sconosciuti,
}

impl Appunti {
    pub fn leggi(dati: &[u8]) -> Result<Self> {
        let Some((&stato, testo)) = dati.split_first() else { bail!("stato degli appunti mancante") };
        Ok(match stato {
            0 => Self::Vuoti,
            1 => Self::Testo(String::from_utf8_lossy(testo).into_owned()),
            2 => Self::Sensibili,
            3 => Self::Sconosciuti,
            s => bail!("stato degli appunti sconosciuto: {s}"),
        })
    }

    /// La copia fatta sul telefono, se è un avviso [`tipo::APPUNTI_CAMBIATI`].
    pub fn da_avviso(m: &Messaggio) -> Option<Result<Self>> {
        (m.tipo == tipo::APPUNTI_CAMBIATI && !m.risposta()).then(|| Self::leggi(&m.dati))
    }
}

/// Tocchi, rotellina, tasti, testo, appunti e «indietro» per uno schermo.
/// `larghezza`/`altezza` sono sempre la misura dell'immagine su cui sono
/// calcolate le coordinate.
#[derive(Clone)]
pub struct InputNostro {
    mittente: Mittente,
    display: i32,
}

impl InputNostro {
    /// L'input dello schermo `display` (0 = principale).
    pub fn new(mittente: Mittente, display: i32) -> Self {
        Self { mittente, display }
    }

    pub fn display(&self) -> i32 {
        self.display
    }

    fn tocchi(&self, tocchi: &[Tocco], larghezza: u16, altezza: u16) -> Result<()> {
        for pezzo in tocchi.chunks(255) {
            self.mittente.manda(tipo::TOCCHI, codifica_tocchi(self.display, larghezza, altezza, pezzo)?)?;
        }
        Ok(())
    }

    /// Tocco o clic del mouse. `azione`: 0 giù, 1 su, 2 movimento. Con il solo
    /// pulsante principale Android lo tratta come un dito (come oggi).
    pub async fn tocco(&mut self, azione: u8, x: i32, y: i32, larghezza: u16, altezza: u16) -> Result<()> {
        self.tocchi(&[Tocco::new(DITO_MOUSE, azione, x, y)], larghezza, altezza)
    }

    /// Tocco di un dito (non del mouse): serve per la pressione lunga.
    pub async fn dito(&mut self, azione: u8, x: i32, y: i32, larghezza: u16, altezza: u16) -> Result<()> {
        self.dito_numero(DITO_GENERICO, azione, x, y, larghezza, altezza).await
    }

    /// Tocco del dito `numero` (per i gesti a più dita, come il pizzico).
    pub async fn dito_numero(&mut self, numero: i64, azione: u8, x: i32, y: i32, larghezza: u16, altezza: u16) -> Result<()> {
        self.tocchi(&[Tocco::new(numero, azione, x, y)], larghezza, altezza)
    }

    /// Più tocchi di dita (`(numero, azione, x, y)`) in un solo invio: sul
    /// telefono un evento ciascuno, nell'ordine.
    pub async fn dita_insieme(&mut self, tocchi: &[(i64, u8, i32, i32)], larghezza: u16, altezza: u16) -> Result<()> {
        let tocchi: Vec<Tocco> = tocchi.iter().map(|&(n, azione, x, y)| Tocco::new(n, azione, x, y)).collect();
        self.tocchi(&tocchi, larghezza, altezza)
    }

    /// Rotellina del mouse nel punto `x`,`y`: scatti verticali (positivo = su,
    /// come Android) e orizzontali (positivo = destra), anche frazionari.
    pub async fn scorri(&mut self, x: i32, y: i32, larghezza: u16, altezza: u16, orizzontale: f32, verticale: f32) -> Result<()> {
        self.mittente.manda(tipo::ROTELLINA, codifica_rotellina(self.display, x, y, larghezza, altezza, orizzontale, verticale))
    }

    /// Tasto Android. `azione`: 0 giù, 1 su; `meta`: modificatori (`KeyEvent.META_*`).
    pub async fn tasto(&mut self, azione: u8, codice: u32, meta: u32) -> Result<()> {
        self.mittente.manda(tipo::TASTO, codifica_tasto(self.display, azione, codice, 0, meta))
    }

    /// Testo già composto dal PC: Android lo accetta solo per i caratteri della
    /// sua mappa virtuale (in pratica ASCII); per gli altri usare
    /// [`InputNostro::incolla`].
    pub async fn testo(&mut self, testo: &str) -> Result<()> {
        self.mittente.manda(tipo::TESTO, codifica_testo(self.display, testo))
    }

    /// Mette il testo negli appunti del telefono e lo incolla: funziona con
    /// qualsiasi carattere.
    pub async fn incolla(&mut self, testo: &str) -> Result<()> {
        self.mittente.manda(tipo::APPUNTI_SCRIVI, codifica_appunti(self.display, true, testo))
    }

    /// Tasto Indietro, giù e su.
    pub async fn indietro(&mut self) -> Result<()> {
        self.mittente.manda(tipo::INDIETRO, codifica_indietro(self.display, GIU))?;
        self.mittente.manda(tipo::INDIETRO, codifica_indietro(self.display, SU))
    }
}

/// Accende o spegne l'avviso delle copie fatte sul telefono (messaggi
/// [`tipo::APPUNTI_CAMBIATI`]); oggi è sempre acceso finché il telefono è collegato.
pub async fn ascolta_appunti(componente: &mut Componente, attivo: bool) -> Result<()> {
    componente.richiesta(tipo::APPUNTI_ASCOLTA, vec![attivo as u8]).await?;
    Ok(())
}

/// Gli appunti attuali del telefono (il testo solo se non sono sensibili).
pub async fn leggi_appunti(componente: &mut Componente) -> Result<Appunti> {
    Appunti::leggi(&componente.richiesta(tipo::APPUNTI_LEGGI, Vec::new()).await?.dati)
}

/// Mette il testo negli appunti del telefono senza incollarlo, e aspetta che
/// sia fatto.
pub async fn scrivi_appunti(componente: &mut Componente, testo: &str) -> Result<()> {
    componente.richiesta(tipo::APPUNTI_SCRIVI, codifica_appunti(0, false, testo)).await?;
    Ok(())
}

/// Conteggi del modulo input sul telefono (righe `chiave=valore`): eventi
/// iniettati, falliti, scartati (misura vecchia, troppe dita, caratteri senza
/// tasto), avvisi delle copie mandati, ultimo errore.
pub async fn conteggi(componente: &mut Componente) -> Result<Vec<(String, String)>> {
    let m = componente.richiesta(tipo::CONTEGGI, Vec::new()).await?;
    Ok(crate::componente::Ciao::leggi(&m.dati).voci)
}

#[cfg(test)]
mod prove {
    use super::*;

    /// Valore di `static final int <nome> = 0x..;` nel sorgente Java.
    fn costante_java(sorgente: &str, nome: &str) -> u8 {
        let riga = sorgente
            .lines()
            .find(|r| r.trim().starts_with(&format!("static final int {nome} = 0x")))
            .unwrap_or_else(|| panic!("{nome} non trovato"));
        let cifre = riga.split("0x").nth(1).unwrap().trim_end_matches(';').trim();
        u8::from_str_radix(cifre, 16).unwrap()
    }

    #[test]
    fn nessun_tipo_usato_da_due_moduli() {
        use crate::componente::tipo as c;
        let input = [
            ("TOCCHI", tipo::TOCCHI),
            ("ROTELLINA", tipo::ROTELLINA),
            ("TASTO", tipo::TASTO),
            ("TESTO", tipo::TESTO),
            ("INDIETRO", tipo::INDIETRO),
            ("APPUNTI_SCRIVI", tipo::APPUNTI_SCRIVI),
            ("APPUNTI_LEGGI", tipo::APPUNTI_LEGGI),
            ("APPUNTI_ASCOLTA", tipo::APPUNTI_ASCOLTA),
            ("APPUNTI_CAMBIATI", tipo::APPUNTI_CAMBIATI),
            ("CONTEGGI", tipo::CONTEGGI),
            ("PROVA", tipo::PROVA),
        ];
        let video = [
            ("APRI", c::VIDEO_APRI),
            ("CHIUDI", c::VIDEO_CHIUDI),
            ("AVVIA_APP", c::VIDEO_AVVIA_APP),
            ("RIDIMENSIONA", c::VIDEO_RIDIMENSIONA),
            ("CHIAVE", c::VIDEO_CHIAVE),
            ("PANNELLO", c::VIDEO_PANNELLO),
            ("EVENTO", c::VIDEO_EVENTO),
        ];
        let base = [c::CIAO, c::BATTITO, c::FINE, c::ERRORE, c::PROVA_CUSTODE];
        let mut tutti: Vec<u8> = base.to_vec();
        tutti.extend(input.iter().map(|(_, v)| *v));
        tutti.extend(video.iter().map(|(_, v)| *v));
        let mut ordinati = tutti.clone();
        ordinati.sort();
        ordinati.dedup();
        assert_eq!(ordinati.len(), tutti.len(), "un tipo di messaggio è usato due volte: {tutti:02x?}");
        assert!(input.iter().all(|(_, v)| (0x50..=0x5f).contains(v)), "input fuori dalla sua fascia");
        assert!(video.iter().all(|(_, v)| (0x40..=0x4f).contains(v)), "video fuori dalla sua fascia");
        // Telefono e PC devono avere gli stessi numeri.
        let java_input = include_str!("../telefono/aiuto/src/phonestra/Input.java");
        for (nome, v) in input {
            assert_eq!(costante_java(java_input, nome), v, "Input.{nome}");
        }
        let java_video = include_str!("../telefono/aiuto/src/phonestra/Video.java");
        for (nome, v) in video {
            assert_eq!(costante_java(java_video, nome), v, "Video.{nome}");
        }
    }

    #[test]
    fn tocchi_in_byte() {
        let m = codifica_tocchi(7, 720, 1280, &[Tocco::new(-2, GIU, 10, 20), Tocco::new(11, SU, -1, 300)]).unwrap();
        assert_eq!(m.len(), 9 + 2 * 21);
        assert_eq!(&m[..9], &[0, 0, 0, 7, 0x02, 0xd0, 0x05, 0x00, 2]);
        // Primo dito: −2, giù, 10, 20, pressione 1.0.
        assert_eq!(&m[9..17], &(-2i64).to_be_bytes());
        assert_eq!(m[17], GIU);
        assert_eq!(&m[18..26], &[0, 0, 0, 10, 0, 0, 0, 20]);
        assert_eq!(&m[26..30], &[0x3f, 0x80, 0, 0]);
        // Secondo dito: 11, su, x negativo, pressione 0.
        assert_eq!(&m[30..38], &11i64.to_be_bytes());
        assert_eq!(m[38], SU);
        assert_eq!(&m[39..43], &[0xff, 0xff, 0xff, 0xff]);
        assert_eq!(&m[47..51], &[0, 0, 0, 0]);
        assert!(codifica_tocchi(0, 1, 1, &vec![Tocco::new(0, GIU, 0, 0); 256]).is_err());
    }

    #[test]
    fn rotellina_frazionaria_e_limitata() {
        let m = codifica_rotellina(3, 100, 200, 720, 1280, 0.25, -40.0);
        assert_eq!(m.len(), 24);
        assert_eq!(&m[..12], &[0, 0, 0, 3, 0, 0, 0, 100, 0, 0, 0, 200]);
        assert_eq!(&m[12..16], &[0x02, 0xd0, 0x05, 0x00]);
        assert_eq!(f32::from_be_bytes(m[16..20].try_into().unwrap()), 0.25);
        assert_eq!(f32::from_be_bytes(m[20..24].try_into().unwrap()), -16.0);
        let nan = codifica_rotellina(0, 0, 0, 1, 1, f32::NAN, f32::INFINITY);
        assert_eq!(f32::from_be_bytes(nan[16..20].try_into().unwrap()), 0.0);
        assert_eq!(f32::from_be_bytes(nan[20..24].try_into().unwrap()), 0.0);
    }

    #[test]
    fn tasti_testo_indietro_appunti() {
        // Ctrl+C: codice 31, META_CTRL_ON | META_CTRL_LEFT_ON.
        assert_eq!(codifica_tasto(0, GIU, 31, 0, 0x3000), vec![0, 0, 0, 0, 0, 0, 0, 0, 31, 0, 0, 0, 0, 0, 0, 0x30, 0]);
        assert_eq!(codifica_testo(-1, "a"), vec![0xff, 0xff, 0xff, 0xff, b'a']);
        assert_eq!(codifica_indietro(2, SU), vec![0, 0, 0, 2, 1]);
        assert_eq!(codifica_appunti(5, true, "à"), vec![0, 0, 0, 5, 1, 0xc3, 0xa0]);
    }

    #[test]
    fn stato_degli_appunti() {
        assert_eq!(Appunti::leggi(&[1, b'o', b'k']).unwrap(), Appunti::Testo("ok".into()));
        assert_eq!(Appunti::leggi(&[0]).unwrap(), Appunti::Vuoti);
        assert_eq!(Appunti::leggi(&[2]).unwrap(), Appunti::Sensibili);
        assert_eq!(Appunti::leggi(&[3]).unwrap(), Appunti::Sconosciuti);
        assert!(Appunti::leggi(&[]).is_err());
        assert!(Appunti::leggi(&[9]).is_err());
        let avviso = Messaggio::new(tipo::APPUNTI_CAMBIATI, vec![1, b'x']);
        assert_eq!(Appunti::da_avviso(&avviso).unwrap().unwrap(), Appunti::Testo("x".into()));
        assert!(Appunti::da_avviso(&Messaggio::new(tipo::TESTO, vec![])).is_none());
    }

    #[test]
    fn stesse_azioni_di_oggi() {
        // Pressione piena in giù e movimento, zero in su.
        assert_eq!(Tocco::new(-1, GIU, 0, 0).pressione, 1.0);
        assert_eq!(Tocco::new(-1, MOVIMENTO, 0, 0).pressione, 1.0);
        assert_eq!(Tocco::new(-1, SU, 0, 0).pressione, 0.0);
    }

    #[tokio::test]
    async fn i_metodi_mandano_i_messaggi_giusti() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut input = InputNostro::new(Mittente::per_prova(tx), 9);
        input.tocco(GIU, 1, 2, 10, 20).await.unwrap();
        input.dita_insieme(&[(10, GIU, 1, 1), (11, GIU, 2, 2)], 10, 20).await.unwrap();
        input.indietro().await.unwrap();
        input.incolla("è").await.unwrap();
        let m = rx.recv().await.unwrap();
        assert_eq!((m.tipo, m.id), (tipo::TOCCHI, 0));
        assert_eq!(m.dati, codifica_tocchi(9, 10, 20, &[Tocco::new(DITO_MOUSE, GIU, 1, 2)]).unwrap());
        let m = rx.recv().await.unwrap();
        assert_eq!(m.dati[8], 2);
        assert_eq!(rx.recv().await.unwrap().dati, codifica_indietro(9, GIU));
        assert_eq!(rx.recv().await.unwrap().dati, codifica_indietro(9, SU));
        let m = rx.recv().await.unwrap();
        assert_eq!((m.tipo, m.dati), (tipo::APPUNTI_SCRIVI, codifica_appunti(9, true, "è")));
        drop(rx);
        assert!(input.testo("a").await.is_err());
    }
}
