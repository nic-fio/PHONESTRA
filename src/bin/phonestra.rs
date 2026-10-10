// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! `phonestra [pacchetto]`: il drawer con le app del telefono; ogni app si apre
//! nella sua finestra. Con un pacchetto apre subito anche quell'app.
//!
//! Un solo processo per tutte le finestre (un secondo avvio riporta in primo
//! piano il drawer) e un solo collegamento al telefono. Quando l'ultima
//! finestra si chiude, il telefono viene rimesso com'era, poi si esce.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use adw::prelude::*;

use phonestra::collegamento::{Collegamento, Stato};
use phonestra::{cassetto, esecutore, finestra, prepara};

/// Il collegamento e il drawer, dopo il primo avvio.
type Avviato = (Arc<Collegamento>, gtk::glib::WeakRef<adw::ApplicationWindow>);

fn main() -> gtk::glib::ExitCode {
    if let Err(e) = gst::init() {
        eprintln!("GStreamer non disponibile: {e}");
        return gtk::glib::ExitCode::FAILURE;
    }
    let pacchetto = std::env::args().nth(1);
    let app = adw::Application::builder().application_id("io.github.nic_fio.Phonestra").build();
    let stato: Rc<RefCell<Option<Avviato>>> = Rc::default();
    app.connect_activate(move |app| {
        // Secondo avvio: il drawer torna in primo piano (o si riapre).
        if let Some((collegamento, drawer)) = stato.borrow_mut().as_mut() {
            match drawer.upgrade() {
                Some(d) => d.present(),
                None => *drawer = cassetto::apri(app, collegamento.clone()).downgrade(),
            }
            return;
        }
        let collegamento = match Collegamento::primo_telefono() {
            Ok(c) => c,
            Err(e) => {
                // Nessun telefono configurato: la procedura guidata; alla fine
                // si riparte da qui, col telefono appena aggiunto.
                eprintln!("{e:#}");
                let a = app.downgrade();
                prepara::apri(app, move |_| {
                    if let Some(a) = a.upgrade() {
                        a.activate();
                    }
                });
                return;
            }
        };
        esecutore().spawn(collegamento.clone().mantieni());

        // Copie fatte sul telefono → appunti del PC. Su Wayland (GNOME) gli
        // appunti si cambiano solo mentre una finestra del programma ha lo
        // stato attivo, e GTK non lo riporta in modo affidabile: la copia si
        // mette subito e, per sicurezza, si rimette alla prossima attivazione
        // di una finestra di Phonestra. Se nel frattempo sul PC si copia altro,
        // quella in attesa si dimentica.
        if let Some(schermo) = gtk::gdk::Display::default() {
            let appunti = schermo.clipboard();
            let in_attesa: Rc<RefCell<Option<String>>> = Rc::default();
            let metti = {
                let appunti = appunti.clone();
                move |testo: &str| {
                    appunti.set_text(testo);
                    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
                        eprintln!("[phonestra] appunti del PC impostati ({} byte, locali: {})", testo.len(), appunti.is_local());
                    }
                }
            };
            {
                let (in_attesa, metti) = (in_attesa.clone(), metti.clone());
                let mut copie = collegamento.appunti();
                gtk::glib::spawn_future_local(async move {
                    while copie.changed().await.is_ok() {
                        let testo = copie.borrow_and_update().1.clone();
                        metti(&testo);
                        *in_attesa.borrow_mut() = Some(testo);
                    }
                });
            }
            {
                // Copia fatta sul PC da un altro programma: la nostra non serve più.
                let in_attesa = in_attesa.clone();
                appunti.connect_changed(move |a| {
                    if !a.is_local() {
                        in_attesa.borrow_mut().take();
                    }
                });
            }
            let quando_attiva = move |w: &gtk::Window| {
                if w.is_active()
                    && let Some(testo) = in_attesa.borrow_mut().take()
                    && !appunti.is_local()
                {
                    metti(&testo);
                }
            };
            app.connect_window_added(move |_, w| {
                let q = quando_attiva.clone();
                w.connect_is_active_notify(move |w| q(w));
            });
        }
        let drawer = cassetto::apri(app, collegamento.clone());
        if let Some(p) = &pacchetto {
            finestra::apri(app, collegamento.clone(), p, p);
        }
        *stato.borrow_mut() = Some((collegamento.clone(), drawer.downgrade()));

        // Ultima finestra chiusa: si rimette a posto il telefono, poi si esce.
        let tieni = Rc::new(RefCell::new(Some(app.hold())));
        app.connect_window_removed(move |app, _| {
            if std::env::var_os("PHONESTRA_DEBUG").is_some() {
                let titoli: Vec<_> = app.windows().iter().map(|w| w.title().unwrap_or_default().to_string()).collect();
                eprintln!("[phonestra] finestra chiusa, restano: {titoli:?}");
            }
            if !app.windows().is_empty() || tieni.borrow().is_none() {
                return;
            }
            collegamento.chiudi();
            let mut s = collegamento.stato();
            let tieni = tieni.clone();
            gtk::glib::spawn_future_local(async move {
                let _ = s.wait_for(|s| *s == Stato::Chiuso).await;
                tieni.borrow_mut().take();
            });
        });
    });
    // Ctrl+C o SIGTERM: si chiudono le finestre come farebbe l'utente, così
    // il telefono viene rimesso com'era prima di uscire.
    {
        let segnale = Arc::new(tokio::sync::Notify::new());
        let n = segnale.clone();
        esecutore().spawn(async move {
            use tokio::signal::unix::{SignalKind, signal};
            let (Ok(mut termina), Ok(mut interrompi)) = (signal(SignalKind::terminate()), signal(SignalKind::interrupt())) else {
                return;
            };
            loop {
                tokio::select! {
                    _ = termina.recv() => {}
                    _ = interrompi.recv() => {}
                }
                n.notify_one();
            }
        });
        let app = app.downgrade();
        gtk::glib::spawn_future_local(async move {
            loop {
                segnale.notified().await;
                let Some(app) = app.upgrade() else { break };
                for w in app.windows() {
                    w.close();
                }
            }
        });
    }
    let uscita = app.run_with_args::<&str>(&[]);
    // Cambio di telefono (o telefono dimenticato): si riparte da capo, col
    // telefono ora in cima alla configurazione.
    if cassetto::RIAVVIA.load(std::sync::atomic::Ordering::SeqCst) {
        let programma = std::env::var_os("APPIMAGE").map(std::path::PathBuf::from).or_else(|| std::env::current_exe().ok());
        if let Some(p) = programma
            && let Err(e) = std::process::Command::new(&p).spawn()
        {
            eprintln!("riavvio non riuscito ({}): {e}", p.display());
        }
    }
    uscita
}
