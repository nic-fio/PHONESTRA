//! Il video col componente nostro (lato PC): schermi virtuali per le app,
//! specchio dello schermo principale, codifica sul telefono, eventi delle app,
//! schermate protette, pannello. Sul telefono: `Video.java`,
//! `SessioneVideo.java`, `Codifica.java`, `EventiApp.java`, `Protetta.java`,
//! `Pannello.java`. Formato e scelte in `memoria/componente.md`, «Video».
//!
//! L'interfaccia imita quella che finestra e drawer usano oggi da
//! [`crate::sessione`] con scrcpy, così il passaggio è una sostituzione:
//!
//! | oggi (scrcpy) | componente nostro |
//! |---|---|
//! | `Sessione::avvia(adb, &Opzioni)` | [`SessioneNostra::avvia`]`(&servizio, &Opzioni)` (stesse [`Opzioni`]) |
//! | `sessione.video` + [`leggi_pacchetto`](crate::sessione::leggi_pacchetto) | uguale: stesso formato dei pacchetti |
//! | `sessione.codec`, `nome_dispositivo` | uguali |
//! | display dal messaggio «New display» del server | [`SessioneNostra::display`], subito |
//! | `Comandi::avvia_app`, `pannello`, `ridimensiona`, `ricomincia_video` | [`ComandiVideo`], stessi nomi |
//! | `am start … APPLICATION_DETAILS_SETTINGS` dal PC | [`ComandiVideo::informazioni_app`] |
//! | `cmd window set-ignore-orientation-request …` dal PC | lo fa il telefono all'apertura |
//! | `togli_dalle_recenti(adb, display)` | [`ComandiVideo::chiudi`]`(true)` |
//! | `dumpsys` per l'orientamento (`chiedi_orientamento`) | [`Evento::Orientamento`] |
//! | `dumpsys window` per le schermate protette | [`Evento::Protetta`] |
//!
//! Tocchi, tasti, appunti restano del pezzo «input». Un solo servizio serve
//! tutte le finestre: il [`Condiviso`] del collegamento, che smista risposte ed
//! eventi (`componente.rs`).

pub mod prova;

use std::collections::HashMap;

use anyhow::{Context, Result, anyhow, bail};
use tokio::sync::mpsc;

use crate::adb::Canale;
use crate::componente::{Condiviso, Messaggio, tipo};
use crate::sessione::Opzioni;

/// Id del codec come quelli annunciati da scrcpy ([`crate::sessione::nome_codec`]).
pub fn id_codec(nome: &str) -> u32 {
    match nome {
        "h265" => 0x6832_3635,
        "av1" => 0x0061_7631,
        _ => 0x6832_3634,
    }
}

/// Righe `chiave=valore` (il contenuto dei messaggi del video).
pub fn coppie(voci: &[(&str, String)]) -> Vec<u8> {
    let mut t = String::new();
    for (k, v) in voci {
        t.push_str(k);
        t.push('=');
        t.push_str(v);
        t.push('\n');
    }
    t.into_bytes()
}

/// Da righe `chiave=valore` a tabella.
pub fn leggi_coppie(dati: &[u8]) -> HashMap<String, String> {
    String::from_utf8_lossy(dati)
        .lines()
        .filter_map(|r| r.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

/// Un valore non ammesso nei messaggi (spazi o a capo lo spezzerebbero).
fn valore_valido(v: &str) -> Result<&str> {
    if v.is_empty() || v.chars().any(|c| c.is_whitespace() || c == '=') {
        bail!("valore non valido per il telefono: {v:?}");
    }
    Ok(v)
}

/// Contenuto di `VIDEO_APRI` per le [`Opzioni`] di oggi.
pub fn richiesta_apertura(opzioni: &Opzioni) -> Vec<u8> {
    let codec = opzioni.codec.to_string();
    if opzioni.specchio {
        // Oggi: `max_size=1920` sullo schermo principale.
        return coppie(&[("specchio", "1".into()), ("lato_massimo", "1920".into()), ("codec", codec)]);
    }
    let (l, a, d) = opzioni.display;
    coppie(&[("larghezza", l.to_string()), ("altezza", a.to_string()), ("dpi", d.to_string()), ("codec", codec)])
}

/// La risposta a `VIDEO_APRI`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Apertura {
    pub id: u32,
    /// Schermo virtuale dell'app, o 0 per lo specchio dello schermo principale.
    pub display: i32,
    pub codec: u32,
    /// Misura allineata dal telefono (quella vera del video).
    pub larghezza: u32,
    pub altezza: u32,
}

pub fn leggi_apertura(dati: &[u8]) -> Result<Apertura> {
    let c = leggi_coppie(dati);
    let numero = |k: &str| -> Result<i64> {
        c.get(k).and_then(|v| v.parse().ok()).ok_or_else(|| anyhow!("risposta di apertura senza «{k}»"))
    };
    Ok(Apertura {
        id: numero("id")? as u32,
        display: numero("display")? as i32,
        codec: id_codec(c.get("codec").map(String::as_str).unwrap_or("h264")),
        larghezza: numero("larghezza")? as u32,
        altezza: numero("altezza")? as u32,
    })
}

/// Un evento di una sessione video, mandato dal telefono quando succede.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evento {
    /// L'app in vista accetta solo il verticale (`verticale`) o no; `valore` è
    /// l'orientamento chiesto (`ActivityInfo.SCREEN_ORIENTATION_*`). Arriva
    /// all'avvio e poi quando cambia (oggi: `chiedi_orientamento`).
    Orientamento { display: i32, verticale: bool, valore: i32 },
    /// Schermata protetta (FLAG_SECURE: nera nel video) presente o no. Arriva
    /// all'avvio e poi quando cambia (oggi: `Collegamento::protetti`).
    Protetta { display: i32, protetta: bool },
    /// Un task dello schermo è passato sullo schermo `display` (l'app aperta
    /// anche sul telefono: «App aperta sul telefono – riportala qui»).
    Spostata { task: i32, display: i32 },
    /// Un task dello schermo è stato chiuso.
    Rimosso { task: i32 },
    /// Il telefono ha chiuso la sessione (codificatore fermo, canale non aperto…).
    Fine { motivo: String },
}

impl Evento {
    /// `(id della sessione, evento)` dal contenuto di `VIDEO_EVENTO`.
    pub fn leggi(dati: &[u8]) -> Option<(u32, Evento)> {
        let c = leggi_coppie(dati);
        let id = c.get("id")?.parse().ok()?;
        let intero = |k: &str| c.get(k).and_then(|v| v.parse::<i32>().ok());
        let evento = match c.get("evento")?.as_str() {
            "orientamento" => Evento::Orientamento {
                display: intero("display")?,
                verticale: intero("verticale")? != 0,
                valore: intero("valore").unwrap_or(-1),
            },
            "protetta" => Evento::Protetta { display: intero("display")?, protetta: intero("protetta")? != 0 },
            "spostata" => Evento::Spostata { task: intero("task")?, display: intero("display")? },
            "rimosso" => Evento::Rimosso { task: intero("task")? },
            "fine" => Evento::Fine { motivo: c.get("motivo").cloned().unwrap_or_default() },
            _ => return None,
        };
        Some((id, evento))
    }
}

/// Gli eventi di una sessione video, dal canale comandi del servizio condiviso.
pub struct Eventi(mpsc::UnboundedReceiver<Messaggio>);

impl Eventi {
    /// Il prossimo evento; `None` quando la sessione è chiusa o il telefono è perso.
    pub async fn recv(&mut self) -> Option<Evento> {
        loop {
            let m = self.0.recv().await?;
            match Evento::leggi(&m.dati) {
                Some((_, e)) => return Some(e),
                None => eprintln!("[video] evento non capito: {}", m.testo().replace('\n', " ")),
            }
        }
    }
}

/// Accende o spegne il pannello fisico del telefono (per tutto il telefono,
/// non per una finestra), senza aspettare. Spento, il custode lo riaccende se
/// il servizio muore.
pub fn pannello(servizio: &Condiviso, acceso: bool) -> Result<()> {
    servizio.manda(tipo::VIDEO_PANNELLO, coppie(&[("acceso", (acceso as u8).to_string())]))
}

/// Come [`pannello`], aspettando che il telefono l'abbia fatto.
pub async fn pannello_atteso(servizio: &Condiviso, acceso: bool) -> Result<String> {
    let m = servizio.domanda(tipo::VIDEO_PANNELLO, coppie(&[("acceso", (acceso as u8).to_string())])).await?;
    Ok(m.testo())
}

/// Una finestra col componente nostro: come [`crate::sessione::Sessione`].
pub struct SessioneNostra {
    pub nome_dispositivo: String,
    /// Id del codec, come `Sessione::codec`.
    pub codec: u32,
    /// Schermo virtuale dell'app (o 0 per lo schermo principale), noto subito.
    pub display: i32,
    /// Flusso video: da leggere con [`crate::sessione::leggi_pacchetto`] in un
    /// compito dedicato, come oggi.
    pub video: Canale,
    pub comandi: ComandiVideo,
    /// Eventi della sessione; `None` quando è chiusa o il telefono è perso.
    pub eventi: Eventi,
}

impl SessioneNostra {
    /// Apre lo schermo (o lo specchio) e il suo canale video. L'app si avvia
    /// dopo con [`ComandiVideo::avvia_app`], come oggi.
    pub async fn avvia(servizio: &Condiviso, opzioni: &Opzioni) -> Result<Self> {
        let (risposta, eventi) =
            servizio.apri_sessione(tipo::VIDEO_APRI, richiesta_apertura(opzioni)).await.context("apertura del video")?;
        let apertura = leggi_apertura(&risposta.dati)?;
        let canale = match servizio.apritore().apri(&format!("video:{}", apertura.id)).await {
            Ok(c) => c,
            Err(e) => {
                let _ = servizio.manda(tipo::VIDEO_CHIUDI, coppie(&[("id", apertura.id.to_string())]));
                servizio.dimentica(apertura.id);
                return Err(e.context("canale del video"));
            }
        };
        Ok(Self {
            nome_dispositivo: servizio.nome_dispositivo(),
            codec: apertura.codec,
            display: apertura.display,
            video: canale,
            comandi: ComandiVideo {
                servizio: servizio.clone(),
                id: apertura.id,
                ridimensionabile: opzioni.ridimensionabile && !opzioni.specchio,
            },
            eventi: Eventi(eventi),
        })
    }
}

/// I comandi del video di una finestra, come quelli di
/// [`crate::sessione::Comandi`] (senza tocchi e tasti: pezzo «input»).
pub struct ComandiVideo {
    servizio: Condiviso,
    id: u32,
    ridimensionabile: bool,
}

impl ComandiVideo {
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Avvia un'app sullo schermo della finestra (START_APP di oggi).
    pub async fn avvia_app(&mut self, pacchetto: &str) -> Result<()> {
        let dati = coppie(&[("id", self.id.to_string()), ("app", valore_valido(pacchetto)?.to_string())]);
        self.servizio.manda(tipo::VIDEO_AVVIA_APP, dati)
    }

    /// Apre la pagina «Informazioni app» di `pacchetto` sullo schermo della finestra.
    pub async fn informazioni_app(&mut self, pacchetto: &str) -> Result<()> {
        let dati = coppie(&[("id", self.id.to_string()), ("informazioni", valore_valido(pacchetto)?.to_string())]);
        self.servizio.manda(tipo::VIDEO_AVVIA_APP, dati)
    }

    /// Accende o spegne il pannello fisico (SET_DISPLAY_POWER di oggi).
    pub async fn pannello(&mut self, acceso: bool) -> Result<()> {
        pannello(&self.servizio, acceso)
    }

    /// Nuova misura dello schermo (RESIZE_DISPLAY di oggi, solo se
    /// ridimensionabile): la densità resta quella iniziale; il telefono manda un
    /// pacchetto `Pacchetto::Dimensione` se la misura (allineata) cambia davvero.
    pub async fn ridimensiona(&mut self, larghezza: u16, altezza: u16) -> Result<()> {
        if !self.ridimensionabile {
            return Ok(());
        }
        let dati = coppie(&[
            ("id", self.id.to_string()),
            ("larghezza", larghezza.to_string()),
            ("altezza", altezza.to_string()),
        ]);
        self.servizio.manda(tipo::VIDEO_RIDIMENSIONA, dati)
    }

    /// Fotogramma chiave coi parametri del codec davanti (RESET_VIDEO di oggi,
    /// ma senza ricreare il codificatore): da lì può partire una registrazione.
    pub async fn ricomincia_video(&mut self) -> Result<()> {
        self.servizio.manda(tipo::VIDEO_CHIAVE, coppie(&[("id", self.id.to_string())]))
    }

    /// Chiude la sessione sul telefono e aspetta che l'abbia fatto. Con
    /// `togli_dalle_recenti` le app dello schermo si tolgono dalle recenti
    /// (finestra chiusa dall'utente, come `togli_dalle_recenti` di oggi).
    pub async fn chiudi(self, togli_dalle_recenti: bool) -> Result<()> {
        let mut voci = vec![("id", self.id.to_string())];
        if togli_dalle_recenti {
            voci.push(("togli_task", "1".into()));
        }
        let esito = self.servizio.domanda(tipo::VIDEO_CHIUDI, coppie(&voci)).await.map(|_| ());
        self.servizio.dimentica(self.id);
        esito
    }
}

#[cfg(test)]
mod prove {
    use super::*;
    use crate::sessione::{Intestazione, Pacchetto, intestazione};

    fn esadecimale(s: &str) -> Vec<u8> {
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    /// Byte prodotti da `SessioneVideo.misura/dati` (Java), letti come li legge il PC.
    #[test]
    fn pacchetti_del_telefono() {
        match intestazione(&esadecimale("8000000000000460000007c8")) {
            Intestazione::Dimensione(Pacchetto::Dimensione { larghezza, altezza }) => {
                assert_eq!((larghezza, altezza), (1120, 1992))
            }
            _ => panic!("attesa una misura"),
        }
        match intestazione(&esadecimale("400000000000000000000005")) {
            Intestazione::Dati { config, chiave, lunghezza, .. } => assert!(config && !chiave && lunghezza == 5),
            _ => panic!("attesi dati"),
        }
        match intestazione(&esadecimale("20000000075bcd1500000001")) {
            Intestazione::Dati { pts, config, chiave, lunghezza } => {
                assert_eq!((pts, config, chiave, lunghezza), (123_456_789, false, true, 1))
            }
            _ => panic!("attesi dati"),
        }
    }

    #[test]
    fn apertura_richiesta_e_risposta() {
        let o = Opzioni { display: (1120, 1992, 448), codec: "h265", ..Opzioni::default() };
        let r = leggi_coppie(&richiesta_apertura(&o));
        assert_eq!(r["larghezza"], "1120");
        assert_eq!(r["dpi"], "448");
        assert_eq!(r["codec"], "h265");
        assert!(!r.contains_key("specchio"));
        let s = leggi_coppie(&richiesta_apertura(&Opzioni { specchio: true, ..Opzioni::default() }));
        assert_eq!((s["specchio"].as_str(), s["lato_massimo"].as_str()), ("1", "1920"));

        let a = leggi_apertura(b"id=3\ndisplay=57\ncodec=h265\nlarghezza=1112\naltezza=1992\navvio=avvio di x\n").unwrap();
        assert_eq!(a, Apertura { id: 3, display: 57, codec: 0x6832_3635, larghezza: 1112, altezza: 1992 });
        assert_eq!(crate::sessione::nome_codec(a.codec), "h265");
        assert!(leggi_apertura(b"display=1\n").is_err());
    }

    #[test]
    fn eventi_del_telefono() {
        let e = |t: &str| Evento::leggi(t.as_bytes());
        assert_eq!(
            e("evento=orientamento\nid=2\ndisplay=57\nverticale=1\nvalore=1\n"),
            Some((2, Evento::Orientamento { display: 57, verticale: true, valore: 1 }))
        );
        assert_eq!(
            e("evento=protetta\nid=2\ndisplay=0\nprotetta=0\n"),
            Some((2, Evento::Protetta { display: 0, protetta: false }))
        );
        assert_eq!(e("evento=spostata\nid=4\ntask=120\ndisplay=0\n"), Some((4, Evento::Spostata { task: 120, display: 0 })));
        assert_eq!(e("evento=rimosso\nid=4\ntask=120\n"), Some((4, Evento::Rimosso { task: 120 })));
        assert_eq!(
            e("evento=fine\nid=1\nmotivo=codificatore fermo: x=y\n"),
            Some((1, Evento::Fine { motivo: "codificatore fermo: x=y".into() }))
        );
        assert_eq!(e("evento=nuovo\nid=1\n"), None);
        assert_eq!(e("evento=protetta\nid=1\n"), None);
    }

    #[test]
    fn valori_dei_messaggi() {
        assert!(valore_valido("com.android.chrome").is_ok());
        assert!(valore_valido("a b").is_err());
        assert!(valore_valido("a\nid=9").is_err());
        assert!(valore_valido("").is_err());
        assert_eq!(coppie(&[("id", "1".into()), ("app", "x".into())]), b"id=1\napp=x\n");
    }
}
