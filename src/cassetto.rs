// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Il drawer (SPECIFICATION §7.2, §8, §15; mockup `mockup/proposals/drawer-glass*`),
//! stile «vetro»:
//!
//! - barra del titolo: «Phonestra» e la pillola del telefono (stato del
//!   collegamento; il clic apre il menu del telefono);
//! - barra laterale: pagine App e Notifiche, sezione Telefoni;
//! - al centro la pagina scelta: ricerca, Preferiti e Tutte le app, oppure le
//!   notifiche raggruppate per app (clic → apre l'app; «Nascondi» agisce solo
//!   in Phonestra, sul telefono restano);
//! - a destra il telefono disegnato: ora, Wi-Fi, batteria, carica; quando il
//!   collegamento cade, «Riconnetti ora».

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use adw::prelude::*;

use crate::app::{self, App};
use crate::collegamento::{Collegamento, Stato};
use crate::notifiche::{Info, Notifica};
use crate::configurazione::{Preferenze, Telefoni};
use crate::adb::sync;
use crate::{azioni, esecutore, finestra, t};

/// Lato delle icone nella griglia, in punti.
const LATO_ICONA: i32 = 52;
/// Lato delle icone nell'elenco delle notifiche, in punti.
const LATO_ICONA_NOTIFICA: i32 = 36;
/// Notifiche mostrate per app prima di «altre N».
const NOTIFICHE_PER_APP: usize = 2;

/// Alla chiusura Phonestra si riavvia (cambio di telefono): lo guarda `main`.
pub static RIAVVIA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

struct Cassetto {
    app: adw::Application,
    collegamento: Arc<Collegamento>,
    griglia: gtk::FlowBox,
    pagine_app: gtk::Stack,
    attesa: adw::StatusPage,
    ricerca: gtk::SearchEntry,
    /// Riquadri della griglia con il nome in minuscolo, per la ricerca.
    riquadri: RefCell<Vec<(gtk::FlowBoxChild, String, App)>>,
    /// L'ultimo elenco delle app è arrivato intero (altrimenti si rilegge al ricollegamento).
    elenco_intero: std::cell::Cell<bool>,
    griglia_preferiti: gtk::FlowBox,
    scheda_preferiti: gtk::Box,
    titolo_tutte: gtk::Label,
    /// Pacchetti preferiti, nell'ordine scelto (salvati in `telefoni.toml`).
    preferiti: RefCell<Vec<String>>,
    riquadri_preferiti: RefCell<Vec<(gtk::FlowBoxChild, App)>>,
    /// Pallini «app aperta» sotto le icone, per pacchetto.
    pallini: RefCell<Vec<(String, gtk::Widget)>>,
    pallini_preferiti: RefCell<Vec<(String, gtk::Widget)>>,
    /// App del launcher per pacchetto: icone e nomi anche per le notifiche.
    per_pacchetto: RefCell<HashMap<String, App>>,
    /// Finestre aperte, per portarle in primo piano invece di riaprirle.
    finestre: RefCell<HashMap<String, gtk::glib::WeakRef<adw::ApplicationWindow>>>,
    colonna_notifiche: gtk::Box,
    pagine_notifiche: gtk::Stack,
    conta_notifiche: gtk::Label,
    nascondi_tutte: gtk::Button,
    /// Notifiche nascoste in Phonestra: chiave → momento. Se la notifica si
    /// aggiorna (momento diverso) ricompare.
    nascoste: RefCell<HashMap<String, u64>>,
    /// App di cui si vedono tutte le notifiche (non solo le ultime).
    espanse: RefCell<HashSet<String>>,
    finestra: gtk::glib::WeakRef<adw::ApplicationWindow>,
    avvisi: adw::ToastOverlay,
    /// App installate dall'utente: solo queste si possono disinstallare.
    app_utente: RefCell<HashSet<String>>,
    telefono: Telefono,
    /// Etichette col nome del telefono (cambiano con «Rinomina»).
    nomi: Vec<gtk::Label>,
    nome: RefCell<String>,
    /// Un trasferimento (invio o installazione) alla volta.
    occupato: std::cell::Cell<bool>,
    annullato: Arc<AtomicBool>,
    /// Lo schermo vero nel telefono a destra (se il video è disponibile).
    vero: Option<finestra::Vista>,
    /// Avvisi a comparsa del sistema (se il servizio c'è).
    avvisi_sistema: RefCell<Option<crate::avvisi::Avvisi>>,
    /// Notifiche già viste (chiave → momento): solo le nuove fanno un avviso.
    /// `None` fino alla prima lettura, che non avvisa di quelle già presenti.
    viste: RefCell<Option<HashMap<String, u64>>>,
    /// Gli altri telefoni configurati, nella barra laterale.
    altri_telefoni: gtk::Box,
}

/// Colori dello stile «vetro»: tema chiaro (quello dei mockup) e scuro.
struct Tavolozza {
    sfondo: &'static str,
    testo: &'static str,
    tenue: &'static str,
    scheda: &'static str,
    bordo: &'static str,
    riga: &'static str,
    sopra: &'static str,
}

const CHIARO: Tavolozza = Tavolozza {
    sfondo: "linear-gradient(135deg, #cfe6fb 0%, #e3ebfb 45%, #dcd6f7 100%)",
    testo: "rgba(0,0,0,0.82)",
    tenue: "rgba(0,0,0,0.55)",
    scheda: "rgba(255,255,255,0.62)",
    bordo: "rgba(255,255,255,0.75)",
    riga: "rgba(255,255,255,0.7)",
    sopra: "rgba(255,255,255,0.6)",
};

const SCURO: Tavolozza = Tavolozza {
    sfondo: "linear-gradient(135deg, #1c2634 0%, #20222e 50%, #2a2439 100%)",
    testo: "rgba(255,255,255,0.9)",
    tenue: "rgba(255,255,255,0.6)",
    scheda: "rgba(255,255,255,0.06)",
    bordo: "rgba(255,255,255,0.08)",
    riga: "rgba(255,255,255,0.07)",
    sopra: "rgba(255,255,255,0.1)",
};

pub fn apri(app: &adw::Application, collegamento: Arc<Collegamento>) -> adw::ApplicationWindow {
    stile();

    // Pagina «App»: ricerca, Preferiti e Tutte le app in due schede.
    let ricerca = gtk::SearchEntry::builder()
        .placeholder_text(t!("Cerca un'app"))
        .hexpand(true)
        .css_classes(["ricerca"])
        .build();
    let griglia = nuova_griglia();
    let griglia_preferiti = nuova_griglia();
    let scheda_preferiti = scheda(t!("Preferiti"), &griglia_preferiti);
    scheda_preferiti.set_visible(false);
    let titolo_tutte = titolo_sezione(t!("Tutte le app"));
    let scheda_tutte = gtk::Box::builder().orientation(gtk::Orientation::Vertical).css_classes(["scheda"]).build();
    scheda_tutte.append(&titolo_tutte);
    scheda_tutte.append(&griglia);
    let colonna_app = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(14).build();
    colonna_app.append(&scheda_preferiti);
    colonna_app.append(&scheda_tutte);
    let scorri = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&colonna_app)
        .vexpand(true)
        .build();
    let attesa = adw::StatusPage::builder()
        .icon_name("phone-symbolic")
        .title(t!("Collegamento a «{}»…", collegamento.nome))
        .description(t!("Il telefono deve essere acceso, sbloccato e sulla stessa rete Wi-Fi."))
        .vexpand(true)
        .build();
    let nessuna_app = adw::StatusPage::builder().icon_name("system-search-symbolic").title(t!("Nessuna app trovata")).vexpand(true).build();
    let pagine_app = gtk::Stack::new();
    pagine_app.add_named(&attesa, Some("attesa"));
    pagine_app.add_named(&scorri, Some("app"));
    pagine_app.add_named(&nessuna_app, Some("vuoto"));
    let pagina_app = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(14).build();
    pagina_app.append(&ricerca);
    pagina_app.append(&pagine_app);

    // Pagina «Notifiche»: titolo, «Nascondi tutte», schede per app.
    let titolo_notifiche = gtk::Label::builder().label(t!("Notifiche")).xalign(0.0).hexpand(true).css_classes(["titolo-pagina"]).build();
    let nascondi_tutte = gtk::Button::builder().label(t!("Nascondi tutte")).css_classes(["pulsante-vetro"]).build();
    let riga_titolo = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    riga_titolo.append(&titolo_notifiche);
    riga_titolo.append(&nascondi_tutte);
    let colonna_notifiche = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(14).build();
    let scorri_notifiche = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&colonna_notifiche)
        .vexpand(true)
        .build();
    let nessuna_notifica = adw::StatusPage::builder()
        .icon_name("preferences-system-notifications-symbolic")
        .title(t!("Nessuna notifica"))
        .description(t!("Le nuove notifiche di «{}» compariranno qui.", collegamento.nome))
        .vexpand(true)
        .build();
    let pagine_notifiche = gtk::Stack::new();
    pagine_notifiche.add_named(&nessuna_notifica, Some("vuoto"));
    pagine_notifiche.add_named(&scorri_notifiche, Some("elenco"));
    let pagina_notifiche = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(14).build();
    pagina_notifiche.append(&riga_titolo);
    pagina_notifiche.append(&pagine_notifiche);

    let pagine = gtk::Stack::builder().hexpand(true).transition_type(gtk::StackTransitionType::Crossfade).build();
    pagine.add_named(&pagina_app, Some("app"));
    pagine.add_named(&pagina_notifiche, Some("notifiche"));

    // Barra laterale.
    let voce_app = voce_laterale("view-app-grid-symbolic", t!("App"), None);
    let conta_notifiche = gtk::Label::builder().valign(gtk::Align::Center).css_classes(["conta"]).visible(false).build();
    let voce_notifiche = voce_laterale("preferences-system-notifications-symbolic", t!("Notifiche"), Some(&conta_notifiche));
    voce_notifiche.set_group(Some(&voce_app));
    voce_app.set_active(true);
    let punto_laterale = gtk::Box::builder().valign(gtk::Align::Center).css_classes(["punto", "grigio"]).build();
    let stato_laterale = gtk::Label::builder().ellipsize(gtk::pango::EllipsizeMode::End).css_classes(["stato-telefono"]).build();
    let riga_telefono = gtk::Box::builder().spacing(12).css_classes(["riga-telefono"]).build();
    riga_telefono.append(&punto_laterale);
    let nome_laterale = gtk::Label::builder().label(&collegamento.nome).xalign(0.0).hexpand(true).ellipsize(gtk::pango::EllipsizeMode::End).build();
    riga_telefono.append(&nome_laterale);
    riga_telefono.append(&stato_laterale);
    let laterale = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).width_request(196).hexpand(false).css_classes(["laterale"]).build();
    laterale.append(&voce_app);
    laterale.append(&voce_notifiche);
    laterale.append(&titolo_sezione(t!("I miei telefoni")));
    laterale.append(&riga_telefono);
    let altri_telefoni = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).build();
    laterale.append(&altri_telefoni);
    let aggiungi = voce_azione("list-add-symbolic", t!("Aggiungi telefono"));
    aggiungi.add_css_class("aggiungi");
    laterale.append(&aggiungi);
    laterale.append(&titolo_sezione(t!("Strumenti")));
    let installa = voce_azione("folder-download-symbolic", t!("Installa app…"));
    let invia = voce_azione("document-send-symbolic", t!("Invia file…"));
    let ricevi = voce_azione("document-save-symbolic", t!("Ricevi file…"));
    laterale.append(&installa);
    laterale.append(&invia);
    laterale.append(&ricevi);
    laterale.append(&gtk::Box::builder().vexpand(true).build());
    let voce_preferenze = voce_laterale("preferences-system-symbolic", t!("Preferenze"), None);
    voce_preferenze.set_group(Some(&voce_app));
    laterale.append(&voce_preferenze);
    let informazioni = voce_azione("help-about-symbolic", t!("Informazioni"));
    laterale.append(&informazioni);

    // Il telefono: lo schermo vero, e il disegno come riserva.
    let vero = match finestra::vista(collegamento.clone(), finestra::SCHERMO, None) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("[cassetto] schermo del telefono non disponibile: {e}");
            None
        }
    };
    let (telefono, parti) = telefono_disegnato(vero.as_ref().map(|v| &v.contenuto));

    let corpo = gtk::Box::builder().spacing(20).margin_start(20).margin_end(20).margin_bottom(20).margin_top(6).build();
    corpo.append(&laterale);
    corpo.append(&pagine);
    corpo.append(&telefono);

    // Barra del titolo: «Phonestra» e la pillola del telefono.
    let configurato = Telefoni::carica().ok().and_then(|t| t.elenco.into_iter().find(|t| t.seriale == collegamento.seriale));
    let (pillola, parti_pillola) = pillola(&collegamento, configurato.as_ref());
    let barra = adw::HeaderBar::new();
    barra.set_title_widget(Some(&pillola));
    // Simbolo del logo (logos/, senza scritta) accanto al nome, alto quanto
    // il testo: il PNG da 48 px resta nitido anche sugli schermi a densità doppia.
    let nome = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let simbolo = gtk::Image::from_paintable(Some(&gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from_static(include_bytes!(
        "../logos/icons/phonestra-48.png"
    )))
    .expect("icona di Phonestra")));
    simbolo.set_pixel_size(22);
    nome.append(&simbolo);
    nome.append(&gtk::Label::builder().label("Phonestra").css_classes(["titolo-app"]).build());
    barra.pack_start(&nome);

    let vista = adw::ToolbarView::new();
    vista.add_top_bar(&barra);
    let avvisi = adw::ToastOverlay::new();
    avvisi.set_child(Some(&corpo));
    vista.set_content(Some(&avvisi));
    let finestra = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(1040)
        .default_height(680)
        .title("Phonestra")
        .content(&vista)
        .css_classes(["phonestra-drawer"])
        .build();
    segui_tema(&finestra);
    // Scrivere nella finestra va direttamente nella ricerca.
    ricerca.set_key_capture_widget(Some(&finestra));

    let cassetto = Rc::new(Cassetto {
        app: app.clone(),
        collegamento: collegamento.clone(),
        griglia,
        pagine_app,
        attesa,
        ricerca: ricerca.clone(),
        riquadri: RefCell::new(Vec::new()),
        elenco_intero: std::cell::Cell::new(false),
        griglia_preferiti,
        scheda_preferiti,
        titolo_tutte,
        preferiti: RefCell::new(configurato.as_ref().map(|t| t.preferiti.clone()).unwrap_or_default()),
        riquadri_preferiti: RefCell::new(Vec::new()),
        pallini: RefCell::new(Vec::new()),
        pallini_preferiti: RefCell::new(Vec::new()),
        per_pacchetto: RefCell::new(HashMap::new()),
        finestre: RefCell::new(HashMap::new()),
        colonna_notifiche,
        pagine_notifiche,
        conta_notifiche,
        nascondi_tutte: nascondi_tutte.clone(),
        nascoste: RefCell::new(HashMap::new()),
        espanse: RefCell::new(HashSet::new()),
        finestra: finestra.downgrade(),
        avvisi,
        app_utente: RefCell::new(HashSet::new()),
        telefono: parti.clone(),
        nomi: vec![nome_laterale, parti_pillola.nome.clone(), parti_pillola.titolo.clone()],
        nome: RefCell::new(collegamento.nome.clone()),
        occupato: std::cell::Cell::new(false),
        annullato: Arc::new(AtomicBool::new(false)),
        vero,
        avvisi_sistema: RefCell::new(None),
        viste: RefCell::new(None),
        altri_telefoni,
    });
    cassetto.mostra_altri_telefoni();
    if let Some(v) = &cassetto.vero {
        // Col drawer si chiude anche lo schermo vero.
        let c = Rc::downgrade(&cassetto);
        finestra.connect_close_request(move |_| {
            if let Some(v) = c.upgrade().as_ref().and_then(|c| c.vero.as_ref()) {
                v.chiudi();
            }
            gtk::glib::Propagation::Proceed
        });
        // Dopo un clic sullo schermo vero la tastiera va al telefono, non alla
        // ricerca delle app; uscendo torna alla ricerca.
        let fuoco = gtk::EventControllerFocus::new();
        let r = ricerca.clone();
        fuoco.connect_enter(move |_| r.set_key_capture_widget(None::<&gtk::Widget>));
        let (r, f, p) = (ricerca.clone(), finestra.downgrade(), pagine.clone());
        fuoco.connect_leave(move |_| {
            if p.visible_child_name().as_deref() == Some("app") {
                r.set_key_capture_widget(f.upgrade().as_ref());
            }
        });
        v.immagine.add_controller(fuoco);
    }
    {
        let c = cassetto.clone();
        aggiungi.connect_clicked(move |_| {
            let c2 = Rc::downgrade(&c);
            crate::prepara::apri(&c.app, move |t| {
                if let Some(c) = c2.upgrade() {
                    c.mostra_altri_telefoni();
                    c.avviso(&t!("«{}» aggiunto: lo trovi in «I miei telefoni»", t.nome_mostrato()));
                }
            });
        });
    }
    {
        let c = Rc::downgrade(&cassetto);
        *cassetto.avvisi_sistema.borrow_mut() = crate::avvisi::Avvisi::nuovi(move |pacchetto| {
            if let Some(c) = c.upgrade() {
                let app = c.per_pacchetto.borrow().get(&pacchetto).cloned();
                if let Some(a) = app {
                    c.apri_app(&a);
                }
            }
        });
    }
    pagine.add_named(&cassetto.pagina_preferenze(), Some("preferenze"));
    {
        let c = cassetto.clone();
        installa.connect_clicked(move |_| c.scegli_file(true));
        let c = cassetto.clone();
        invia.connect_clicked(move |_| c.scegli_file(false));
        let c = cassetto.clone();
        ricevi.connect_clicked(move |_| c.scegli_dal_telefono());
        let c = cassetto.clone();
        informazioni.connect_clicked(move |_| c.informazioni());
    }
    {
        let (c, p) = (cassetto.clone(), pillola.clone());
        parti_pillola.rinomina.connect_clicked(move |_| {
            p.popdown();
            c.rinomina();
        });
        let (c, p) = (cassetto.clone(), pillola.clone());
        parti_pillola.dimentica.connect_clicked(move |_| {
            p.popdown();
            c.dimentica();
        });
    }
    {
        let a = cassetto.annullato.clone();
        parti.annulla.connect_clicked(move |_| a.store(true, Ordering::SeqCst));
    }
    {
        // File trascinati sul telefono disegnato: .apk da installare, il resto
        // da inviare ai Download.
        let bersaglio = gtk::DropTarget::new(gtk::gdk::FileList::static_type(), gtk::gdk::DragAction::COPY);
        let c = cassetto.clone();
        bersaglio.connect_drop(move |_, valore, _, _| {
            c.telefono.cornice.remove_css_class("sopra");
            c.telefono.rilascio.set_visible(false);
            match valore.get::<gtk::gdk::FileList>() {
                Ok(elenco) => {
                    c.trasferisci(elenco.files());
                    true
                }
                Err(_) => false,
            }
        });
        let t = parti.clone();
        bersaglio.connect_enter(move |_, _, _| {
            t.cornice.add_css_class("sopra");
            t.rilascio.set_visible(true);
            gtk::gdk::DragAction::COPY
        });
        let t = parti.clone();
        bersaglio.connect_leave(move |_| {
            t.cornice.remove_css_class("sopra");
            t.rilascio.set_visible(false);
        });
        telefono.add_controller(bersaglio);
    }

    // Barra laterale → pagina; la ricerca cattura i tasti solo sulle app.
    for (voce, pagina) in [(&voce_app, "app"), (&voce_notifiche, "notifiche"), (&voce_preferenze, "preferenze")] {
        let (pagine, ricerca, f) = (pagine.clone(), ricerca.clone(), finestra.downgrade());
        voce.connect_toggled(move |v| {
            if !v.is_active() {
                return;
            }
            pagine.set_visible_child_name(pagina);
            let finestra = f.upgrade();
            ricerca.set_key_capture_widget(if pagina == "app" { finestra.as_ref() } else { None });
        });
    }
    {
        let c = cassetto.clone();
        ricerca.connect_search_changed(move |_| c.filtra());
    }
    {
        // Invio apre la prima app trovata.
        let c = cassetto.clone();
        ricerca.connect_activate(move |_| {
            let prima = c.riquadri.borrow().iter().find(|(r, _, _)| r.is_visible()).map(|(_, _, a)| a.clone());
            if let Some(a) = prima {
                c.apri_app(&a);
            }
        });
    }
    {
        let c = cassetto.clone();
        cassetto.griglia.connect_child_activated(move |_, riquadro| {
            let scelta = c.riquadri.borrow().iter().find(|(r, _, _)| r == riquadro).map(|(_, _, a)| a.clone());
            if let Some(a) = scelta {
                c.apri_app(&a);
            }
        });
    }
    {
        let c = cassetto.clone();
        cassetto.griglia_preferiti.connect_child_activated(move |_, riquadro| {
            let scelta = c.riquadri_preferiti.borrow().iter().find(|(r, _)| r == riquadro).map(|(_, a)| a.clone());
            if let Some(a) = scelta {
                c.apri_app(&a);
            }
        });
    }
    {
        let c = cassetto.clone();
        nascondi_tutte.connect_clicked(move |_| {
            let tutte = c.collegamento.notifiche().borrow().clone();
            c.nascoste.borrow_mut().extend(tutte.into_iter().map(|n| (n.chiave, n.quando)));
            c.mostra_notifiche();
        });
    }
    for pulsante in [&parti.riconnetti, &parti_pillola.riconnetti] {
        let (collegamento, pillola) = (collegamento.clone(), pillola.clone());
        pulsante.connect_clicked(move |_| {
            pillola.popdown();
            collegamento.riconnetti_ora();
        });
    }
    {
        // L'ora sul telefono disegnato (è quella del PC: sono la stessa).
        let ora = parti.ora.clone();
        let aggiorna = move || {
            let testo = gtk::glib::DateTime::now_local().and_then(|t| t.format("%H:%M")).map(|s| s.to_string()).unwrap_or_default();
            ora.set_label(&testo);
        };
        aggiorna();
        gtk::glib::timeout_add_seconds_local(10, move || {
            aggiorna();
            gtk::glib::ControlFlow::Continue
        });
    }

    // Stato del collegamento e del componente sul telefono → pillola, riga
    // del telefono, telefono disegnato; al primo collegamento si caricano le app.
    let aggiorna_stato = {
        let (pagine_app, scorri) = (cassetto.pagine_app.clone(), scorri.clone());
        let (parti, parti_pillola) = (parti.clone(), parti_pillola.clone());
        let (punto_laterale, stato_laterale) = (punto_laterale.clone(), stato_laterale.clone());
        move |s: Stato, guasto: Option<String>| {
            // Collegato, ma il componente sul telefono non parte: le app non si
            // aprono e l'utente deve saperlo.
            let guasto = guasto.filter(|_| s == Stato::Collegato);
            let (colore, testo) = match s {
                Stato::Cerco => ("grigio", t!("collegamento…")),
                Stato::Collegato if guasto.is_some() => ("rosso", t!("Phonestra non parte sul telefono")),
                Stato::Collegato => ("verde", t!("collegato via Wi-Fi")),
                Stato::Bloccato => ("arancione", t!("bloccato: sbloccalo")),
                Stato::Perso => ("arancione", t!("riconnessione…")),
                Stato::Chiuso => ("grigio", t!("chiuso")),
            };
            for punto in [&parti_pillola.punto, &punto_laterale] {
                for c in ["grigio", "verde", "arancione", "rosso"] {
                    punto.remove_css_class(c);
                }
                punto.add_css_class(colore);
            }
            parti_pillola.stato.set_label(&format!("· {testo}"));
            // Nella barra laterale scritte brevi: la spiegazione è nella pillola.
            stato_laterale.set_label(match s {
                Stato::Cerco => t!("collegamento…"),
                Stato::Collegato if guasto.is_some() => t!("non parte"),
                Stato::Collegato => t!("attivo"),
                Stato::Bloccato => t!("bloccato"),
                Stato::Perso => t!("riconnessione…"),
                Stato::Chiuso => t!("chiuso"),
            });
            // Il telefono disegnato: velo con spiegazione quando non si può usare.
            let spiega_guasto = guasto.as_deref().map(crate::finestra::testo_guasto);
            let (velo, titolo, spiega) = match s {
                Stato::Cerco => (true, t!("Collegamento…"), t!("Il telefono deve essere acceso, sbloccato e sulla stessa rete Wi-Fi.")),
                Stato::Bloccato => (true, t!("Telefono bloccato"), t!("Sbloccalo per continuare: mi ricollego da solo.")),
                Stato::Perso => (true, t!("Collegamento perso"), t!("Riprovo da solo in sottofondo.\nSe il telefono è bloccato, sbloccalo.")),
                Stato::Collegato if guasto.is_some() => {
                    (true, t!("Phonestra non parte sul telefono"), spiega_guasto.as_deref().unwrap_or_default())
                }
                _ => (false, "", ""),
            };
            parti.velo.set_visible(velo);
            parti.titolo_velo.set_label(titolo);
            parti.spiega_velo.set_label(spiega);
            parti.riconnetti.set_visible(s == Stato::Perso || guasto.is_some());
            parti.dati.set_visible(!velo);
            let usabile = s == Stato::Collegato && guasto.is_none();
            let vero = usabile && parti.pagine.child_by_name("vero").is_some();
            parti.pagine.set_visible_child_name(if vero { "vero" } else { "disegno" });
            parti_pillola.riconnetti.set_visible(!usabile);
            // Senza collegamento (o senza componente) le app non si aprono:
            // griglia attenuata.
            scorri.set_sensitive(usabile);
            if usabile {
                scorri.remove_css_class("attenuato");
            } else {
                scorri.add_css_class("attenuato");
            }
            let _ = &pagine_app;
        }
    };
    {
        let c = cassetto.clone();
        let aggiorna_stato = aggiorna_stato.clone();
        let mut ricevitore = collegamento.stato();
        let mut guasto = collegamento.guasto();
        gtk::glib::spawn_future_local(async move {
            let mut caricate = false;
            loop {
                let s = *ricevitore.borrow_and_update();
                aggiorna_stato(s, guasto.borrow_and_update().clone());
                if s == Stato::Collegato && (!caricate || !c.elenco_intero.get()) {
                    if !caricate && std::env::var_os("PHONESTRA_PROVA_RICEVI").is_some() {
                        // Prove dell'interfaccia: «Ricevi file…» si apre da sola.
                        c.scegli_dal_telefono();
                    }
                    caricate = true;
                    c.clone().carica();
                }

                tokio::select! {
                    r = ricevitore.changed() => if r.is_err() { break },
                    r = guasto.changed() => if r.is_err() { break },
                }
            }
        });
    }
    {
        let mut ricevitore = collegamento.info();
        let (parti, parti_pillola) = (parti.clone(), parti_pillola.clone());
        gtk::glib::spawn_future_local(async move {
            loop {
                let info: Info = ricevitore.borrow_and_update().clone();
                match info.batteria {
                    Some(livello) => {
                        let decina = (livello.min(100) / 10) * 10;
                        let icona = match (info.in_carica, decina) {
                            (true, 100) => "battery-level-100-charged-symbolic".to_string(),
                            (true, d) => format!("battery-level-{d}-charging-symbolic"),
                            (false, d) => format!("battery-level-{d}-symbolic"),
                        };
                        parti.icona_batteria.set_icon_name(Some(&icona));
                        parti.batteria.set_label(&t!("{} %", livello));
                    }
                    None => {
                        parti.icona_batteria.set_icon_name(None);
                        parti.batteria.set_label("");
                    }
                }
                let mut dettagli = Vec::new();
                if let Some(r) = &info.rete {
                    dettagli.push(t!("Wi-Fi «{}»", r));
                }
                if let Some(b) = info.batteria {
                    dettagli.push(if info.in_carica { t!("batteria {} %, in carica", b) } else { t!("batteria {} %", b) });
                }
                parti_pillola.dettagli.set_label(&dettagli.join(" · "));
                if ricevitore.changed().await.is_err() {
                    break;
                }
            }
        });
    }
    {
        let c = cassetto.clone();
        let mut ricevitore = collegamento.notifiche();
        gtk::glib::spawn_future_local(async move {
            loop {
                ricevitore.borrow_and_update();
                c.mostra_notifiche();
                if ricevitore.changed().await.is_err() {
                    break;
                }
            }
        });
    }

    finestra.present();
    crate::foto::prendi(&finestra, "drawer");
    finestra
}

/// Tema del sistema, chiaro o scuro: classe `scuro` sulla finestra.
pub(crate) fn segui_tema(finestra: &adw::ApplicationWindow) {
    let gestore = adw::StyleManager::default();
    let f = finestra.downgrade();
    let segui = move |g: &adw::StyleManager| {
        if let Some(f) = f.upgrade() {
            if g.is_dark() {
                f.add_css_class("scuro");
            } else {
                f.remove_css_class("scuro");
            }
        }
    };
    segui(&gestore);
    gestore.connect_dark_notify(segui);
}

/// Voce della barra laterale: icona, nome e, se c'è, un contatore.
fn voce_laterale(icona: &str, nome: &str, conta: Option<&gtk::Label>) -> gtk::ToggleButton {
    let contenuto = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    contenuto.append(&gtk::Image::from_icon_name(icona));
    contenuto.append(&gtk::Label::builder().label(nome).xalign(0.0).hexpand(true).build());
    if let Some(c) = conta {
        contenuto.append(c);
    }
    gtk::ToggleButton::builder().child(&contenuto).css_classes(["voce-laterale"]).build()
}

/// Il nome del logo di Phonestra (`logos/icons/phonestra-256.png`) nel tema
/// delle icone, per chi vuole un nome e non un'immagine (la finestra
/// «Informazioni»). Phonestra non installa icone nel sistema: il PNG si scrive
/// una volta nella sua cache e quella cartella si aggiunge al tema. Se non si
/// può scrivere, resta l'icona generica del telefono.
fn icona_nel_tema() -> &'static str {
    const NOME: &str = "phonestra";
    static PRONTA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let pronta = *PRONTA.get_or_init(|| {
        let cartella = gtk::glib::user_cache_dir().join("Phonestra").join("icone");
        let file = cartella.join(format!("{NOME}.png"));
        let png: &[u8] = include_bytes!("../logos/icons/phonestra-256.png");
        let scritta = std::fs::read(&file).is_ok_and(|v| v == png)
            || std::fs::create_dir_all(&cartella).and_then(|_| std::fs::write(&file, png)).is_ok();
        if scritta && let Some(schermo) = gtk::gdk::Display::default() {
            gtk::IconTheme::for_display(&schermo).add_search_path(&cartella);
        }
        scritta
    });
    if pronta { NOME } else { "phone-symbolic" }
}

/// Voce della barra laterale che fa un'azione (non cambia pagina).
fn voce_azione(icona: &str, nome: &str) -> gtk::Button {
    let contenuto = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    contenuto.append(&gtk::Image::from_icon_name(icona));
    contenuto.append(&gtk::Label::builder().label(nome).xalign(0.0).hexpand(true).build());
    gtk::Button::builder().child(&contenuto).css_classes(["voce-laterale"]).build()
}

fn titolo_sezione(testo: &str) -> gtk::Label {
    gtk::Label::builder().label(testo.to_uppercase()).xalign(0.0).css_classes(["titolo-sezione"]).build()
}

/// Scheda bianca semitrasparente con titolo e contenuto.
fn scheda(titolo: &str, contenuto: &impl IsA<gtk::Widget>) -> gtk::Box {
    let s = gtk::Box::builder().orientation(gtk::Orientation::Vertical).css_classes(["scheda"]).build();
    s.append(&titolo_sezione(titolo));
    s.append(contenuto);
    s
}

/// Parti del telefono (schermo vero o disegno di riserva) che cambiano.
#[derive(Clone)]
struct Telefono {
    ora: gtk::Label,
    icona_batteria: gtk::Image,
    batteria: gtk::Label,
    dati: gtk::Box,
    /// «vero» (lo schermo in diretta) o «disegno» (riserva senza collegamento).
    pagine: gtk::Stack,
    velo: gtk::Box,
    titolo_velo: gtk::Label,
    spiega_velo: gtk::Label,
    riconnetti: gtk::Button,
    trasferimento: gtk::Box,
    /// Freccia verso il telefono (invio) o verso il PC (ricezione).
    icona_trasferimento: gtk::Image,
    nome_trasferimento: gtk::Label,
    dettaglio_trasferimento: gtk::Label,
    barra_trasferimento: gtk::ProgressBar,
    annulla: gtk::Button,
    rilascio: gtk::Label,
    cornice: gtk::Box,
}

/// Il telefono a destra: lo schermo vero in diretta (`vero`, SPECIFICATION §7.2)
/// dentro una cornice; senza collegamento un disegno con ora e batteria.
fn telefono_disegnato(vero: Option<&adw::ToastOverlay>) -> (gtk::Box, Telefono) {
    let ora = gtk::Label::builder().xalign(0.0).hexpand(true).css_classes(["ora-telefono"]).build();
    let icona_batteria = gtk::Image::new();
    let batteria = gtk::Label::new(None);
    let dati = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    dati.append(&gtk::Image::from_icon_name("network-wireless-signal-excellent-symbolic"));
    dati.append(&icona_batteria);
    dati.append(&batteria);
    let stato = gtk::Box::builder().css_classes(["stato-telefono-disegnato"]).build();
    stato.append(&ora);
    stato.append(&dati);
    let disegno = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).css_classes(["disegno"]).build();
    disegno.append(&gtk::Box::builder().halign(gtk::Align::Center).css_classes(["foro"]).build());
    disegno.append(&stato);
    disegno.append(&gtk::Box::builder().vexpand(true).build());

    let pagine = gtk::Stack::builder().transition_type(gtk::StackTransitionType::Crossfade).vexpand(true).build();
    pagine.add_named(&disegno, Some("disegno"));
    if let Some(v) = vero {
        pagine.add_named(v, Some("vero"));
    }

    // Trasferimento in corso: nome, avanzamento, «×» per annullare.
    let nome_trasferimento = gtk::Label::builder().xalign(0.0).ellipsize(gtk::pango::EllipsizeMode::Middle).css_classes(["nome-trasferimento"]).build();
    let dettaglio_trasferimento = gtk::Label::builder().xalign(0.0).css_classes(["dettaglio-trasferimento"]).build();
    let annulla = gtk::Button::builder().icon_name("window-close-symbolic").tooltip_text(t!("Annulla")).valign(gtk::Align::Center).css_classes(["flat", "circular", "annulla"]).build();
    let testi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).hexpand(true).build();
    testi.append(&nome_trasferimento);
    testi.append(&dettaglio_trasferimento);
    let riga = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let icona_trasferimento = gtk::Image::from_icon_name("document-send-symbolic");
    riga.append(&icona_trasferimento);
    riga.append(&testi);
    riga.append(&annulla);
    let barra_trasferimento = gtk::ProgressBar::new();
    let trasferimento = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(14)
        .valign(gtk::Align::End)
        .css_classes(["trasferimento"])
        .visible(false)
        .build();
    trasferimento.append(&riga);
    trasferimento.append(&barra_trasferimento);

    // Mentre si trascina un file sopra il telefono.
    let rilascio = gtk::Label::builder()
        .label(t!("Rilascia per inviare al telefono\n(un .apk si installa)"))
        .justify(gtk::Justification::Center)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .css_classes(["rilascio"])
        .visible(false)
        .build();

    let titolo_velo = gtk::Label::builder().css_classes(["titolo-velo"]).wrap(true).justify(gtk::Justification::Center).build();
    let spiega_velo = gtk::Label::builder().wrap(true).justify(gtk::Justification::Center).css_classes(["spiega-velo"]).build();
    let riconnetti = gtk::Button::builder().label(t!("Riconnetti ora")).halign(gtk::Align::Center).css_classes(["pulsante-bianco"]).build();
    let velo = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .valign(gtk::Align::Fill)
        .css_classes(["velo"])
        .visible(false)
        .build();
    let centro = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).valign(gtk::Align::Center).vexpand(true).build();
    centro.append(&titolo_velo);
    centro.append(&spiega_velo);
    centro.append(&riconnetti);
    velo.append(&centro);

    let schermo = gtk::Overlay::builder().child(&pagine).css_classes(["schermo"]).vexpand(true).overflow(gtk::Overflow::Hidden).build();
    schermo.add_overlay(&velo);
    schermo.add_overlay(&trasferimento);
    schermo.add_overlay(&rilascio);
    // Proporzioni di un telefono vero (schermo 9:19,5 più la cornice) e
    // larghezza fissa: lo spazio in più va tutto alla colonna centrale. Se la
    // finestra è bassa il telefono si rimpicciolisce invece di schiacciarsi.
    let cornice = gtk::Box::builder().css_classes(["telefono"]).build();
    cornice.append(&schermo);
    schermo.set_hexpand(true);
    let proporzioni = gtk::AspectFrame::builder().ratio(0.475).obey_child(false).yalign(0.0).vexpand(true).child(&cornice).build();
    let telefono = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).width_request(290).hexpand(false).build();
    telefono.append(&proporzioni);
    proporzioni.set_hexpand(true);
    let parti = Telefono {
        ora,
        icona_batteria,
        batteria,
        dati,
        pagine,
        velo,
        titolo_velo,
        spiega_velo,
        riconnetti,
        trasferimento,
        icona_trasferimento,
        nome_trasferimento,
        dettaglio_trasferimento,
        barra_trasferimento,
        annulla,
        rilascio,
        cornice,
    };
    (telefono, parti)
}

/// Parti della pillola e del suo menu che cambiano.
#[derive(Clone)]
struct Pillola {
    punto: gtk::Box,
    nome: gtk::Label,
    stato: gtk::Label,
    titolo: gtk::Label,
    dettagli: gtk::Label,
    riconnetti: gtk::Button,
    rinomina: gtk::Button,
    dimentica: gtk::Button,
}

/// La pillola del telefono nella barra del titolo e il menu del telefono.
fn pillola(collegamento: &Collegamento, configurato: Option<&crate::configurazione::Telefono>) -> (gtk::MenuButton, Pillola) {
    let punto = gtk::Box::builder().valign(gtk::Align::Center).css_classes(["punto", "grigio"]).build();
    let stato = gtk::Label::builder().css_classes(["tenue"]).build();
    let contenuto = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    contenuto.append(&punto);
    let nome = gtk::Label::builder().label(&collegamento.nome).css_classes(["nome-pillola"]).build();
    contenuto.append(&nome);
    contenuto.append(&stato);

    let titolo = gtk::Label::builder().label(&collegamento.nome).xalign(0.0).css_classes(["heading"]).build();
    let dettagli = gtk::Label::builder().xalign(0.0).css_classes(["caption", "dim-label"]).build();
    let riconnetti = gtk::Button::builder().css_classes(["flat", "voce-menu"]).build();
    let riga = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    riga.append(&gtk::Image::from_icon_name("view-refresh-symbolic"));
    riga.append(&gtk::Label::new(Some(t!("Riconnetti"))));
    riconnetti.set_child(Some(&riga));
    let menu = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).width_request(280).build();
    let testa = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).margin_start(10).margin_end(10).margin_top(6).margin_bottom(8).build();
    testa.append(&titolo);
    if let Some(t) = configurato {
        let modello = format!("{} · Android {}", t.modello, t.android);
        testa.append(&gtk::Label::builder().label(modello).xalign(0.0).css_classes(["caption", "dim-label"]).build());
    }
    testa.append(&dettagli);
    menu.append(&testa);
    menu.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    menu.append(&riconnetti);
    let rinomina = voce_menu("document-edit-symbolic", t!("Rinomina…"), true);
    menu.append(&rinomina);
    let debug_wireless = gtk::Box::builder().spacing(10).css_classes(["riga-menu"]).sensitive(false).tooltip_text(t!("In arrivo")).build();
    debug_wireless.append(&gtk::Image::from_icon_name("security-medium-symbolic"));
    debug_wireless.append(&gtk::Label::builder().label(t!("Spegni il Debug wireless alla chiusura")).xalign(0.0).hexpand(true).build());
    debug_wireless.append(&gtk::Switch::builder().valign(gtk::Align::Center).build());
    menu.append(&debug_wireless);
    menu.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let dimentica = voce_menu("user-trash-symbolic", t!("Dimentica questo telefono…"), true);
    dimentica.add_css_class("voce-rossa");
    menu.append(&dimentica);

    let pulsante = gtk::MenuButton::builder()
        .child(&contenuto)
        .popover(&gtk::Popover::builder().child(&menu).build())
        .css_classes(["pillola"])
        .build();
    (pulsante, Pillola { punto, nome, stato, titolo, dettagli, riconnetti, rinomina, dimentica })
}

/// Voce di un menu: icona e testo; spenta («In arrivo») se non ancora fatta.
fn voce_menu(icona: &str, testo: &str, attiva: bool) -> gtk::Button {
    let riga = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    riga.append(&gtk::Image::from_icon_name(icona));
    riga.append(&gtk::Label::new(Some(testo)));
    let b = gtk::Button::builder().child(&riga).css_classes(["flat"]).sensitive(attiva).build();
    if !attiva {
        b.set_tooltip_text(Some(t!("In arrivo")));
    }
    b
}

/// Aspetto «vetro» del drawer, con colori espliciti per il tema chiaro e per
/// quello scuro: i testi non dipendono da come il tema colora la finestra.
pub(crate) fn stile() {
    thread_local!(static FATTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    if FATTO.with(|f| f.replace(true)) {
        return;
    }
    let css = gtk::CssProvider::new();
    css.load_from_string(&format!("{}\n{}\n{}", css_tavolozza("window.phonestra-drawer", &CHIARO), css_tavolozza("window.phonestra-drawer.scuro", &SCURO), CSS_COMUNE));
    let css_ricevi = gtk::CssProvider::new();
    css_ricevi.load_from_string(crate::ricevi::CSS);
    if let Some(schermo) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&schermo, &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        gtk::style_context_add_provider_for_display(&schermo, &css_ricevi, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

fn css_tavolozza(w: &str, t: &Tavolozza) -> String {
    let Tavolozza { sfondo, testo, tenue, scheda, bordo, riga, sopra } = t;
    format!(
        "{w} {{ background: {sfondo}; color: {testo}; }}
         {w} headerbar {{ background: none; box-shadow: none; color: {testo}; }}
         {w} .titolo-sezione, {w} .tenue, {w} .stato-telefono, {w} .riga-notifica .testo, {w} .riga-notifica .quando {{ color: {tenue}; }}
         {w} .scheda {{ background: {scheda}; box-shadow: inset 0 0 0 1px {bordo}, 0 1px 4px rgba(0,0,0,0.05); }}
         {w} entry.ricerca, {w} searchentry.ricerca {{ background: {riga}; color: {testo}; box-shadow: inset 0 0 0 1px {bordo}, 0 1px 3px rgba(0,0,0,0.06); }}
         {w} menubutton.pillola > button {{ background: {scheda}; color: {testo}; box-shadow: inset 0 0 0 1px {bordo}; }}
         {w} menubutton.pillola > button:hover, {w} menubutton.pillola > button:checked {{ background: {riga}; }}
         {w} button.voce-laterale {{ color: {testo}; }}
         {w} button.voce-laterale:hover {{ background: {sopra}; }}
         {w} flowboxchild.riquadro-app:hover {{ background: {sopra}; }}
         {w} .riga-notifica {{ background: {riga}; }}
         {w} button.pulsante-vetro {{ background: {riga}; color: {testo}; box-shadow: inset 0 0 0 1px {bordo}; }}"
    )
}

const CSS_COMUNE: &str = "
    window.phonestra-app headerbar { background: linear-gradient(135deg, #d8eafb, #e6ecfb 50%, #e2dcf8); color: rgba(0,0,0,0.82); box-shadow: inset 0 -1px rgba(0,0,0,0.06); }
    window.phonestra-app.scuro headerbar { background: linear-gradient(135deg, #1c2634, #20222e 50%, #2a2439); color: rgba(255,255,255,0.9); }
    window.phonestra-app headerbar button { border-radius: 17px; }
    window.phonestra-app button.registra.in-corso { background: white; color: #c01c28; border-radius: 13px; box-shadow: 0 0 0 1px rgba(192,28,40,0.25); font-weight: 700; padding: 0 10px; min-height: 26px; }
    .punto-rec { min-width: 8px; min-height: 8px; border-radius: 4px; background: #e01b24; }
    .velo-app { padding: 30px; }
    .velo-app.chiaro { background: rgba(248,250,253,0.55); color: rgba(0,0,0,0.82); }
    .velo-app.nero { background: #111; color: white; }
    .velo-app .cerchio { min-width: 64px; min-height: 64px; border-radius: 32px; background: rgba(255,255,255,0.85); box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
    .velo-app.nero .cerchio { background: rgba(255,255,255,0.1); box-shadow: none; }
    .titolo-velo-app { font-size: 17px; font-weight: 700; }
    .testo-velo-app { font-size: 13.5px; opacity: 0.75; }
    picture.sfocata { filter: blur(6px) grayscale(40%); opacity: 0.8; }
    window.phonestra-drawer .titolo-app { font-weight: 700; margin-left: 6px; }
    window.phonestra-drawer .titolo-sezione { font-size: 11px; font-weight: 700; letter-spacing: 1px; margin: 16px 12px 4px 12px; }
    window.phonestra-drawer .scheda .titolo-sezione { margin: 0 0 6px 4px; }
    window.phonestra-drawer .titolo-pagina { font-size: 16px; font-weight: 700; margin-left: 4px; }
    window.phonestra-drawer .scheda { border-radius: 18px; padding: 14px 16px 10px 16px; }
    window.phonestra-drawer entry.ricerca, window.phonestra-drawer searchentry.ricerca { border-radius: 20px; min-height: 40px; padding: 0 16px; }
    window.phonestra-drawer menubutton.pillola > button { border-radius: 15px; min-height: 30px; padding: 0 14px; }
    window.phonestra-drawer .nome-pillola { font-weight: 600; }
    window.phonestra-drawer .tenue { font-weight: 500; }
    window.phonestra-drawer .punto { min-width: 8px; min-height: 8px; border-radius: 4px; }
    window.phonestra-drawer .punto.grigio { background: #9a9996; }
    window.phonestra-drawer .punto.verde { background: #26a269; }
    window.phonestra-drawer .punto.arancione { background: #e66100; }
    window.phonestra-drawer .punto.rosso { background: #c01c28; }
    window.phonestra-drawer button.voce-laterale { background: none; box-shadow: none; border-radius: 10px; min-height: 38px; padding: 0 12px; font-weight: 600; }
    window.phonestra-drawer button.voce-laterale:disabled { filter: none; opacity: 0.6; }
    window.phonestra-drawer button.voce-laterale:disabled label, window.phonestra-drawer button.voce-laterale:disabled image { color: inherit; }
    window.phonestra-drawer button.voce-laterale.aggiungi { color: #3584e4; }
    window.phonestra-drawer button.altro-telefono { font-weight: 600; }
    window.phonestra-drawer button.voce-laterale:checked { background: #3584e4; color: white; box-shadow: 0 2px 8px rgba(53,132,228,0.35); }
    window.phonestra-drawer .conta { background: #3584e4; color: white; border-radius: 10px; min-width: 20px; min-height: 20px; font-size: 11px; font-weight: 700; padding: 0 5px; }
    window.phonestra-drawer button.voce-laterale:checked .conta { background: white; color: #3584e4; }
    window.phonestra-drawer .riga-telefono { min-height: 38px; padding: 0 12px; font-weight: 600; }
    window.phonestra-drawer .stato-telefono { font-size: 11.5px; font-weight: 500; }
    window.phonestra-drawer flowbox { background: none; }
    window.phonestra-drawer flowboxchild.riquadro-app { border-radius: 14px; padding: 10px 2px 6px 2px; background: none; }
    window.phonestra-drawer .nome-app { font-size: 12.5px; font-weight: 500; }
    window.phonestra-drawer .pallino { min-width: 6px; min-height: 6px; border-radius: 3px; background: #3584e4; }
    window.phonestra-drawer .attenuato { opacity: 0.45; }
    window.phonestra-drawer .telefono { background: #1e1e22; border-radius: 30px; padding: 7px; box-shadow: 0 8px 22px rgba(0,0,0,0.25); }
    window.phonestra-drawer .schermo { border-radius: 24px; color: white;
        background-image: radial-gradient(circle at 20% 15%, rgba(127,184,255,0.9) 0%, transparent 45%),
                          radial-gradient(circle at 85% 70%, rgba(176,124,255,0.9) 0%, transparent 50%),
                          linear-gradient(160deg, #3d6fd8, #5a3fc0 60%, #1b8fb0); }
    window.phonestra-drawer .foro { min-width: 9px; min-height: 9px; border-radius: 5px; background: black; margin-top: 10px; }
    window.phonestra-drawer .stato-telefono-disegnato { padding: 0 16px; font-size: 11px; font-weight: 600; margin-top: -14px; }
    window.phonestra-drawer .rilascio { font-size: 13px; font-weight: 700; color: white; padding: 14px 16px; border-radius: 16px; background: rgba(20,40,90,0.75); border: 2px dashed rgba(255,255,255,0.7); }
    window.phonestra-drawer .voce-rossa { color: #c01c28; }
    window.phonestra-drawer .riga-menu { padding: 6px 10px; min-height: 30px; }
    window.phonestra-drawer .trasferimento { background: rgba(255,255,255,0.22); border-radius: 16px; padding: 10px; box-shadow: inset 0 0 0 1px rgba(255,255,255,0.25); }
    window.phonestra-drawer .nome-trasferimento { font-size: 12px; font-weight: 700; }
    window.phonestra-drawer .dettaglio-trasferimento { font-size: 10.5px; opacity: 0.85; }
    window.phonestra-drawer .trasferimento progressbar trough { background: rgba(255,255,255,0.3); min-height: 3px; border-radius: 2px; }
    window.phonestra-drawer .trasferimento progressbar progress { background: white; min-height: 3px; border-radius: 2px; }
    window.phonestra-drawer .trasferimento button.annulla { color: white; min-width: 24px; min-height: 24px; padding: 0; }
    window.phonestra-drawer .telefono.sopra { box-shadow: 0 0 0 4px #3584e4, 0 8px 22px rgba(0,0,0,0.25); }
    window.phonestra-drawer .velo { background: rgba(20,20,40,0.55); border-radius: 24px; padding: 20px; }
    window.phonestra-drawer .titolo-velo { font-size: 15px; font-weight: 700; }
    window.phonestra-drawer .spiega-velo { font-size: 12px; }
    window.phonestra-drawer button.pulsante-bianco { background: white; color: #1c3f8f; border-radius: 18px; padding: 8px 18px; font-weight: 700; margin-top: 6px; }
    window.phonestra-drawer button.pulsante-vetro { border-radius: 16px; padding: 6px 14px; font-weight: 600; }
    window.phonestra-drawer .riga-notifica { border-radius: 12px; padding: 12px 14px; }
    window.phonestra-drawer .riga-notifica .titolo { font-weight: 600; }
    window.phonestra-drawer .riga-notifica .testo { font-size: 13px; }
    window.phonestra-drawer .riga-notifica .quando { font-size: 12px; }
    window.phonestra-drawer button.altre { background: none; box-shadow: none; color: #3584e4; font-weight: 600; padding: 4px 14px; }
    window.phonestra-drawer button.riga-cliccabile { background: none; box-shadow: none; padding: 0; border-radius: 12px; }
";

/// Una riga delle Preferenze: titolo, spiegazione, controllo a destra.
fn riga_preferenza(titolo: &str, spiega: &str, controllo: &impl IsA<gtk::Widget>) -> gtk::Box {
    let testi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).hexpand(true).build();
    testi.append(&gtk::Label::builder().label(titolo).xalign(0.0).css_classes(["titolo"]).build());
    testi.append(&gtk::Label::builder().label(spiega).xalign(0.0).wrap(true).css_classes(["testo"]).build());
    let riga = gtk::Box::builder().spacing(14).css_classes(["riga-notifica"]).build();
    riga.append(&testi);
    riga.append(controllo);
    riga
}

/// Testo del pulsante «App che possono avvisare»: «tutte ›» o «tutte tranne N ›».
fn aggiorna_scelta_app(pulsante: &gtk::Button) {
    let n = Preferenze::attuali().app_silenziate.len();
    pulsante.set_label(&if n == 0 { t!("tutte ›").to_string() } else { t!("tutte tranne {} ›", n) });
}

/// Dimensione leggibile: «820 KB», «82 MB», «1,4 GB».
fn misura(byte: usize) -> String {
    let b = byte as f64;
    if b < 1e6 {
        format!("{:.0} KB", b / 1e3)
    } else if b < 1e9 {
        format!("{:.0} MB", b / 1e6)
    } else {
        let gb = format!("{:.1} GB", b / 1e9);
        // Virgola decimale in italiano, punto in inglese.
        if crate::lingua::attuale() == crate::lingua::Lingua::Italiano { gb.replace('.', ",") } else { gb }
    }
}

fn nuova_griglia() -> gtk::FlowBox {
    gtk::FlowBox::builder()
        .homogeneous(true)
        .selection_mode(gtk::SelectionMode::None)
        .activate_on_single_click(true)
        .min_children_per_line(3)
        .max_children_per_line(12)
        .row_spacing(2)
        .column_spacing(2)
        .valign(gtk::Align::Start)
        .build()
}

/// Ora della notifica: «10:14» se è di oggi, altrimenti «26 set». Il mese
/// viene dalle traduzioni di Phonestra, non dalla lingua del sistema: le due
/// possono essere diverse.
fn orario(quando: u64) -> String {
    let Ok(t) = gtk::glib::DateTime::from_unix_local((quando / 1000) as i64) else {
        return String::new();
    };
    let oggi = gtk::glib::DateTime::now_local().ok().map(|o| (o.year(), o.day_of_year()));
    if oggi == Some((t.year(), t.day_of_year())) {
        return t.format("%H:%M").map(|s| s.to_string()).unwrap_or_default();
    }
    let g = t.day_of_month();
    match t.month() {
        1 => t!("{} gen", g),
        2 => t!("{} feb", g),
        3 => t!("{} mar", g),
        4 => t!("{} apr", g),
        5 => t!("{} mag", g),
        6 => t!("{} giu", g),
        7 => t!("{} lug", g),
        8 => t!("{} ago", g),
        9 => t!("{} set", g),
        10 => t!("{} ott", g),
        11 => t!("{} nov", g),
        _ => t!("{} dic", g),
    }
}

fn icona(png: Option<&[u8]>, lato: i32) -> gtk::Image {
    let immagine = gtk::Image::builder().pixel_size(lato).build();
    match png.and_then(|p| gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from(p)).ok()) {
        Some(t) => immagine.set_paintable(Some(&t)),
        None => immagine.set_icon_name(Some("application-x-executable-symbolic")),
    }
    immagine
}

impl Cassetto {
    /// Legge le app dal telefono e riempie la griglia.
    fn carica(self: Rc<Self>) {
        let Some(adb) = self.collegamento.adb().borrow().clone() else {
            return;
        };
        if self.riquadri.borrow().is_empty() {
            self.attesa.set_title(t!("Lettura delle app…"));
            self.pagine_app.set_visible_child_name("attesa");
        }
        let lato = (LATO_ICONA * self.griglia.scale_factor().max(1)) as u32;
        {
            let (c, adb) = (self.clone(), adb.clone());
            gtk::glib::spawn_future_local(async move {
                match esecutore().spawn(async move { azioni::app_utente(&adb).await }).await {
                    Ok(Ok(utente)) => *c.app_utente.borrow_mut() = utente,
                    Ok(Err(e)) => eprintln!("[cassetto] app dell'utente non lette: {e:#}"),
                    Err(e) => eprintln!("[cassetto] app dell'utente non lette: {e}"),
                }
            });
        }
        gtk::glib::spawn_future_local(async move {
            match esecutore().spawn(async move { app::elenco(&adb, lato).await }).await {
                Ok(Ok(elenco)) => self.riempi(elenco),
                Ok(Err(e)) => self.errore(&format!("{e:#}")),
                Err(e) => self.errore(&e.to_string()),
            }
        });
    }

    fn errore(&self, testo: &str) {
        eprintln!("[cassetto] {testo}");
        self.elenco_intero.set(false);
        if self.riquadri.borrow().is_empty() {
            self.attesa.set_title(t!("App non lette"));
            self.attesa.set_description(Some(testo));
        }
    }

    fn riempi(self: &Rc<Self>, elenco: Vec<App>) {
        self.elenco_intero.set(true);
        self.griglia.remove_all();
        let mut riquadri = Vec::with_capacity(elenco.len());
        let mut pallini = Vec::with_capacity(elenco.len());
        let mut per_pacchetto = HashMap::new();
        for a in elenco {
            let (riquadro, pallino) = self.riquadro(&a);
            self.griglia.append(&riquadro);
            per_pacchetto.entry(a.pacchetto.clone()).or_insert_with(|| a.clone());
            pallini.push((a.pacchetto.clone(), pallino));
            riquadri.push((riquadro, a.nome.to_lowercase(), a));
        }
        *self.riquadri.borrow_mut() = riquadri;
        *self.pallini.borrow_mut() = pallini;
        *self.per_pacchetto.borrow_mut() = per_pacchetto;
        self.mostra_preferiti();
        self.filtra();
        self.aggiorna_pallini();
        // Ora le notifiche possono mostrare icone e nomi delle app.
        self.mostra_notifiche();
    }

    /// Riquadro di un'app: icona, nome e pallino «aperta»; clic destro per il
    /// menu dell'app.
    fn riquadro(self: &Rc<Self>, a: &App) -> (gtk::FlowBoxChild, gtk::Widget) {
        let nome = gtk::Label::builder()
            .label(&a.nome)
            .max_width_chars(10)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .lines(2)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .justify(gtk::Justification::Center)
            .valign(gtk::Align::Start)
            .css_classes(["nome-app"])
            .build();
        let pallino = gtk::Box::builder().halign(gtk::Align::Center).css_classes(["pallino"]).visible(false).build();
        let contenuto = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
        contenuto.append(&icona(Some(&a.icona), LATO_ICONA));
        contenuto.append(&nome);
        contenuto.append(&pallino);
        let riquadro = gtk::FlowBoxChild::builder()
            .child(&contenuto)
            .tooltip_text(&a.nome)
            .width_request(88)
            .halign(gtk::Align::Center)
            .css_classes(["riquadro-app"])
            .build();
        let clic_destro = gtk::GestureClick::new();
        clic_destro.set_button(gtk::gdk::BUTTON_SECONDARY);
        {
            let (c, a, r) = (self.clone(), a.clone(), riquadro.downgrade());
            clic_destro.connect_pressed(move |_, _, _, _| {
                if let Some(riquadro) = r.upgrade() {
                    c.menu_app(&riquadro, &a);
                }
            });
        }
        riquadro.add_controller(clic_destro);
        (riquadro, pallino.upcast())
    }

    /// Menu dell'app (clic destro): apri o porta in primo piano, preferiti,
    /// chiudi se aperta.
    fn menu_app(self: &Rc<Self>, riquadro: &gtk::FlowBoxChild, a: &App) {
        let aperta = self.finestra_di(&a.pacchetto);
        let e_preferito = self.preferiti.borrow().contains(&a.pacchetto);
        let colonna = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).width_request(220).build();
        let testa = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).margin_start(10).margin_end(10).margin_top(6).margin_bottom(8).build();
        testa.append(&gtk::Label::builder().label(&a.nome).xalign(0.0).css_classes(["heading"]).build());
        if aperta.is_some() {
            testa.append(&gtk::Label::builder().label(t!("aperta in una finestra")).xalign(0.0).css_classes(["caption", "dim-label"]).build());
        }
        colonna.append(&testa);
        colonna.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let menu = gtk::Popover::builder().child(&colonna).build();
        let voce = |icona: &str, testo: &str| {
            let riga = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            riga.append(&gtk::Image::from_icon_name(icona));
            riga.append(&gtk::Label::new(Some(testo)));
            let b = gtk::Button::builder().child(&riga).css_classes(["flat"]).build();
            colonna.append(&b);
            b
        };
        let apri = voce(
            if aperta.is_some() { "go-next-symbolic" } else { "media-playback-start-symbolic" },
            if aperta.is_some() { t!("Porta in primo piano") } else { t!("Apri") },
        );
        let preferito = voce(
            if e_preferito { "starred-symbolic" } else { "non-starred-symbolic" },
            if e_preferito { t!("Togli dai preferiti") } else { t!("Aggiungi ai preferiti") },
        );
        let informazioni = voce("help-about-symbolic", t!("Informazioni sull'app"));
        colonna.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let chiudi = aperta.as_ref().map(|_| voce("window-close-symbolic", t!("Chiudi app")));
        let di_sistema = !self.app_utente.borrow().contains(&a.pacchetto);
        let disinstalla = voce_menu("user-trash-symbolic", t!("Disinstalla…"), !di_sistema);
        if di_sistema {
            disinstalla.set_tooltip_text(Some(t!("App di sistema: non si può disinstallare")));
        }
        disinstalla.add_css_class("voce-rossa");
        colonna.append(&disinstalla);
        {
            let (c, a, m) = (self.clone(), a.clone(), menu.downgrade());
            informazioni.connect_clicked(move |_| {
                if let Some(m) = m.upgrade() {
                    m.popdown();
                }
                finestra::apri(
                    &c.app,
                    c.collegamento.clone(),
                    &format!("{}{}", finestra::INFORMAZIONI, a.pacchetto),
                    &t!("Informazioni · {}", a.nome),
                );
            });
        }
        {
            let (c, a, m) = (self.clone(), a.clone(), menu.downgrade());
            disinstalla.connect_clicked(move |_| {
                if let Some(m) = m.upgrade() {
                    m.popdown();
                }
                c.disinstalla(&a);
            });
        }
        {
            let (c, a, m) = (self.clone(), a.clone(), menu.downgrade());
            apri.connect_clicked(move |_| {
                if let Some(m) = m.upgrade() {
                    m.popdown();
                }
                c.apri_app(&a);
            });
        }
        {
            let (c, pacchetto, m) = (self.clone(), a.pacchetto.clone(), menu.downgrade());
            preferito.connect_clicked(move |_| {
                if let Some(m) = m.upgrade() {
                    m.popdown();
                }
                c.cambia_preferito(&pacchetto);
            });
        }
        if let (Some(chiudi), Some(f)) = (chiudi, aperta) {
            let (m, f) = (menu.downgrade(), f.downgrade());
            chiudi.connect_clicked(move |_| {
                if let Some(m) = m.upgrade() {
                    m.popdown();
                }
                if let Some(f) = f.upgrade() {
                    f.close();
                }
            });
        }
        menu.set_parent(riquadro);
        menu.connect_closed(|m| {
            let m = m.clone();
            // Staccato dopo la chiusura, a animazione finita.
            gtk::glib::idle_add_local_once(move || m.unparent());
        });
        menu.popup();
    }

    /// Aggiunge o toglie un'app dai preferiti e lo salva.
    fn cambia_preferito(self: &Rc<Self>, pacchetto: &str) {
        {
            let mut preferiti = self.preferiti.borrow_mut();
            match preferiti.iter().position(|p| p == pacchetto) {
                Some(i) => {
                    preferiti.remove(i);
                }
                None => preferiti.push(pacchetto.to_string()),
            }
            if let Err(e) = crate::configurazione::Telefoni::imposta_preferiti(&self.collegamento.seriale, &preferiti) {
                eprintln!("[cassetto] preferiti non salvati: {e:#}");
            }
        }
        self.mostra_preferiti();
        self.filtra();
        self.aggiorna_pallini();
    }

    /// Ricostruisce la scheda dei preferiti (solo le app ancora presenti).
    fn mostra_preferiti(self: &Rc<Self>) {
        self.griglia_preferiti.remove_all();
        let mut riquadri = Vec::new();
        let mut pallini = Vec::new();
        for pacchetto in self.preferiti.borrow().iter() {
            let app = self.per_pacchetto.borrow().get(pacchetto).cloned();
            if let Some(a) = app {
                let (riquadro, pallino) = self.riquadro(&a);
                self.griglia_preferiti.append(&riquadro);
                pallini.push((a.pacchetto.clone(), pallino));
                riquadri.push((riquadro, a));
            }
        }
        *self.riquadri_preferiti.borrow_mut() = riquadri;
        *self.pallini_preferiti.borrow_mut() = pallini;
    }

    /// Mostra solo le app il cui nome contiene il testo cercato; mentre si
    /// cerca, i preferiti si nascondono.
    fn filtra(&self) {
        let cercato = self.ricerca.text().to_lowercase();
        let riquadri = self.riquadri.borrow();
        if riquadri.is_empty() {
            return;
        }
        let con_preferiti = cercato.trim().is_empty() && !self.riquadri_preferiti.borrow().is_empty();
        self.scheda_preferiti.set_visible(con_preferiti);
        self.titolo_tutte.set_label(if cercato.trim().is_empty() { t!("TUTTE LE APP") } else { t!("RISULTATI") });
        let mut visibili = 0;
        for (riquadro, nome, _) in riquadri.iter() {
            let si = nome.contains(cercato.trim());
            riquadro.set_visible(si);
            visibili += si as usize;
        }
        self.pagine_app.set_visible_child_name(if visibili > 0 { "app" } else { "vuoto" });
    }

    /// La finestra aperta di un'app, se c'è.
    fn finestra_di(&self, pacchetto: &str) -> Option<adw::ApplicationWindow> {
        self.finestre.borrow().get(pacchetto).and_then(|w| w.upgrade())
    }

    /// Pallino sotto le icone delle app aperte.
    fn aggiorna_pallini(&self) {
        self.finestre.borrow_mut().retain(|_, w| w.upgrade().is_some());
        let aperte = self.finestre.borrow();
        for (pacchetto, pallino) in self.pallini.borrow().iter().chain(self.pallini_preferiti.borrow().iter()) {
            pallino.set_visible(aperte.contains_key(pacchetto));
        }
    }

    /// Ricostruisce le notifiche (sono poche: si rifà da capo), raggruppate per
    /// app; ogni gruppo mostra le ultime e «altre N».
    fn mostra_notifiche(self: &Rc<Self>) {
        let tutte = self.collegamento.notifiche().borrow().clone();
        self.avvisa_nuove(&tutte);
        // Le nascoste che non esistono più sul telefono si dimenticano.
        self.nascoste.borrow_mut().retain(|k, _| tutte.iter().any(|n| &n.chiave == k));
        let visibili: Vec<Notifica> =
            tutte.into_iter().filter(|n| self.nascoste.borrow().get(&n.chiave) != Some(&n.quando)).collect();
        // Gruppi nell'ordine della notifica più recente di ciascuna app.
        let mut gruppi: Vec<(String, Vec<&Notifica>)> = Vec::new();
        for n in &visibili {
            match gruppi.iter_mut().find(|(p, _)| p == &n.pacchetto) {
                Some((_, v)) => v.push(n),
                None => gruppi.push((n.pacchetto.clone(), vec![n])),
            }
        }
        while let Some(figlio) = self.colonna_notifiche.first_child() {
            self.colonna_notifiche.remove(&figlio);
        }
        for (pacchetto, notifiche) in &gruppi {
            let app = self.per_pacchetto.borrow().get(pacchetto).cloned();
            let nome_app = app.as_ref().map(|a| a.nome.clone()).unwrap_or_else(|| pacchetto.clone());
            let titolo = if notifiche.len() > 1 { format!("{nome_app} · {}", notifiche.len()) } else { nome_app.clone() };
            let elenco = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
            let tutte_visibili = self.espanse.borrow().contains(pacchetto);
            let quante = if tutte_visibili { notifiche.len() } else { notifiche.len().min(NOTIFICHE_PER_APP) };
            for n in &notifiche[..quante] {
                elenco.append(&self.riga_notifica(n, app.as_ref(), &nome_app));
            }
            if notifiche.len() > quante {
                let altre = gtk::Button::builder()
                    .label(match notifiche.len() - quante {
                        1 => t!("altre 1 notifiche di {} ›", nome_app),
                        n => t!("altre {} notifiche di {} ›", n, nome_app),
                    })
                    .halign(gtk::Align::Start)
                    .css_classes(["altre"])
                    .build();
                let (c, p) = (self.clone(), pacchetto.clone());
                altre.connect_clicked(move |_| {
                    c.espanse.borrow_mut().insert(p.clone());
                    c.mostra_notifiche();
                });
                elenco.append(&altre);
            }
            self.colonna_notifiche.append(&scheda(&titolo, &elenco));
        }
        self.pagine_notifiche.set_visible_child_name(if visibili.is_empty() { "vuoto" } else { "elenco" });
        self.nascondi_tutte.set_visible(!visibili.is_empty());
        self.conta_notifiche.set_label(&visibili.len().to_string());
        self.conta_notifiche.set_visible(!visibili.is_empty());
    }

    /// Una notifica: icona, titolo, testo, ora e «×»; il clic apre l'app.
    fn riga_notifica(self: &Rc<Self>, n: &Notifica, app: Option<&App>, nome_app: &str) -> gtk::Widget {
        let testi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).hexpand(true).build();
        testi.append(
            &gtk::Label::builder()
                .label(if n.titolo.is_empty() { nome_app } else { &n.titolo })
                .xalign(0.0)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .css_classes(["titolo"])
                .build(),
        );
        if !n.testo.is_empty() {
            testi.append(
                &gtk::Label::builder()
                    .label(&n.testo)
                    .xalign(0.0)
                    .wrap(true)
                    .wrap_mode(gtk::pango::WrapMode::WordChar)
                    .lines(3)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .css_classes(["testo"])
                    .build(),
            );
        }
        let nascondi = gtk::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text(t!("Nascondi (sul telefono resta)"))
            .valign(gtk::Align::Center)
            .css_classes(["flat", "circular"])
            .build();
        {
            let (c, chiave, quando) = (self.clone(), n.chiave.clone(), n.quando);
            nascondi.connect_clicked(move |_| {
                c.nascoste.borrow_mut().insert(chiave.clone(), quando);
                c.mostra_notifiche();
            });
        }
        let riga = gtk::Box::builder().spacing(12).css_classes(["riga-notifica"]).build();
        let figura = icona(app.map(|a| a.icona.as_slice()), LATO_ICONA_NOTIFICA);
        figura.set_valign(gtk::Align::Start);
        riga.append(&figura);
        riga.append(&testi);
        riga.append(&gtk::Label::builder().label(orario(n.quando)).valign(gtk::Align::Start).css_classes(["quando"]).build());
        riga.append(&nascondi);
        if let Some(a) = app {
            // Tutta la riga apre l'app; la «×» resta un pulsante a sé.
            let clic = gtk::GestureClick::new();
            let (c, a) = (self.clone(), a.clone());
            riga.set_tooltip_text(Some(&t!("Apri {}", a.nome)));
            clic.connect_released(move |g, _, x, y| {
                let sulla_x = g
                    .widget()
                    .and_then(|w| w.pick(x, y, gtk::PickFlags::DEFAULT))
                    .is_some_and(|w| w.ancestor(gtk::Button::static_type()).is_some());
                if !sulla_x {
                    c.apri_app(&a);
                }
            });
            riga.add_controller(clic);
            riga.set_cursor_from_name(Some("pointer"));
        }
        riga.upcast()
    }

    /// Avvisi a comparsa per le notifiche arrivate dall'ultima volta.
    fn avvisa_nuove(&self, tutte: &[Notifica]) {
        let mut viste = self.viste.borrow_mut();
        let Some(viste) = viste.as_mut() else {
            *viste = Some(tutte.iter().map(|n| (n.chiave.clone(), n.quando)).collect());
            return;
        };
        let preferenze = Preferenze::attuali();
        for n in tutte {
            if viste.insert(n.chiave.clone(), n.quando) == Some(n.quando) {
                continue;
            }
            if !preferenze.avvisi || preferenze.app_silenziate.contains(&n.pacchetto) {
                continue;
            }
            let app = self.per_pacchetto.borrow().get(&n.pacchetto).cloned();
            let nome_app = app.as_ref().map_or(n.pacchetto.clone(), |a| a.nome.clone());
            let (titolo, testo) = if preferenze.solo_nome_app {
                (nome_app, t!("Nuova notifica").to_string())
            } else if n.titolo.is_empty() {
                (nome_app, n.testo.clone())
            } else {
                (format!("{nome_app} · {}", n.titolo), n.testo.clone())
            };
            if let Some(avvisi) = self.avvisi_sistema.borrow().as_ref() {
                avvisi.mostra(&titolo, &testo, app.as_ref().map(|a| a.icona.as_slice()), &n.pacchetto);
            }
        }
        viste.retain(|k, _| tutte.iter().any(|n| &n.chiave == k));
    }

    /// La pagina Preferenze (solo le voci che fanno già qualcosa).
    fn pagina_preferenze(self: &Rc<Self>) -> gtk::Box {
        let p = Preferenze::attuali();
        let interruttore = |attivo: bool, cambia: fn(&mut Preferenze, bool)| {
            let s = gtk::Switch::builder().active(attivo).valign(gtk::Align::Center).build();
            s.connect_active_notify(move |s| {
                let valore = s.is_active();
                if let Err(e) = Preferenze::cambia(|p| cambia(p, valore)) {
                    eprintln!("[cassetto] preferenze non salvate: {e:#}");
                }
            });
            s
        };
        let colonna = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(14).build();

        let finestre = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
        finestre.append(&riga_preferenza(
            t!("Esc torna indietro"),
            t!("Il tasto Esc fa come «Indietro» di Android. Disattivalo se un'app usa Esc per altro."),
            &interruttore(p.esc_indietro, |p, v| p.esc_indietro = v),
        ));
        colonna.append(&scheda(t!("Finestre delle app"), &finestre));

        let notifiche = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
        notifiche.append(&riga_preferenza(
            t!("Avviso a comparsa"),
            t!("Un avviso del sistema quando arriva una notifica sul telefono."),
            &interruttore(p.avvisi, |p, v| p.avvisi = v),
        ));
        notifiche.append(&riga_preferenza(
            t!("Solo il nome dell'app"),
            t!("Negli avvisi niente mittente né testo: utile se altri vedono il tuo schermo."),
            &interruttore(p.solo_nome_app, |p, v| p.solo_nome_app = v),
        ));
        let scegli_app = gtk::Button::builder().valign(gtk::Align::Center).css_classes(["pulsante-vetro"]).build();
        aggiorna_scelta_app(&scegli_app);
        {
            let c = self.clone();
            scegli_app.connect_clicked(move |b| c.scegli_app_silenziate(b));
        }
        notifiche.append(&riga_preferenza(t!("App che possono avvisare"), t!("Scegli da quali app ricevere gli avvisi."), &scegli_app));
        colonna.append(&scheda(t!("Notifiche"), &notifiche));

        let file = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
        let nomi: Vec<&str> = azioni::CARTELLE.iter().map(|c| azioni::nome_cartella(c)).collect();
        let cartelle = gtk::DropDown::from_strings(&nomi);
        cartelle.set_valign(gtk::Align::Center);
        cartelle.set_selected(azioni::CARTELLE.iter().position(|c| *c == p.cartella_file).unwrap_or(0) as u32);
        cartelle.connect_selected_notify(|d| {
            if let Some(cartella) = azioni::CARTELLE.get(d.selected() as usize)
                && let Err(e) = Preferenze::cambia(|p| p.cartella_file = cartella.to_string())
            {
                eprintln!("[cassetto] preferenze non salvate: {e:#}");
            }
        });
        file.append(&riga_preferenza(t!("File inviati al telefono"), t!("Cartella del telefono in cui arrivano."), &cartelle));
        let ricevuti = gtk::Button::builder()
            .label(crate::ricevi::nome_cartella(&p.cartella_ricevuti()))
            .valign(gtk::Align::Center)
            .css_classes(["pulsante-vetro"])
            .build();
        {
            let c = self.clone();
            ricevuti.connect_clicked(move |b| c.scegli_cartella_ricevuti(b));
        }
        file.append(&riga_preferenza(t!("File ricevuti dal telefono"), t!("Cartella del PC in cui arrivano."), &ricevuti));
        colonna.append(&scheda(t!("File"), &file));

        let elenco = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
        let aggiorna = gtk::Button::builder().label(t!("Aggiorna ora")).valign(gtk::Align::Center).css_classes(["pulsante-vetro"]).build();
        {
            let c = self.clone();
            aggiorna.connect_clicked(move |_| {
                c.clone().carica();
                c.avviso(t!("Rilettura delle app del telefono…"));
            });
        }
        elenco.append(&riga_preferenza(
            t!("Elenco delle app"),
            t!("Si aggiorna da solo; usa il pulsante se manca un'app appena installata."),
            &aggiorna,
        ));
        colonna.append(&scheda(t!("App del telefono"), &elenco));

        // Lingua: «Italiano» e «English» non si traducono (ogni lingua si
        // chiama col suo nome); vale dal prossimo avvio (SPECIFICATION §15.1).
        let lingua = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).build();
        let lingue = gtk::DropDown::from_strings(&[t!("Automatica (del sistema)"), "Italiano", "English"]);
        lingue.set_valign(gtk::Align::Center);
        lingue.set_selected(match p.lingua.as_deref() {
            Some("it") => 1,
            Some("en") => 2,
            _ => 0,
        });
        lingue.connect_selected_notify(|d| {
            let scelta = match d.selected() {
                1 => Some("it"),
                2 => Some("en"),
                _ => None,
            };
            if let Err(e) = Preferenze::cambia(|p| p.lingua = scelta.map(str::to_string)) {
                eprintln!("[cassetto] preferenze non salvate: {e:#}");
            }
        });
        lingua.append(&riga_preferenza(t!("Lingua dell'interfaccia"), t!("Vale dal prossimo avvio di Phonestra."), &lingue));
        colonna.append(&scheda(t!("Lingua"), &lingua));

        let pagina = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(14).build();
        pagina.append(&gtk::Label::builder().label(t!("Preferenze")).xalign(0.0).css_classes(["titolo-pagina"]).build());
        pagina.append(&gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&colonna).vexpand(true).build());
        pagina
    }

    /// Finestra con un interruttore per app: quali possono fare avvisi.
    fn scegli_app_silenziate(self: &Rc<Self>, pulsante: &gtk::Button) {
        let mut elenco: Vec<App> = self.per_pacchetto.borrow().values().cloned().collect();
        elenco.sort_by_key(|a| a.nome.to_lowercase());
        let silenziate = Preferenze::attuali().app_silenziate;
        let lista = gtk::ListBox::builder().selection_mode(gtk::SelectionMode::None).css_classes(["boxed-list"]).build();
        for a in elenco {
            let riga = adw::SwitchRow::builder().title(&a.nome).active(!silenziate.contains(&a.pacchetto)).build();
            riga.add_prefix(&icona(Some(&a.icona), 32));
            let (pacchetto, pulsante) = (a.pacchetto.clone(), pulsante.clone());
            riga.connect_active_notify(move |r| {
                let avvisa = r.is_active();
                let esito = Preferenze::cambia(|p| {
                    p.app_silenziate.retain(|x| x != &pacchetto);
                    if !avvisa {
                        p.app_silenziate.push(pacchetto.clone());
                    }
                });
                if let Err(e) = esito {
                    eprintln!("[cassetto] preferenze non salvate: {e:#}");
                }
                aggiorna_scelta_app(&pulsante);
            });
            lista.append(&riga);
        }
        let contenuto = gtk::Box::builder().orientation(gtk::Orientation::Vertical).margin_start(12).margin_end(12).margin_top(6).margin_bottom(12).build();
        contenuto.append(&lista);
        let vista = adw::ToolbarView::new();
        vista.add_top_bar(&adw::HeaderBar::new());
        vista.set_content(Some(&gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&contenuto).vexpand(true).build()));
        let dialogo = adw::Dialog::builder().title(t!("App che possono avvisare")).content_width(420).content_height(560).child(&vista).build();
        let finestra = self.finestra.upgrade();
        dialogo.present(finestra.as_ref());
    }

    /// Gli altri telefoni configurati nella barra laterale; il clic passa a quel telefono.
    fn mostra_altri_telefoni(self: &Rc<Self>) {
        while let Some(figlio) = self.altri_telefoni.first_child() {
            self.altri_telefoni.remove(&figlio);
        }
        let elenco = Telefoni::carica().map(|t| t.elenco).unwrap_or_default();
        for t in elenco.into_iter().filter(|t| t.seriale != self.collegamento.seriale) {
            let nome = t.nome_mostrato();
            let contenuto = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            contenuto.append(&gtk::Box::builder().valign(gtk::Align::Center).css_classes(["punto", "grigio"]).build());
            contenuto.append(&gtk::Label::builder().label(&nome).xalign(0.0).hexpand(true).ellipsize(gtk::pango::EllipsizeMode::End).build());
            contenuto.append(&gtk::Label::builder().label(t!("non attivo")).css_classes(["stato-telefono"]).build());
            let voce = gtk::Button::builder().child(&contenuto).tooltip_text(t!("Passa a «{}»", nome)).css_classes(["voce-laterale", "altro-telefono"]).build();
            let c = self.clone();
            voce.connect_clicked(move |_| c.passa_a(&t));
            self.altri_telefoni.append(&voce);
        }
    }

    /// Passa a un altro telefono (uno solo attivo alla volta, SPECIFICATION §6):
    /// le app aperte si chiudono, poi Phonestra si riavvia collegato a quello.
    fn passa_a(self: &Rc<Self>, t: &crate::configurazione::Telefono) {
        let (c, t) = (self.clone(), t.clone());
        gtk::glib::spawn_future_local(async move {
            let aperte = c.finestre.borrow().values().filter(|w| w.upgrade().is_some()).count();
            if aperte > 0 {
                let domanda = match aperte {
                    1 => t!("Chiudere 1 app di «{}» e passare a «{}»?", c.nome.borrow(), t.nome_mostrato()),
                    n => t!("Chiudere {} app di «{}» e passare a «{}»?", n, c.nome.borrow(), t.nome_mostrato()),
                };
                if !c.conferma(&domanda, t!("Un solo telefono alla volta: le finestre delle app si chiudono."), t!("Passa"), false).await {
                    return;
                }
            }
            if let Err(e) = Telefoni::metti_primo(&t.seriale) {
                c.avviso(&t!("Cambio non riuscito: {}", format!("{e:#}")));
                return;
            }
            RIAVVIA.store(true, Ordering::SeqCst);
            for w in c.app.windows() {
                w.close();
            }
        });
    }

    /// Un messaggio breve in fondo alla finestra.
    fn avviso(&self, testo: &str) {
        self.avvisi.add_toast(adw::Toast::builder().title(testo).timeout(5).build());
    }

    fn adb(&self) -> Option<crate::adb::Adb> {
        let adb = self.collegamento.adb().borrow().clone();
        if adb.is_none() {
            self.avviso(t!("Il telefono non è collegato"));
        }
        adb
    }

    /// Chiede conferma; `true` se l'utente sceglie `azione`.
    async fn conferma(&self, titolo: &str, testo: &str, azione: &str, distruttiva: bool) -> bool {
        let dialogo = adw::AlertDialog::new(Some(titolo), Some(testo));
        dialogo.add_responses(&[("annulla", t!("Annulla")), ("si", azione)]);
        dialogo.set_response_appearance(
            "si",
            if distruttiva { adw::ResponseAppearance::Destructive } else { adw::ResponseAppearance::Suggested },
        );
        dialogo.set_default_response(Some(if distruttiva { "annulla" } else { "si" }));
        dialogo.set_close_response("annulla");
        let finestra = self.finestra.upgrade();
        dialogo.choose_future(finestra.as_ref()).await == "si"
    }

    /// «Rinomina…»: il nome mostrato in Phonestra (sul telefono non cambia).
    fn rinomina(self: &Rc<Self>) {
        let c = self.clone();
        gtk::glib::spawn_future_local(async move {
            let campo = gtk::Entry::builder().text(c.nome.borrow().as_str()).activates_default(true).build();
            let dialogo = adw::AlertDialog::new(
                Some(t!("Rinomina il telefono")),
                Some(t!("Il nome si vede solo in Phonestra. Lascia vuoto per tornare a quello predefinito.")),
            );
            dialogo.set_extra_child(Some(&campo));
            dialogo.add_responses(&[("annulla", t!("Annulla")), ("si", t!("Rinomina"))]);
            dialogo.set_response_appearance("si", adw::ResponseAppearance::Suggested);
            dialogo.set_default_response(Some("si"));
            dialogo.set_close_response("annulla");
            let finestra = c.finestra.upgrade();
            if dialogo.choose_future(finestra.as_ref()).await != "si" {
                return;
            }
            match Telefoni::rinomina(&c.collegamento.seriale, &campo.text()) {
                Ok(nuovo) => {
                    for etichetta in &c.nomi {
                        etichetta.set_label(&nuovo);
                    }
                    *c.nome.borrow_mut() = nuovo;
                }
                Err(e) => c.avviso(&t!("Nome non salvato: {}", format!("{e:#}"))),
            }
        });
    }

    /// «Dimentica questo telefono…»: lo toglie dalla configurazione e chiude
    /// Phonestra (il telefono viene rimesso com'era, come a ogni chiusura).
    fn dimentica(self: &Rc<Self>) {
        let c = self.clone();
        gtk::glib::spawn_future_local(async move {
            let nome = c.nome.borrow().clone();
            let testo = t!(
                "Phonestra toglie «{}» dalla sua configurazione e si chiude; per collegarlo di nuovo basterà «Aggiungi telefono».\n\n\
                 Sul telefono questo PC resta associato: per toglierlo, Debug wireless › Dispositivi associati (o, se era stato autorizzato col cavo, Opzioni sviluppatore › Revoca autorizzazioni debug USB).",
                nome
            );
            if !c.conferma(&t!("Dimenticare «{}»?", nome), &testo, t!("Dimentica"), true).await {
                return;
            }
            if let Err(e) = Telefoni::dimentica(&c.collegamento.seriale) {
                c.avviso(&t!("Telefono non dimenticato: {}", format!("{e:#}")));
                return;
            }
            // Se restano altri telefoni, Phonestra riparte col primo.
            if Telefoni::carica().is_ok_and(|t| !t.elenco.is_empty()) {
                RIAVVIA.store(true, Ordering::SeqCst);
            }
            for w in c.app.windows() {
                w.close();
            }
        });
    }

    fn informazioni(&self) {
        let dialogo = adw::AboutDialog::builder()
            .application_name("Phonestra")
            .application_icon(icona_nel_tema())
            .version(env!("CARGO_PKG_VERSION"))
            .developer_name("nic-fio")
            .comments(t!("Le app del telefono Android in finestre sul PC Linux, senza installare niente sul telefono."))
            .build();
        let finestra = self.finestra.upgrade();
        dialogo.present(finestra.as_ref());
    }

    /// «Installa app…» (`apk`) o «Invia file…»: scelta dei file, poi trasferimento.
    fn scegli_file(self: &Rc<Self>, apk: bool) {
        let dialogo = gtk::FileDialog::builder().title(if apk { t!("Scegli l'app da installare") } else { t!("Scegli i file da inviare") }).modal(true).build();
        if apk {
            let filtro = gtk::FileFilter::new();
            filtro.set_name(Some(t!("App Android (.apk)")));
            filtro.add_suffix("apk");
            let filtri = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filtri.append(&filtro);
            dialogo.set_filters(Some(&filtri));
        }
        let c = self.clone();
        gtk::glib::spawn_future_local(async move {
            let finestra = c.finestra.upgrade();
            let scelti: Vec<gtk::gio::File> = if apk {
                match dialogo.open_future(finestra.as_ref()).await {
                    Ok(f) => vec![f],
                    Err(_) => return,
                }
            } else {
                match dialogo.open_multiple_future(finestra.as_ref()).await {
                    Ok(elenco) => (0..elenco.n_items()).filter_map(|i| elenco.item(i).and_downcast::<gtk::gio::File>()).collect(),
                    Err(_) => return,
                }
            };
            c.trasferisci(scelti);
        });
    }

    /// Manda i file al telefono uno alla volta: gli `.apk` si installano (con
    /// conferma), il resto va nei Download.
    fn trasferisci(self: &Rc<Self>, file: Vec<gtk::gio::File>) {
        if file.is_empty() {
            return;
        }
        if self.occupato.replace(true) {
            self.avviso(t!("Aspetta la fine del trasferimento in corso"));
            return;
        }
        let c = self.clone();
        gtk::glib::spawn_future_local(async move {
            c.annullato.store(false, Ordering::SeqCst);
            for f in file {
                c.trasferisci_uno(&f).await;
                if c.annullato.load(Ordering::SeqCst) {
                    break;
                }
            }
            c.telefono.trasferimento.set_visible(false);
            c.occupato.set(false);
        });
    }

    async fn trasferisci_uno(self: &Rc<Self>, file: &gtk::gio::File) {
        let nome = file.basename().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|| "file".into());
        let apk = nome.to_lowercase().ends_with(".apk");
        if apk {
            let testo = t!(
                "L'app verrà installata su «{}». Dal PC Android non chiede conferma: installa solo app di cui ti fidi.",
                self.nome.borrow()
            );
            if !self.conferma(&t!("Installare «{}»?", nome), &testo, t!("Installa"), false).await {
                return;
            }
        }
        let Some(adb) = self.adb() else { return };
        let t = &self.telefono;
        t.icona_trasferimento.set_icon_name(Some("document-send-symbolic"));
        t.nome_trasferimento.set_label(&nome);
        t.dettaglio_trasferimento.set_label(t!("lettura…"));
        t.barra_trasferimento.set_fraction(0.0);
        t.trasferimento.set_visible(true);
        let dati = match file.load_contents_future().await {
            Ok((d, _)) => d.to_vec(),
            Err(e) => {
                self.avviso(&t!("«{}» non letto: {}", nome, e));
                return;
            }
        };
        let totale = dati.len();
        let mandati = Arc::new(AtomicUsize::new(0));
        let orologio = {
            let (m, t) = (mandati.clone(), t.clone());
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                let n = m.load(Ordering::SeqCst);
                t.barra_trasferimento.set_fraction(n as f64 / totale.max(1) as f64);
                let dettaglio = if apk && n >= totale {
                    t!("installazione in corso…").to_string()
                } else if apk {
                    t!("Installazione · {} di {}", misura(n), misura(totale))
                } else {
                    t!("Invio al telefono · {} di {}", misura(n), misura(totale))
                };
                t.dettaglio_trasferimento.set_label(&dettaglio);
                gtk::glib::ControlFlow::Continue
            })
        };
        let avanzamento = {
            let (m, a) = (mandati.clone(), self.annullato.clone());
            move |n: usize| {
                m.store(n, Ordering::SeqCst);
                !a.load(Ordering::SeqCst)
            }
        };
        let nome_file = nome.clone();
        let cartella = Preferenze::attuali().cartella_file;
        let cartella_mostrata = azioni::nome_cartella(&cartella).to_string();
        let esito = esecutore()
            .spawn(async move {
                if apk {
                    azioni::installa(&adb, &dati, avanzamento).await.map(|()| None)
                } else {
                    azioni::invia_file(&adb, &cartella, &nome_file, &dati, avanzamento).await.map(Some)
                }
            })
            .await;
        orologio.remove();
        match esito {
            Ok(Ok(None)) => {
                self.avviso(&t!("«{}» installata", nome));
                self.clone().carica();
            }
            Ok(Ok(Some(usato))) => self.avviso(&t!("«{}» è in {} sul telefono", usato, cartella_mostrata)),
            Ok(Err(_)) if self.annullato.load(Ordering::SeqCst) => self.avviso(&t!("Invio di «{}» annullato", nome)),
            Ok(Err(e)) => self.avviso(&t!("«{}»: {}", nome, format!("{e:#}"))),
            Err(e) => self.avviso(&t!("«{}»: {}", nome, e)),
        }
    }

    /// «Ricevi file…»: la finestra per scegliere i file del telefono, poi la copia.
    fn scegli_dal_telefono(self: &Rc<Self>) {
        let Some(adb) = self.adb() else { return };
        let arrivo = crate::ricevi::nome_cartella(&Preferenze::attuali().cartella_ricevuti());
        let c = self.clone();
        let finestra = self.finestra.upgrade();
        crate::ricevi::apri(finestra.as_ref(), adb, &arrivo, move |file| c.ricevi(file));
    }

    /// Preferenze › File: la cartella del PC dove arrivano i file ricevuti.
    fn scegli_cartella_ricevuti(self: &Rc<Self>, pulsante: &gtk::Button) {
        let attuale = Preferenze::attuali().cartella_ricevuti();
        let dialogo = gtk::FileDialog::builder()
            .title(t!("Dove salvare i file ricevuti dal telefono"))
            .initial_folder(&gtk::gio::File::for_path(&attuale))
            .modal(true)
            .build();
        let (c, pulsante) = (self.clone(), pulsante.clone());
        gtk::glib::spawn_future_local(async move {
            let finestra = c.finestra.upgrade();
            let Ok(scelta) = dialogo.select_folder_future(finestra.as_ref()).await else { return };
            let Some(percorso) = scelta.path() else { return };
            // Scaricati resta «nessuna scelta»: segue il sistema se cambia.
            let valore = (gtk::glib::user_special_dir(gtk::glib::UserDirectory::Downloads).as_deref() != Some(percorso.as_path())).then_some(percorso.clone());
            match Preferenze::cambia(|p| p.cartella_ricevuti = valore) {
                Ok(()) => pulsante.set_label(&crate::ricevi::nome_cartella(&percorso)),
                Err(e) => c.avviso(&t!("Preferenza non salvata: {}", format!("{e:#}"))),
            }
        });
    }

    /// Copia i file scelti dal telefono al PC, uno alla volta, con la scheda
    /// di trasferimento sul telefono disegnato; alla fine un avviso con
    /// «Apri la cartella».
    fn ricevi(self: &Rc<Self>, file: Vec<crate::ricevi::FileTelefono>) {
        if self.occupato.replace(true) {
            self.avviso(t!("Aspetta la fine del trasferimento in corso"));
            return;
        }
        let Some(adb) = self.adb() else {
            self.occupato.set(false);
            return;
        };
        let cartella = Preferenze::attuali().cartella_ricevuti();
        let nome_cartella = crate::ricevi::nome_cartella(&cartella);
        let c = self.clone();
        gtk::glib::spawn_future_local(async move {
            c.annullato.store(false, Ordering::SeqCst);
            let t = c.telefono.clone();
            t.icona_trasferimento.set_icon_name(Some("document-save-symbolic"));
            t.barra_trasferimento.set_fraction(0.0);
            t.trasferimento.set_visible(true);
            let quanti = file.len();
            let mut arrivati: Vec<std::path::PathBuf> = Vec::new();
            let mut errori: Vec<String> = Vec::new();
            let _ = std::fs::create_dir_all(&cartella);
            for (i, f) in file.into_iter().enumerate() {
                if c.annullato.load(Ordering::SeqCst) {
                    break;
                }
                t.nome_trasferimento.set_label(&f.nome);
                let numero = i + 1;
                let arrivo = crate::ricevi::arrivo(&cartella, &f.nome);
                let ricevuti = Arc::new(std::sync::atomic::AtomicU64::new(0));
                let orologio = {
                    let (r, t, totale) = (ricevuti.clone(), t.clone(), f.dimensione);
                    let aggiorna = move || {
                        let n = r.load(Ordering::SeqCst);
                        t.barra_trasferimento.set_fraction(n as f64 / totale.max(1) as f64);
                        let (fatti, totale) = (crate::ricevi::misura(n), crate::ricevi::misura(totale));
                        t.dettaglio_trasferimento.set_label(&if quanti > 1 {
                            t!("Dal telefono · {} di {} · {} di {}", numero, quanti, fatti, totale)
                        } else {
                            t!("Dal telefono · {} di {}", fatti, totale)
                        });
                    };
                    aggiorna();
                    gtk::glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
                        aggiorna();
                        gtk::glib::ControlFlow::Continue
                    })
                };
                let (adb, a, percorso, destinazione, modificato) =
                    (adb.clone(), c.annullato.clone(), f.percorso.clone(), arrivo.clone(), f.modificato);
                let esito = esecutore()
                    .spawn(async move {
                        let mut uscita = std::fs::File::create(&destinazione)?;
                        let copia = sync::ricevi(&adb, &percorso, &mut uscita, |n| {
                            ricevuti.store(n, Ordering::SeqCst);
                            !a.load(Ordering::SeqCst)
                        })
                        .await;
                        match copia {
                            Ok(_) => {
                                // La data del telefono: le foto restano in ordine.
                                let quando = std::time::UNIX_EPOCH + std::time::Duration::from_secs(modificato.max(0) as u64);
                                let _ = uscita.set_modified(quando);
                                Ok(())
                            }
                            Err(e) => {
                                drop(uscita);
                                let _ = std::fs::remove_file(&destinazione);
                                Err(e)
                            }
                        }
                    })
                    .await;
                orologio.remove();
                match esito {
                    Ok(Ok(())) => arrivati.push(arrivo),
                    Ok(Err(_)) if c.annullato.load(Ordering::SeqCst) => break,
                    Ok(Err(e)) => errori.push(t!("«{}»: {}", f.nome, format!("{e:#}"))),
                    Err(e) => errori.push(t!("«{}»: {}", f.nome, e)),
                }
            }
            t.trasferimento.set_visible(false);
            t.icona_trasferimento.set_icon_name(Some("document-send-symbolic"));
            c.occupato.set(false);
            let annullato = c.annullato.load(Ordering::SeqCst);
            let testo = match (arrivati.len(), errori.len(), annullato) {
                (0, 0, true) => t!("Ricezione annullata").to_string(),
                (1, _, true) => t!("Ricezione annullata: 1 file in {}", nome_cartella),
                (n, _, true) => t!("Ricezione annullata: {} file in {}", n, nome_cartella),
                (1, 0, _) => t!(
                    "«{}» ricevuto in {}",
                    arrivati[0].file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                    nome_cartella
                ),
                (n, 0, _) => t!("{} file ricevuti in {}", n, nome_cartella),
                (0, _, _) => t!("Nessun file ricevuto: {}", errori[0]),
                (1, e, _) => t!("1 file ricevuto in {}, {} no: {}", nome_cartella, e, errori[0]),
                (n, e, _) => t!("{} file ricevuti in {}, {} no: {}", n, nome_cartella, e, errori[0]),
            };
            for e in &errori {
                eprintln!("[ricevi] {e}");
            }
            let avviso = adw::Toast::builder().title(testo).timeout(8).build();
            if !arrivati.is_empty() {
                avviso.set_button_label(Some(t!("Apri la cartella")));
                let (c2, singolo) = (c.clone(), (arrivati.len() == 1).then(|| arrivati[0].clone()));
                let cartella = cartella.clone();
                avviso.connect_button_clicked(move |_| {
                    let finestra = c2.finestra.upgrade();
                    // Un file solo: la cartella con il file già evidenziato.
                    match &singolo {
                        Some(f) => gtk::FileLauncher::new(Some(&gtk::gio::File::for_path(f))).open_containing_folder(
                            finestra.as_ref(),
                            None::<&gtk::gio::Cancellable>,
                            |_| {},
                        ),
                        None => gtk::FileLauncher::new(Some(&gtk::gio::File::for_path(&cartella))).launch(
                            finestra.as_ref(),
                            None::<&gtk::gio::Cancellable>,
                            |_| {},
                        ),
                    }
                });
            }
            c.avvisi.add_toast(avviso);
        });
    }

    /// «Disinstalla…»: con conferma; solo app dell'utente.
    fn disinstalla(self: &Rc<Self>, a: &App) {
        let (c, a) = (self.clone(), a.clone());
        gtk::glib::spawn_future_local(async move {
            let testo = t!("L'app e i suoi dati verranno tolti da «{}».", c.nome.borrow());
            if !c.conferma(&t!("Disinstallare {}?", a.nome), &testo, t!("Disinstalla"), true).await {
                return;
            }
            let Some(adb) = c.adb() else { return };
            if let Some(f) = c.finestra_di(&a.pacchetto) {
                f.close();
            }
            let pacchetto = a.pacchetto.clone();
            match esecutore().spawn(async move { azioni::disinstalla(&adb, &pacchetto).await }).await {
                Ok(Ok(())) => {
                    c.avviso(&t!("{} disinstallata", a.nome));
                    if c.preferiti.borrow().contains(&a.pacchetto) {
                        c.cambia_preferito(&a.pacchetto);
                    }
                    c.clone().carica();
                }
                Ok(Err(e)) => c.avviso(&t!("{} non disinstallata: {}", a.nome, format!("{e:#}"))),
                Err(e) => c.avviso(&t!("{} non disinstallata: {}", a.nome, e)),
            }
        });
    }

    /// Apre l'app nella sua finestra, o porta in primo piano quella già aperta.
    fn apri_app(self: &Rc<Self>, a: &App) {
        self.apri_finestra(&a.pacchetto, &a.nome);
    }

    /// Apre `pacchetto` (un'app o lo schermo del telefono) nella sua finestra,
    /// o porta in primo piano quella già aperta.
    fn apri_finestra(self: &Rc<Self>, pacchetto: &str, nome: &str) {
        if let Some(f) = self.finestra_di(pacchetto) {
            f.present();
            return;
        }
        let f = finestra::apri(&self.app, self.collegamento.clone(), pacchetto, nome);
        self.finestre.borrow_mut().insert(pacchetto.to_string(), f.downgrade());
        {
            // Finestra chiusa → via il pallino.
            let c = Rc::downgrade(self);
            f.connect_destroy(move |_| {
                if let Some(c) = c.upgrade() {
                    gtk::glib::idle_add_local_once(move || c.aggiorna_pallini());
                }
            });
        }
        self.aggiorna_pallini();
    }
}
