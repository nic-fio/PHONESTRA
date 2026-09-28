//! Phonestra: le app del telefono Android in finestre Linux, senza `adb`.
//!
//! - [`configurazione`]: cartelle XDG, chiave ADB, telefoni salvati;
//! - [`usb`]: stato del telefono collegato col cavo, letto da sysfs;
//! - [`rete`]: ricerca dei telefoni col Debug wireless via mDNS;
//! - [`telefono`]: collegamento (USB o Wi-Fi TLS) e comandi di shell;
//! - [`adb`]: livello ADB proprio, con più canali contemporanei (tappa 2);
//! - [`app`]: le app del telefono con nomi e icone, per il drawer (tappa 3);
//! - [`componente`]: il componente nostro sul telefono (servizio, canali, battito, custode);
//! - [`input_nostro`], [`prova_input`]: il modulo input del componente e le sue prove;
//! - [`video_nostro`]: il video col componente nostro (schermi, codifica, eventi delle app,
//!   formato dei pacchetti);
//! - [`misura_audio`]: misure dell'audio del telefono (studio, fase 0);
//! - [`audio_nostro`]: l'audio del telefono col componente nostro (loopback, AAC);
//! - [`azioni`]: installare, disinstallare, inviare file;
//! - [`avvisi`]: le notifiche nuove del telefono come notifiche del sistema;
//! - [`collegamento`]: il telefono attivo, condiviso da drawer e finestre;
//! - [`prepara`]: «Aggiungi un telefono», il primo collegamento senza cavo;
//! - [`procedura`]: la stessa col cavo, per Android 10 o precedente;
//! - [`cassetto`], [`finestra`]: l'interfaccia GTK4/libadwaita.

pub mod adb;
pub mod app;
pub mod appunti;
pub mod azioni;
pub mod componente;
pub mod input_nostro;
pub mod audio_nostro;
pub mod avvisi;
pub mod cassetto;
pub mod collegamento;
pub mod configurazione;
pub mod finestra;
pub mod foto;
pub mod misura_audio;
pub mod notifiche;
pub mod prepara;
pub mod procedura;
pub mod prova_input;
pub mod rete;
pub mod telefono;
pub mod usb;
pub mod video_nostro;

/// Messaggi di diagnosi, solo con `PHONESTRA_DEBUG=1`.
pub(crate) fn diagnosi(testo: &str) {
    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
        eprintln!("[diagnosi] {testo}");
    }
}

/// Il motore asincrono (tokio) condiviso: rete e telefono girano qui, fuori dal
/// thread dell'interfaccia GTK.
pub fn esecutore() -> &'static tokio::runtime::Runtime {
    static ESECUTORE: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    ESECUTORE.get_or_init(|| tokio::runtime::Runtime::new().expect("motore tokio"))
}
