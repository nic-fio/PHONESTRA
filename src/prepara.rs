//! «Aggiungi un telefono» senza cavo (SPECIFICHE §5.2; mockup
//! `mockup/proposte/guida-elenco.html`): l'elenco delle impostazioni da
//! attivare sul telefono. Come farlo sul proprio modello lo scopre l'utente:
//! per ogni voce la parola da cercare nelle Impostazioni e «Chiedi a Google»
//! (Modalità IA di Google Search con la domanda già scritta).
//!
//! Le voci che si vedono in rete si spuntano da sole: il Debug wireless acceso
//! (`_adb-tls-connect`) e la schermata del codice aperta (`_adb-tls-pairing`),
//! che attiva il campo delle 6 cifre. Con le 6 cifre Phonestra si associa
//! (`adb::abbina`), si collega, toglie la scadenza all'autorizzazione e salva
//! il telefono. Per i telefoni con Android 10 o precedente resta la
//! procedura col cavo (`procedura`).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use adw::prelude::*;

use crate::configurazione::{self, Telefoni, Telefono};
use crate::rete::{self, TelefonoInRete};
use crate::telefono::Collegamento;

/// Una voce dell'elenco.
struct Voce {
    titolo: &'static str,
    sotto: &'static str,
    perche: &'static str,
    /// La parola da cercare nelle Impostazioni del telefono.
    cerca: Option<&'static str>,
    domanda: &'static str,
}

const VOCI: [Voce; 4] = [
    Voce {
        titolo: "Telefono e PC sulla stessa rete Wi-Fi",
        sotto: "La rete di casa o dell'ufficio",
        perche: "Phonestra e il telefono si parlano attraverso la rete. Non vanno bene la rete «ospiti» né i dati mobili del telefono.",
        cerca: None,
        domanda: "Come vedo a quale rete Wi-Fi è collegato il mio telefono Android?",
    },
    Voce {
        titolo: "Sblocca le Opzioni sviluppatore",
        sotto: "Impostazioni nascoste di Android",
        perche: "Servono per permettere a un PC di collegarsi. Il nome spaventa, ma sono sicure e si possono spegnere quando vuoi. Di solito si sbloccano toccando 7 volte «Numero build».",
        cerca: Some("numero build"),
        domanda: "Come si attivano le Opzioni sviluppatore sul mio telefono Android?",
    },
    Voce {
        titolo: "Spegni le protezioni che bloccano il collegamento",
        sotto: "Solo se ci sono",
        perche: "Alcuni telefoni hanno una protezione che rifiuta ogni collegamento dal PC: Blocco automatico (Samsung), Protezione avanzata (Google Pixel). Se c'è ed è accesa, il Debug wireless resta grigio o si rispegne. Se non la trovi, il tuo telefono non ce l'ha.",
        cerca: Some("blocco automatico"),
        domanda: "Sul mio telefono Android c'è una protezione come Blocco automatico o Protezione avanzata che impedisce il debug wireless? Come la spengo?",
    },
    Voce {
        titolo: "Debug wireless: accendilo e associa questo PC",
        sotto: "Una sola voce, due azioni",
        perche: "1. Accendi l'interruttore e tocca «Consenti» per la rete di casa: me ne accorgo da solo.\n2. Tocca la scritta «Debug wireless» (non l'interruttore): si apre la sua pagina. Lì tocca «Associa dispositivo con codice di associazione» e scrivi qui sotto le 6 cifre. Si fa una volta sola.",
        cerca: Some("debug wireless"),
        domanda: "Come accendo il Debug wireless e associo un computer col codice di associazione sul mio telefono Android?",
    },
];
const ULTIMA: usize = VOCI.len() - 1;

/// La funzione che ridisegna la finestra, condivisa dai pulsanti.
type Ridisegna = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

/// Cosa arriva dai controlli in sottofondo.
enum Evento {
    /// Telefoni nuovi col Debug wireless acceso, e la schermata del codice.
    Rete { accesi: Vec<TelefonoInRete>, codice: Option<TelefonoInRete> },
    Associato(Result<Telefono, String>),
}

#[derive(Default)]
struct Stato {
    /// Voci che l'utente ha segnato come fatte.
    fatte: [bool; ULTIMA],
    aperta: usize,
    acceso: bool,
    codice: Option<TelefonoInRete>,
    associazione_in_corso: bool,
    errore: Option<String>,
    fatto: Option<Telefono>,
}

/// I widget di una voce.
struct Scheda {
    riquadro: gtk::Box,
    numero: gtk::Label,
    segno: gtk::Label,
    corpo: gtk::Revealer,
}

/// Apre «Aggiungi un telefono»; `fatto` riceve il telefono appena salvato.
pub fn apri(app: &adw::Application, fatto: impl Fn(Telefono) + 'static) -> adw::ApplicationWindow {
    crate::cassetto::stile();
    stile();
    let fatto: Rc<dyn Fn(Telefono)> = Rc::new(fatto);
    let stato: Rc<RefCell<Stato>> = Rc::default();

    // Il campo del codice, dentro l'ultima voce.
    let campo = gtk::Entry::builder()
        .max_length(6)
        .width_chars(8)
        .xalign(0.5)
        .input_purpose(gtk::InputPurpose::Digits)
        .placeholder_text("······")
        .halign(gtk::Align::Start)
        .sensitive(false)
        .css_classes(["codice"])
        .build();
    let nota_codice = gtk::Label::builder().xalign(0.0).wrap(true).css_classes(["nota"]).build();

    // La funzione che ridisegna nasce dopo i pulsanti: la ricevono qui.
    let ridisegna: Ridisegna = Rc::default();
    let elenco = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(8).build();
    let mut schede = Vec::new();
    for (i, v) in VOCI.iter().enumerate() {
        let numero = gtk::Label::builder().label((i + 1).to_string()).valign(gtk::Align::Center).css_classes(["num"]).build();
        let testi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).hexpand(true).build();
        testi.append(&gtk::Label::builder().label(v.titolo).xalign(0.0).wrap(true).css_classes(["titolo"]).build());
        testi.append(&gtk::Label::builder().label(v.sotto).xalign(0.0).css_classes(["sotto"]).build());
        let segno = gtk::Label::builder().valign(gtk::Align::Center).css_classes(["segno"]).build();
        let capo = gtk::Box::builder().spacing(14).css_classes(["capo"]).build();
        capo.append(&numero);
        capo.append(&testi);
        capo.append(&segno);

        let dentro = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).css_classes(["dentro"]).build();
        dentro.append(&gtk::Label::builder().label(v.perche).xalign(0.0).wrap(true).css_classes(["perche"]).build());
        if let Some(parola) = v.cerca {
            let dove = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(4).css_classes(["modo"]).build();
            dove.append(&gtk::Label::builder().label("DOVE SI TROVA").xalign(0.0).css_classes(["etichetta"]).build());
            dove.append(
                &gtk::Label::builder()
                    .label(format!("Usa lo strumento di ricerca delle Impostazioni per individuare l'impostazione: scrivi <b>{parola}</b>."))
                    .use_markup(true)
                    .xalign(0.0)
                    .wrap(true)
                    .build(),
            );
            dentro.append(&dove);
        }
        if i == ULTIMA {
            dentro.append(&campo);
            dentro.append(&nota_codice);
        }
        let azioni = gtk::Box::builder().spacing(8).build();
        if i < ULTIMA {
            let fatta = gtk::Button::builder().label("Fatto ›").css_classes(["suggested-action", "pill"]).build();
            let (st, r, n) = (stato.clone(), ridisegna.clone(), i);
            fatta.connect_clicked(move |_| {
                {
                    let mut s = st.borrow_mut();
                    s.fatte[n] = true;
                    s.aperta = n + 1;
                }
                if let Some(a) = r.borrow().as_ref() {
                    a();
                }
            });
            azioni.append(&fatta);
        }
        let google = gtk::Button::builder().label("Chiedi a Google ↗").css_classes(["pill"]).build();
        let domanda = v.domanda;
        google.connect_clicked(move |b| chiedi_a_google(b, domanda));
        azioni.append(&google);
        dentro.append(&azioni);
        let corpo = gtk::Revealer::builder().child(&dentro).transition_type(gtk::RevealerTransitionType::SlideDown).build();

        let riquadro = gtk::Box::builder().orientation(gtk::Orientation::Vertical).css_classes(["voce-elenco"]).build();
        riquadro.append(&capo);
        riquadro.append(&corpo);
        elenco.append(&riquadro);
        schede.push(Scheda { riquadro, numero, segno, corpo });
    }

    // In cima: titolo e spiegazione; alla fine il riquadro «Fatto».
    let titolo = gtk::Label::builder().label("Prepara il telefono").xalign(0.0).css_classes(["titolo-istruzioni"]).build();
    let intro = gtk::Label::builder()
        .label("Per collegarmi al telefono servono queste 4 cose, una volta sola, senza cavo. Ogni telefono le ha in un posto un po' diverso: per ognuna ti dico cosa cercare, e «Chiedi a Google» ti spiega come si fa sul tuo modello, spesso con un video. Quelle che vedo da qui le spunto da solo.")
        .xalign(0.0)
        .wrap(true)
        .css_classes(["spiega"])
        .build();
    let esito = gtk::Label::builder().xalign(0.0).wrap(true).css_classes(["titolo-istruzioni"]).build();
    let inizia = gtk::Button::builder().label("Inizia a usare il telefono").halign(gtk::Align::Start).css_classes(["suggested-action", "pill"]).build();
    let riquadro_fatto = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(12).visible(false).css_classes(["fatto-tutto"]).build();
    riquadro_fatto.append(&esito);
    riquadro_fatto.append(&inizia);
    let colonna = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(12).build();
    colonna.append(&titolo);
    colonna.append(&intro);
    colonna.append(&riquadro_fatto);
    colonna.append(&elenco);
    let scorri = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&colonna).hexpand(true).build();

    // A destra: cosa si vede in rete, l'aiuto e la strada col cavo.
    let in_rete = gtk::Label::builder().xalign(0.0).wrap(true).max_width_chars(30).build();
    let lato = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(12).width_request(270).hexpand(false).build();
    lato.append(&riquadro_laterale("COSA VEDO IN RETE", &in_rete));
    lato.append(&riquadro_laterale(
        "TI SEI BLOCCATO?",
        &gtk::Label::builder()
            .label("Fai una foto di questa finestra e mandala a chi ti ha consigliato Phonestra: c'è scritto a che punto sei.")
            .xalign(0.0)
            .wrap(true)
            .max_width_chars(30)
            .build(),
    ));
    let col_cavo = gtk::Button::builder().label("Android 10 o precedente?\nCollega col cavo").css_classes(["flat"]).halign(gtk::Align::Start).build();
    lato.append(&col_cavo);

    let corpo = gtk::Box::builder().spacing(22).margin_start(24).margin_end(24).margin_bottom(22).margin_top(6).build();
    corpo.append(&scorri);
    corpo.append(&lato);
    let barra = adw::HeaderBar::new();
    barra.set_title_widget(Some(&gtk::Label::builder().label("Aggiungi un telefono").css_classes(["titolo-app"]).build()));
    let vista = adw::ToolbarView::new();
    vista.add_top_bar(&barra);
    vista.set_content(Some(&corpo));
    let finestra = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(1060)
        .default_height(720)
        .title("Aggiungi un telefono")
        .content(&vista)
        .css_classes(["phonestra-drawer", "procedura", "prepara"])
        .build();
    crate::cassetto::segui_tema(&finestra);

    // Ridisegna tutto dallo stato.
    let schede = Rc::new(schede);
    let aggiorna: Rc<dyn Fn()> = {
        let (st, schede, campo, nota_codice, in_rete, riquadro_fatto, esito) =
            (stato.clone(), schede.clone(), campo.clone(), nota_codice.clone(), in_rete.clone(), riquadro_fatto.clone(), esito.clone());
        Rc::new(move || {
            let s = st.borrow();
            // Col Debug wireless acceso le prime tre sono per forza fatte.
            let fatta = |i: usize| s.fatto.is_some() || (i < ULTIMA && (s.fatte[i] || s.acceso));
            for (i, sc) in schede.iter().enumerate() {
                for c in ["fatta", "aperta"] {
                    sc.riquadro.remove_css_class(c);
                }
                let aperta = i == s.aperta && s.fatto.is_none();
                sc.corpo.set_reveal_child(aperta);
                if aperta {
                    sc.riquadro.add_css_class("aperta");
                }
                if fatta(i) {
                    sc.riquadro.add_css_class("fatta");
                    sc.numero.set_label("✓");
                } else {
                    sc.numero.set_label(&(i + 1).to_string());
                }
                let (testo, classe) = match i {
                    ULTIMA if s.fatto.is_some() => ("✓ collegato", "visto"),
                    ULTIMA if s.associazione_in_corso => ("associo…", "attesa"),
                    ULTIMA if s.codice.is_some() => ("scrivi il codice", "attesa"),
                    ULTIMA if s.acceso => ("✓ acceso: ora il codice", "visto"),
                    ULTIMA => ("me ne accorgo da solo", "attesa"),
                    _ if fatta(i) && s.acceso => ("✓ visto da Phonestra", "visto"),
                    _ if fatta(i) => ("✓ fatto", "visto"),
                    _ => ("da fare sul telefono", "mano"),
                };
                sc.segno.set_label(testo);
                for c in ["visto", "attesa", "mano"] {
                    sc.segno.remove_css_class(c);
                }
                sc.segno.add_css_class(classe);
            }
            campo.set_sensitive(s.codice.is_some() && !s.associazione_in_corso && s.fatto.is_none());
            nota_codice.set_label(&match (&s.errore, &s.codice) {
                (Some(e), _) => e.clone(),
                (None, Some(_)) => "Vedo la schermata del codice: scrivi le 6 cifre, al resto penso io.".into(),
                (None, None) => "Il campo si attiva quando apri la schermata del codice sul telefono.".into(),
            });
            in_rete.set_label(&match (&s.fatto, &s.codice, s.acceso) {
                (Some(t), _, _) => format!("● {} collegato", t.nome),
                (None, Some(_), _) => "● Un telefono col Debug wireless acceso\n● Schermata del codice aperta".into(),
                (None, None, true) => "● Un telefono col Debug wireless acceso".into(),
                (None, None, false) => "○ Nessun telefono col Debug wireless acceso".into(),
            });
            riquadro_fatto.set_visible(s.fatto.is_some());
            if let Some(t) = &s.fatto {
                esito.set_label(&format!("Fatto! «{}» è collegato via Wi-Fi.", t.nome));
            }
        })
    };
    *ridisegna.borrow_mut() = Some(aggiorna.clone());
    for (i, sc) in schede.iter().enumerate() {
        let clic = gtk::GestureClick::new();
        let (st, a) = (stato.clone(), aggiorna.clone());
        clic.connect_released(move |_, _, _, _| {
            st.borrow_mut().aperta = i;
            a();
        });
        if let Some(capo) = sc.riquadro.first_child() {
            capo.add_controller(clic);
        }
    }

    // Controlli in sottofondo e associazione.
    let annullato = Arc::new(AtomicBool::new(false));
    {
        let a = annullato.clone();
        finestra.connect_close_request(move |_| {
            a.store(true, Ordering::SeqCst);
            gtk::glib::Propagation::Proceed
        });
    }
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Evento>();
    // Per le prove dell'interfaccia: `PHONESTRA_PROVA_PASSO=acceso|codice|fatto`.
    match std::env::var("PHONESTRA_PROVA_PASSO").ok().as_deref() {
        Some(p) => {
            let finto = TelefonoInRete { seriale: "PROVA".into(), indirizzo: ([192, 168, 0, 99], 40000).into(), istanza: String::new() };
            let _ = match p {
                "acceso" => tx.send(Evento::Rete { accesi: vec![finto], codice: None }),
                "codice" => tx.send(Evento::Rete { accesi: vec![finto.clone()], codice: Some(finto) }),
                "fatto" => tx.send(Evento::Associato(Ok(Telefono {
                    seriale: "PROVA".into(),
                    nome: "Galaxy S26 di prova".into(),
                    modello: "SM-S942B".into(),
                    android: "16".into(),
                    ultimo_indirizzo: None,
                    spegnimento_originale: None,
                    volume_originale: None,
                    preferiti: Vec::new(),
                }))),
                _ => Ok(()),
            };
        }
        None => {
            let (tx, a) = (tx.clone(), annullato.clone());
            std::thread::spawn(move || osserva(tx, a));
        }
    }
    {
        let (st, a, campo) = (stato.clone(), aggiorna.clone(), campo.clone());
        gtk::glib::spawn_future_local(async move {
            while let Some(e) = rx.recv().await {
                {
                    let mut s = st.borrow_mut();
                    match e {
                        Evento::Rete { accesi, codice } => {
                            if !accesi.is_empty() && !s.acceso {
                                s.aperta = ULTIMA;
                            }
                            s.acceso = !accesi.is_empty() || codice.is_some();
                            // Appena compare la schermata del codice, si scrive subito.
                            let appena_aperta = s.codice.is_none() && codice.is_some();
                            s.codice = codice;
                            if appena_aperta {
                                s.aperta = ULTIMA;
                                let c = campo.clone();
                                gtk::glib::idle_add_local_once(move || {
                                    c.grab_focus();
                                });
                            }
                        }
                        Evento::Associato(Ok(t)) => {
                            s.associazione_in_corso = false;
                            s.fatto = Some(t);
                        }
                        Evento::Associato(Err(errore)) => {
                            s.associazione_in_corso = false;
                            s.errore = Some(errore);
                            campo.set_text("");
                        }
                    }
                }
                a();
            }
        });
    }
    {
        let (st, a, tx, annullato) = (stato.clone(), aggiorna.clone(), tx.clone(), annullato.clone());
        campo.connect_changed(move |c| {
            let codice: String = c.text().chars().filter(char::is_ascii_digit).collect();
            if codice != c.text() {
                c.set_text(&codice);
                return;
            }
            let Some(schermata) = st.borrow().codice.clone() else { return };
            if codice.len() != 6 || st.borrow().associazione_in_corso {
                return;
            }
            {
                let mut s = st.borrow_mut();
                s.associazione_in_corso = true;
                s.errore = None;
            }
            a();
            let (tx, annullato) = (tx.clone(), annullato.clone());
            std::thread::spawn(move || {
                let esito = associa(&schermata, &codice).map_err(|e| {
                    eprintln!("[prepara] associazione non riuscita: {e:#}");
                    format!("Codice non accettato ({e}). Riapri «Associa dispositivo con codice di associazione» sul telefono e scrivi il codice nuovo.")
                });
                if !annullato.load(Ordering::SeqCst) {
                    let _ = tx.send(Evento::Associato(esito));
                }
            });
        });
    }
    {
        let (st, f, fatto) = (stato.clone(), finestra.downgrade(), fatto.clone());
        inizia.connect_clicked(move |_| {
            if let Some(t) = st.borrow_mut().fatto.take() {
                fatto(t);
            }
            if let Some(f) = f.upgrade() {
                f.close();
            }
        });
    }
    {
        let (f, fatto) = (finestra.downgrade(), fatto.clone());
        col_cavo.connect_clicked(move |_| {
            let Some(finestra) = f.upgrade() else { return };
            if let Some(app) = finestra.application().and_downcast::<adw::Application>() {
                let fatto = fatto.clone();
                crate::procedura::apri(&app, move |t| fatto(t));
            }
            finestra.close();
        });
    }
    aggiorna();
    finestra.present();
    crate::foto::prendi(&finestra, "prepara");
    finestra
}

fn riquadro_laterale(titolo: &str, contenuto: &impl IsA<gtk::Widget>) -> gtk::Box {
    let r = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(8).css_classes(["laterale-prepara"]).build();
    r.append(&gtk::Label::builder().label(titolo).xalign(0.0).css_classes(["etichetta"]).build());
    r.append(contenuto);
    r
}

/// Apre nel browser la Modalità IA di Google con la domanda già scritta.
fn chiedi_a_google(origine: &gtk::Button, domanda: &str) {
    let codificata: String = domanda
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            b' ' => "+".into(),
            _ => format!("%{b:02X}"),
        })
        .collect();
    let indirizzo = format!("https://www.google.com/search?udm=50&q={codificata}");
    let finestra = origine.root().and_downcast::<gtk::Window>();
    gtk::UriLauncher::new(&indirizzo).launch(finestra.as_ref(), gtk::gio::Cancellable::NONE, |esito| {
        if let Err(e) = esito {
            eprintln!("[prepara] browser non aperto: {e}");
        }
    });
}

/// Guarda la rete: telefoni nuovi col Debug wireless acceso (i già salvati
/// non contano) e la schermata del codice. Manda solo i cambiamenti.
fn osserva(tx: tokio::sync::mpsc::UnboundedSender<Evento>, annullato: Arc<AtomicBool>) {
    let mut ultimo = String::new();
    while !annullato.load(Ordering::SeqCst) {
        let salvati: Vec<String> = Telefoni::carica().map(|t| t.elenco.into_iter().map(|x| x.seriale).collect()).unwrap_or_default();
        let (accesi, codice) = std::thread::scope(|s| {
            let accesi = s.spawn(|| rete::cerca(Duration::from_millis(1500)).unwrap_or_default());
            let codice = s.spawn(|| rete::cerca_abbinamento(Duration::from_millis(1500)).unwrap_or_default());
            (accesi.join().unwrap_or_default(), codice.join().unwrap_or_default())
        });
        let accesi: Vec<_> = accesi.into_iter().filter(|t| !salvati.contains(&t.seriale)).collect();
        let codice = codice.into_iter().next();
        let chiave = format!("{accesi:?}{codice:?}");
        if chiave != ultimo {
            ultimo = chiave;
            if tx.send(Evento::Rete { accesi, codice }).is_err() {
                return;
            }
        }
    }
}

/// Associazione col codice, poi collegamento, autorizzazione senza scadenza e
/// telefono salvato (gira in un thread suo: ci sono chiamate bloccanti).
fn associa(schermata: &TelefonoInRete, codice: &str) -> anyhow::Result<Telefono> {
    let chiave = configurazione::chiave()?;
    let identita = crate::esecutore().block_on(crate::adb::abbina::abbina(schermata.indirizzo, codice, &chiave))?;
    eprintln!("[prepara] associato a {identita}");
    // Il collegamento si fa sulla porta di `_adb-tls-connect`, non su quella del codice.
    let trovato = (0..4)
        .find_map(|_| rete::cerca(Duration::from_secs(3)).ok()?.into_iter().find(|t| t.seriale == schermata.seriale))
        .ok_or_else(|| anyhow::anyhow!("associato, ma il telefono non si annuncia per il collegamento"))?;
    let mut c = Collegamento::wifi(trovato.indirizzo)?;
    let mut telefono = c.descrivi()?;
    // Rendere definitivo: l'autorizzazione non scade più.
    c.shell("settings put global adb_allowed_connection_time 0")?;
    telefono.ultimo_indirizzo = Some(trovato.indirizzo);
    let mut telefoni = Telefoni::carica()?;
    telefoni.registra(telefono.clone());
    telefoni.salva()?;
    Ok(telefono)
}

fn stile() {
    thread_local!(static FATTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    if FATTO.with(|f| f.replace(true)) {
        return;
    }
    let css = gtk::CssProvider::new();
    css.load_from_string(
        "window.prepara .titolo-istruzioni { font-size: 24px; font-weight: 700; }
         window.prepara .spiega { font-size: 14px; opacity: 0.75; }
         window.prepara .voce-elenco { border-radius: 16px; background: rgba(255,255,255,0.62); box-shadow: inset 0 0 0 1px rgba(255,255,255,0.75), 0 1px 4px rgba(0,0,0,0.05); }
         window.prepara.scuro .voce-elenco { background: rgba(255,255,255,0.06); box-shadow: inset 0 0 0 1px rgba(255,255,255,0.08); }
         window.prepara .capo { padding: 12px 16px; }
         window.prepara .capo .num { min-width: 28px; min-height: 28px; border-radius: 14px; background: rgba(0,0,0,0.08); font-size: 13px; font-weight: 700; }
         window.prepara.scuro .capo .num { background: rgba(255,255,255,0.1); }
         window.prepara .voce-elenco.aperta .num { background: #3584e4; color: white; }
         window.prepara .voce-elenco.fatta .num { background: #26a269; color: white; }
         window.prepara .capo .titolo { font-size: 15px; font-weight: 700; }
         window.prepara .voce-elenco.fatta .capo .titolo { opacity: 0.6; }
         window.prepara .capo .sotto { font-size: 12.5px; opacity: 0.6; }
         window.prepara .segno { font-size: 12px; font-weight: 600; }
         window.prepara .segno.visto { color: #1a7a4c; }
         window.prepara .segno.attesa { color: #1c5aa8; }
         window.prepara .segno.mano { opacity: 0.6; }
         window.prepara.scuro .segno.visto { color: #8ff0a4; }
         window.prepara.scuro .segno.attesa { color: #99c1f1; }
         window.prepara .dentro { padding: 0 16px 14px 58px; }
         window.prepara .perche { font-size: 13.5px; opacity: 0.75; }
         window.prepara .modo { border-radius: 12px; padding: 10px 14px; background: rgba(255,255,255,0.8); font-size: 13.5px; }
         window.prepara.scuro .modo { background: rgba(255,255,255,0.08); }
         window.prepara .etichetta { font-size: 11px; font-weight: 700; letter-spacing: 1px; opacity: 0.6; }
         window.prepara entry.codice { font-size: 26px; font-weight: 700; letter-spacing: 8px; min-height: 52px; border-radius: 12px; }
         window.prepara .nota { font-size: 12.5px; opacity: 0.7; }
         window.prepara .fatto-tutto { border-radius: 16px; padding: 18px 20px; background: rgba(38,162,105,0.14); }
         window.prepara .laterale-prepara { border-radius: 16px; padding: 14px 16px; background: rgba(255,255,255,0.62); box-shadow: inset 0 0 0 1px rgba(255,255,255,0.75); font-size: 13px; }
         window.prepara.scuro .laterale-prepara { background: rgba(255,255,255,0.06); box-shadow: inset 0 0 0 1px rgba(255,255,255,0.08); }",
    );
    if let Some(schermo) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&schermo, &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}
