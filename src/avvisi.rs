//! Avvisi a comparsa: le notifiche nuove del telefono diventano notifiche del
//! sistema (SPECIFICATION §8), col servizio standard `org.freedesktop.Notifications`
//! (GNOME, KDE, Xfce). Le notifiche di GTK non si usano: su GNOME funzionano
//! solo per i programmi con un file `.desktop`, che Phonestra non installa.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::gio;
use gtk::glib;
use gtk::glib::prelude::*;

use crate::t;

const SERVIZIO: &str = "org.freedesktop.Notifications";
const PERCORSO: &str = "/org/freedesktop/Notifications";

pub struct Avvisi {
    connessione: gio::DBusConnection,
    /// Avviso mostrato (numero del sistema) → pacchetto dell'app da aprire.
    aperti: Rc<RefCell<HashMap<u32, String>>>,
    _iscrizione: gio::SignalSubscription,
}

impl Avvisi {
    /// `apri` riceve il pacchetto quando l'utente fa clic su un avviso.
    pub fn nuovi(apri: impl Fn(String) + 'static) -> Option<Self> {
        let connessione = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>)
            .map_err(|e| eprintln!("[avvisi] bus di sessione non disponibile: {e}"))
            .ok()?;
        let aperti: Rc<RefCell<HashMap<u32, String>>> = Rc::default();
        let a = aperti.clone();
        let iscrizione = connessione.subscribe_to_signal(
            Some(SERVIZIO),
            Some(SERVIZIO),
            Some("ActionInvoked"),
            Some(PERCORSO),
            None,
            gio::DBusSignalFlags::NONE,
            move |segnale| {
                if let Some((id, _azione)) = segnale.parameters.get::<(u32, String)>()
                    && let Some(pacchetto) = a.borrow_mut().remove(&id)
                {
                    apri(pacchetto);
                }
            },
        );
        Some(Self { connessione, aperti, _iscrizione: iscrizione })
    }

    /// Mostra un avviso; il clic apre `pacchetto`. `icona` è il PNG dell'app.
    pub fn mostra(&self, titolo: &str, testo: &str, icona: Option<&[u8]>, pacchetto: &str) {
        let percorso_icona = icona.and_then(|png| salva_icona(pacchetto, png)).unwrap_or_else(|| "phone-symbolic".into());
        let azioni = vec!["default".to_string(), t!("Apri").to_string()];
        let suggerimenti: HashMap<String, glib::Variant> = HashMap::from([("category".to_string(), "im.received".to_variant())]);
        let parametri = ("Phonestra", 0u32, percorso_icona, titolo, testo, azioni, suggerimenti, -1i32).to_variant();
        let (connessione, aperti, pacchetto) = (self.connessione.clone(), self.aperti.clone(), pacchetto.to_string());
        glib::spawn_future_local(async move {
            let risposta = connessione
                .call_future(
                    Some(SERVIZIO),
                    PERCORSO,
                    SERVIZIO,
                    "Notify",
                    Some(&parametri),
                    Some(glib::VariantTy::new("(u)").unwrap()),
                    gio::DBusCallFlags::NONE,
                    5000,
                )
                .await;
            match risposta.map(|r| r.get::<(u32,)>()) {
                Ok(Some((id,))) => {
                    aperti.borrow_mut().insert(id, pacchetto);
                }
                Ok(None) => eprintln!("[avvisi] risposta inattesa"),
                Err(e) => eprintln!("[avvisi] avviso non mostrato: {e}"),
            }
        });
    }
}

/// L'icona dell'app in un file (il servizio delle notifiche vuole un
/// percorso): `~/.config/Phonestra/icone/<pacchetto>.png`.
fn salva_icona(pacchetto: &str, png: &[u8]) -> Option<String> {
    let cartella = crate::configurazione::cartella().ok()?.join("icone");
    std::fs::create_dir_all(&cartella).ok()?;
    let percorso = cartella.join(format!("{pacchetto}.png"));
    if std::fs::read(&percorso).ok().as_deref() != Some(png) {
        std::fs::write(&percorso, png).ok()?;
    }
    Some(percorso.to_string_lossy().into_owned())
}
