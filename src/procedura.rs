//! «Aggiungi un telefono»: la procedura guidata del primo collegamento col
//! cavo (SPECIFICATION §5.2; mockup `mockup/proposals/procedure-*`).
//!
//! Nessun «Avanti»: un controllo in sottofondo guarda il cavo ogni secondo e
//! avanza da solo. Solo i passaggi interni del Debug USB (7 tocchi, Opzioni
//! sviluppatore, interruttore) non si vedono dal PC: si sfogliano a mano.
//! Il telefono disegnato mostra la schermata da cercare col punto evidenziato;
//! le istruzioni sono per famiglia di marca (`data/instructions.toml`).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use adw::prelude::*;
use serde::Deserialize;

use crate::configurazione::{Telefoni, Telefono};
use crate::telefono::Collegamento as Cavo;
use crate::{rete, t, usb};

/// Istruzioni di una famiglia di marca.
#[derive(Debug, Clone, Deserialize)]
pub struct Famiglia {
    pub nome: String,
    pub marche: Vec<String>,
    pub verificata: bool,
    pub percorso_build: Vec<String>,
    pub voce_build: String,
    pub percorso_debug: Vec<String>,
    /// Interruttore in più, dopo «Consenti» (Xiaomi).
    #[serde(default)]
    pub sicurezza: Option<String>,
    /// Cosa fare se la voce «Debug USB» è grigia (bloccata dal telefono).
    #[serde(default)]
    pub grigio: Option<String>,
    /// Interruttore da spegnere prima di tutto, perché dalla fabbrica può
    /// essere acceso e bloccare il Debug USB (Samsung: Blocco automatico).
    #[serde(default)]
    pub prima: Option<Preliminare>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Preliminare {
    pub percorso: Vec<String>,
    pub voce: String,
}

impl Famiglia {
    /// Pagine del telefono disegnato per il Debug USB.
    fn pagine(&self) -> usize {
        if self.prima.is_some() { 4 } else { 3 }
    }
}

#[derive(Deserialize)]
struct Istruzioni {
    famiglia: Vec<Famiglia>,
}

/// La famiglia del produttore letto dal cavo; l'ultima vale per tutti gli altri.
pub fn famiglia(produttore: &str) -> Famiglia {
    famiglia_in(produttore, crate::lingua::attuale())
}

/// Come [`famiglia`], con le istruzioni della lingua data (in inglese, i nomi
/// inglesi delle impostazioni del telefono: SPECIFICATION §15.1).
pub fn famiglia_in(produttore: &str, lingua: crate::lingua::Lingua) -> Famiglia {
    let testo = match lingua {
        crate::lingua::Lingua::Italiano => include_str!("../data/instructions.toml"),
        crate::lingua::Lingua::Inglese => include_str!("../data/en/instructions.toml"),
    };
    let tutte: Istruzioni = toml::from_str(testo).expect("istruzioni per marca valide");
    let p = produttore.to_lowercase();
    let generica = tutte.famiglia.last().cloned().expect("almeno una famiglia");
    tutte.famiglia.into_iter().find(|f| f.marche.iter().any(|m| p.contains(m.as_str()))).unwrap_or(generica)
}

/// A che punto è il telefono, visto dal PC.
#[derive(Debug, Clone)]
enum Passo {
    Collega,
    /// Cavo collegato, Debug USB spento.
    DebugUsb(Famiglia),
    /// In «Solo ricarica» Linux non dà il permesso senza root.
    TrasferimentoFile,
    Consenti,
    /// Xiaomi: manca «Debug USB (impostazioni di sicurezza)».
    Sicurezza(Famiglia),
    ReteWifi,
    Verifica,
    /// Autorizzato senza «Consenti sempre»: il Wi-Fi viene rifiutato.
    ConsentiSempre,
    NonInRete,
    Fatto(Telefono),
}

impl Passo {
    /// Riga dell'elenco dei passi a sinistra.
    fn numero(&self) -> usize {
        match self {
            Passo::Collega | Passo::TrasferimentoFile => 0,
            Passo::DebugUsb(_) => 1,
            Passo::Consenti | Passo::Sicurezza(_) | Passo::ConsentiSempre => 2,
            Passo::ReteWifi | Passo::Verifica | Passo::NonInRete => 3,
            Passo::Fatto(_) => 4,
        }
    }
}

/// Il controllo in sottofondo: gira in un thread suo (il cavo si usa con
/// chiamate bloccanti) e manda i passi all'interfaccia.
fn osserva(tx: tokio::sync::mpsc::UnboundedSender<Passo>, annullato: Arc<AtomicBool>) {
    let mut ultimo = String::new();
    let manda = |p: Passo, ultimo: &mut String| {
        let chiave = format!("{p:?}");
        if chiave != *ultimo {
            *ultimo = chiave;
            let _ = tx.send(p);
        }
    };
    let aspetta = |secondi: u64| std::thread::sleep(Duration::from_secs(secondi));
    'da_capo: loop {
        if annullato.load(Ordering::SeqCst) {
            return;
        }
        let Some(t) = usb::telefoni().into_iter().next() else {
            manda(Passo::Collega, &mut ultimo);
            aspetta(1);
            continue;
        };
        let fam = famiglia(&t.produttore);
        if t.stato == usb::Stato::SoloRicarica {
            manda(Passo::TrasferimentoFile, &mut ultimo);
            aspetta(1);
            continue;
        }
        if t.stato == usb::Stato::DebugSpento {
            manda(Passo::DebugUsb(fam), &mut ultimo);
            aspetta(1);
            continue;
        }
        manda(Passo::Consenti, &mut ultimo);
        let mut cavo = match Cavo::usb(t.vendor, t.prodotto) {
            Ok(c) => c,
            Err(e) => {
                let testo = format!("{e:#}").to_lowercase();
                if t.stato == usb::Stato::DebugAttivoSoloRicarica && (testo.contains("access") || testo.contains("permission")) {
                    manda(Passo::TrasferimentoFile, &mut ultimo);
                }
                aspetta(2);
                continue;
            }
        };
        let Ok(mut telefono) = cavo.descrivi() else {
            aspetta(2);
            continue;
        };
        // Xiaomi: senza l'interruttore di sicurezza i tocchi vengono rifiutati.
        if fam.sicurezza.is_some() {
            loop {
                match cavo.shell("input keyevent 0") {
                    Ok(u) if !u.contains("SecurityException") => break,
                    Err(e) if !format!("{e:#}").contains("SecurityException") && !format!("{e:#}").contains("INJECT_EVENTS") => break,
                    _ => {}
                }
                manda(Passo::Sicurezza(fam.clone()), &mut ultimo);
                aspetta(2);
                if annullato.load(Ordering::SeqCst) {
                    return;
                }
                if usb::telefoni().is_empty() {
                    continue 'da_capo;
                }
            }
        }
        manda(Passo::ReteWifi, &mut ultimo);
        if !matches!(cavo.prepara_wifi(), Ok(true)) {
            continue;
        }
        manda(Passo::Verifica, &mut ultimo);
        let trovato = (0..3).find_map(|_| {
            rete::cerca(Duration::from_secs(4)).ok()?.into_iter().find(|x| x.seriale == telefono.seriale)
        });
        let Some(trovato) = trovato else {
            manda(Passo::NonInRete, &mut ultimo);
            aspetta(3);
            continue;
        };
        match Cavo::wifi(trovato.indirizzo).and_then(|mut w| w.shell("echo ok")) {
            Ok(_) => {}
            Err(e) if format!("{e:#}").contains("CertificateUnknown") => {
                // Si riparte quando il cavo è stato staccato e ricollegato.
                manda(Passo::ConsentiSempre, &mut ultimo);
                while !usb::telefoni().is_empty() {
                    if annullato.load(Ordering::SeqCst) {
                        return;
                    }
                    aspetta(1);
                }
                continue;
            }
            Err(_) => {
                manda(Passo::NonInRete, &mut ultimo);
                aspetta(3);
                continue;
            }
        }
        telefono.ultimo_indirizzo = Some(trovato.indirizzo);
        let salvato = Telefoni::carica().and_then(|mut t| {
            t.registra(telefono.clone());
            t.salva()
        });
        if let Err(e) = salvato {
            eprintln!("[procedura] telefono non salvato: {e:#}");
            aspetta(3);
            continue;
        }
        let _ = tx.send(Passo::Fatto(telefono));
        return;
    }
}

/// I passi dell'elenco a sinistra (titolo, sottotitolo), nella lingua in uso.
fn passi() -> [(&'static str, &'static str); 5] {
    [
        (t!("Collega il cavo USB"), t!("Telefono e PC, con un cavo che trasmette dati")),
        (t!("Attiva il Debug USB"), t!("Opzioni sviluppatore › Debug USB")),
        (t!("Consenti il collegamento"), t!("Spunta «Consenti sempre da questo computer»")),
        (t!("Consenti la rete Wi-Fi"), t!("Solo la prima volta per ogni rete")),
        (t!("Fatto"), t!("Puoi scollegare il cavo")),
    ]
}

/// Apre la procedura; `fatto` riceve il telefono appena salvato.
pub fn apri(app: &adw::Application, fatto: impl Fn(Telefono) + 'static) -> adw::ApplicationWindow {
    crate::cassetto::stile();
    stile();

    // A sinistra i passi.
    let colonna_passi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(4).width_request(250).build();
    let mut righe = Vec::new();
    for (i, (titolo, sotto)) in passi().iter().enumerate() {
        let numero = gtk::Label::builder().label((i + 1).to_string()).valign(gtk::Align::Start).css_classes(["num"]).build();
        let testi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).build();
        testi.append(&gtk::Label::builder().label(*titolo).xalign(0.0).css_classes(["titolo"]).build());
        testi.append(&gtk::Label::builder().label(*sotto).xalign(0.0).wrap(true).css_classes(["sotto"]).build());
        let riga = gtk::Box::builder().spacing(12).css_classes(["passo"]).build();
        riga.append(&numero);
        riga.append(&testi);
        colonna_passi.append(&riga);
        righe.push((riga, numero));
    }

    // Al centro le istruzioni.
    let titolo = gtk::Label::builder().xalign(0.0).wrap(true).css_classes(["titolo-istruzioni"]).build();
    let spiega = gtk::Label::builder().xalign(0.0).wrap(true).css_classes(["spiega"]).build();
    let elenco = gtk::Label::builder().xalign(0.0).wrap(true).use_markup(true).css_classes(["elenco-istruzioni"]).build();
    let attesa = gtk::Label::builder().xalign(0.0).wrap(true).css_classes(["attesa"]).build();
    // Fermi al passo 1 da un po': cosa vede il PC, in parole semplici (chi
    // prova da solo non deve indagare: basta una foto della finestra).
    let diagnosi = gtk::Label::builder().xalign(0.0).wrap(true).selectable(true).visible(false).css_classes(["diagnosi"]).build();
    let inizia = gtk::Button::builder().label(t!("Inizia a usare il telefono")).halign(gtk::Align::Start).css_classes(["suggested-action", "pill"]).visible(false).build();
    let istruzioni = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(14).hexpand(true).css_classes(["istruzioni"]).build();
    for w in [titolo.upcast_ref::<gtk::Widget>(), spiega.upcast_ref(), elenco.upcast_ref(), attesa.upcast_ref(), diagnosi.upcast_ref(), inizia.upcast_ref()] {
        istruzioni.append(w);
    }

    // A destra il telefono disegnato, con le frecce per le pagine del Debug USB.
    let schermo = gtk::Box::builder().orientation(gtk::Orientation::Vertical).css_classes(["schermo"]).overflow(gtk::Overflow::Hidden).build();
    let cornice = gtk::Box::builder().css_classes(["telefono"]).build();
    cornice.append(&schermo);
    schermo.set_hexpand(true);
    let proporzioni = gtk::AspectFrame::builder().ratio(0.475).obey_child(false).yalign(0.0).child(&cornice).vexpand(true).build();
    let indietro = gtk::Button::builder().icon_name("go-previous-symbolic").css_classes(["flat", "circular"]).build();
    let avanti = gtk::Button::builder().icon_name("go-next-symbolic").css_classes(["flat", "circular"]).build();
    let pagina = gtk::Label::new(None);
    let frecce = gtk::Box::builder().spacing(12).halign(gtk::Align::Center).visible(false).build();
    frecce.append(&indietro);
    frecce.append(&pagina);
    frecce.append(&avanti);
    let lato = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(8).width_request(290).hexpand(false).build();
    lato.append(&proporzioni);
    lato.append(&frecce);

    let corpo = gtk::Box::builder().spacing(24).margin_start(24).margin_end(24).margin_bottom(24).margin_top(8).build();
    corpo.append(&colonna_passi);
    corpo.append(&istruzioni);
    corpo.append(&lato);
    let barra = adw::HeaderBar::new();
    barra.set_title_widget(Some(&gtk::Label::builder().label(t!("Aggiungi un telefono")).css_classes(["titolo-app"]).build()));
    let vista = adw::ToolbarView::new();
    vista.add_top_bar(&barra);
    vista.set_content(Some(&corpo));
    let finestra = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(1060)
        .default_height(700)
        .title(t!("Aggiungi un telefono"))
        .content(&vista)
        .css_classes(["phonestra-drawer", "procedura"])
        .build();
    crate::cassetto::segui_tema(&finestra);

    // Pagine del Debug USB (0..3) e famiglia attuale.
    let stato: std::rc::Rc<std::cell::RefCell<(usize, Option<Famiglia>)>> = std::rc::Rc::default();
    let disegna_debug = {
        let (schermo, pagina, stato, elenco) = (schermo.clone(), pagina.clone(), stato.clone(), elenco.clone());
        move || {
            let (n, fam) = stato.borrow().clone();
            let Some(f) = fam else { return };
            pagina.set_label(&t!("{} di {}", n + 1, f.pagine()));
            let ultimo = |v: &[String]| v.last().cloned().unwrap_or_default();
            // Col passo preliminare le altre pagine scalano di uno.
            let contenuto = match (&f.prima, n) {
                (Some(p), 0) => schermata_impostazioni(
                    &ultimo(&p.percorso),
                    &[(p.voce.as_str(), t!("acceso")), (t!("App non autorizzate"), ""), (t!("Comandi tramite cavo USB"), "")],
                    0,
                    t!("spegni"),
                ),
                (prima, n) => match n - usize::from(prima.is_some()) {
                0 => schermata_impostazioni(
                    &ultimo(&f.percorso_build),
                    &[(t!("Versione Android"), "16"), (f.voce_build.as_str(), ""), (t!("Versione kernel"), "…")],
                    1,
                    t!("tocca 7 volte"),
                ),
                1 => schermata_impostazioni(
                    t!("Impostazioni"),
                    &[(t!("Informazioni sul telefono"), ""), (ultimo(&f.percorso_debug).as_str(), t!("compare dopo i 7 tocchi")), (t!("Batteria"), "")],
                    1,
                    t!("apri"),
                ),
                _ => schermata_impostazioni(
                    &ultimo(&f.percorso_debug),
                    &[(t!("Debug USB"), t!("spento")), (t!("Debug wireless"), "")],
                    0,
                    t!("accendi"),
                ),
                },
            };
            sostituisci(&schermo, &contenuto);
            let percorso = |v: &[String]| v.join(" › ");
            let preliminare = f.prima.as_ref().map_or(String::new(), |p| {
                t!(
                    "1. Apri <b>{}</b> e, se è acceso, <b>spegnilo</b>: bloccherebbe il Debug USB.\n\n",
                    gtk::glib::markup_escape_text(&percorso(&p.percorso))
                )
            });
            let d = usize::from(f.prima.is_some());
            elenco.set_markup(&t!(
                "{}{}. Apri <b>{}</b>.\n\n{}. Tocca <b>7 volte</b> «{}» e inserisci il PIN: compaiono le <b>Opzioni sviluppatore</b>.\n\n{}. Apri <b>{}</b> e accendi <b>Debug USB</b>.{}{}",
                preliminare,
                1 + d,
                gtk::glib::markup_escape_text(&percorso(&f.percorso_build)),
                2 + d,
                gtk::glib::markup_escape_text(&f.voce_build),
                3 + d,
                gtk::glib::markup_escape_text(&percorso(&f.percorso_debug)),
                f.grigio.as_ref().map_or(String::new(), |g| t!(
                    "\n\n<b>La voce è grigia?</b> {}",
                    gtk::glib::markup_escape_text(g)
                )),
                if f.verificata {
                    String::new()
                } else {
                    t!(
                        "\n\n<i>Percorso non ancora verificato per {}: se non trovi la voce, cercala con la ricerca delle Impostazioni.</i>",
                        gtk::glib::markup_escape_text(&f.nome)
                    )
                }
            ));
        }
    };
    {
        let (st, d) = (stato.clone(), disegna_debug.clone());
        indietro.connect_clicked(move |_| {
            let n = st.borrow().0;
            st.borrow_mut().0 = n.saturating_sub(1);
            d();
        });
        let (st, d) = (stato.clone(), disegna_debug.clone());
        avanti.connect_clicked(move |_| {
            let (n, ultima) = {
                let s = st.borrow();
                (s.0, s.1.as_ref().map_or(2, |f| f.pagine() - 1))
            };
            st.borrow_mut().0 = (n + 1).min(ultima);
            d();
        });
    }

    let mostra = {
        let (schermo, frecce, titolo, spiega, elenco, attesa, inizia) =
            (schermo.clone(), frecce.clone(), titolo.clone(), spiega.clone(), elenco.clone(), attesa.clone(), inizia.clone());
        move |p: &Passo| {
            let attuale = p.numero();
            for (i, (riga, numero)) in righe.iter().enumerate() {
                for c in ["fatto", "adesso"] {
                    riga.remove_css_class(c);
                }
                if i < attuale || matches!(p, Passo::Fatto(_)) {
                    riga.add_css_class("fatto");
                    numero.set_label("✓");
                } else {
                    numero.set_label(&(i + 1).to_string());
                    if i == attuale {
                        riga.add_css_class("adesso");
                    }
                }
            }
            frecce.set_visible(matches!(p, Passo::DebugUsb(_)));
            inizia.set_visible(matches!(p, Passo::Fatto(_)));
            elenco.set_visible(matches!(p, Passo::DebugUsb(_) | Passo::Sicurezza(_)));
            attesa.set_label(t!("In attesa del telefono… me ne accorgo da solo."));
            attesa.set_visible(!matches!(p, Passo::Fatto(_)));
            let (tit, s, disegno): (&str, String, Option<gtk::Box>) = match p {
                Passo::Collega => (
                    t!("Collega il telefono al PC"),
                    t!("Usa un cavo USB che trasmette dati (alcuni cavi servono solo a ricaricare). Il telefono deve essere acceso e sbloccato.\n\nSe non succede niente: apri la tendina del telefono, tocca la notifica «USB» e scegli «Trasferimento file»; se il telefono chiede di consentire l'accesso ai dati, tocca «Consenti».").into(),
                    Some(schermata_messaggio(t!("Collega il cavo USB"))),
                ),
                Passo::DebugUsb(f) => {
                    stato.borrow_mut().1 = Some(f.clone());
                    attesa.set_label(t!("Sfoglia i passaggi con le frecce sotto il telefono disegnato. Quando il Debug USB è acceso me ne accorgo da solo e vado avanti."));
                    (
                        t!("Attiva il Debug USB"),
                        t!("Android non permette ai programmi di attivarlo: va fatto a mano, una volta sola. Istruzioni per {}.", f.nome),
                        None,
                    )
                }
                Passo::TrasferimentoFile => (
                    t!("Scegli «Trasferimento file»"),
                    t!("Il telefono è in «Solo ricarica»: così il PC non può parlargli. Apri la tendina del telefono, tocca la notifica «USB» e scegli «Trasferimento file». Se chiede di consentire l'accesso ai dati, tocca «Consenti».").into(),
                    Some(schermata_impostazioni(t!("Usa USB per"), &[(t!("Trasferimento file"), ""), (t!("Solo ricarica"), "")], 0, t!("scegli"))),
                ),
                Passo::Consenti => (
                    t!("Consenti il collegamento"),
                    t!("Sul telefono è comparsa una richiesta. Prima di toccare «Consenti» spunta la casella: senza, il cavo funziona ma il Wi-Fi no.").into(),
                    Some(schermata_dialogo(t!("Consentire il debug USB?"), t!("Impronta della chiave RSA del computer"), t!("Consenti sempre da questo computer"))),
                ),
                Passo::Sicurezza(f) => {
                    let voce = f.sicurezza.clone().unwrap_or_default();
                    elenco.set_markup(&t!("Apri <b>{}</b> e accendi <b>{}</b>.\n\nServono l'accesso all'account Xiaomi e internet; su alcune versioni anche una SIM inserita.",
                        gtk::glib::markup_escape_text(&f.percorso_debug.join(" › ")),
                        gtk::glib::markup_escape_text(&voce)
                    ));
                    (
                        t!("Un passaggio in più per Xiaomi"),
                        t!("Sui telefoni Xiaomi il Debug USB normale non basta: senza questo interruttore le app si vedono ma non rispondono a mouse e tastiera.").into(),
                        Some(schermata_impostazioni(f.percorso_debug.last().map_or(t!("Opzioni sviluppatore"), |s| s.as_str()), &[(voce.as_str(), t!("spento"))], 0, t!("accendi"))),
                    )
                }
                Passo::ReteWifi => (
                    t!("Consenti la rete Wi-Fi"),
                    t!("Phonestra accende il Debug wireless. La prima volta su ogni rete il telefono chiede il permesso: tocca «Consenti».").into(),
                    Some(schermata_dialogo(t!("Consentire il debug wireless su questa rete?"), t!("Nome della rete Wi-Fi del telefono"), t!("Consenti sempre su questa rete"))),
                ),
                Passo::Verifica => (
                    t!("Provo il collegamento Wi-Fi…"),
                    t!("Cerco il telefono in rete e provo a collegarmi senza cavo.").into(),
                    Some(schermata_messaggio(t!("Verifica in corso…"))),
                ),
                Passo::ConsentiSempre => (
                    t!("Manca «Consenti sempre»"),
                    t!("Il telefono ha autorizzato il PC solo per questa volta, e così il Wi-Fi viene rifiutato. Scollega e ricollega il cavo; quando compare la richiesta, spunta «Consenti sempre da questo computer».").into(),
                    Some(schermata_dialogo(t!("Consentire il debug USB?"), t!("Impronta della chiave RSA del computer"), t!("Consenti sempre da questo computer"))),
                ),
                Passo::NonInRete => (
                    t!("Non trovo il telefono in rete"),
                    t!("Controlla che il telefono sia sulla stessa rete Wi-Fi del PC (non la rete ospiti). Continuo a provare da solo.").into(),
                    Some(schermata_messaggio(t!("Stessa rete Wi-Fi del PC?"))),
                ),
                Passo::Fatto(tel) => (
                    t!("Fatto!"),
                    t!("Il {} è configurato e si collega via Wi-Fi: puoi scollegare il cavo.", tel.nome),
                    Some(schermata_messaggio(t!("✓ Configurato"))),
                ),
            };
            titolo.set_label(tit);
            spiega.set_label(&s);
            match disegno {
                Some(d) => sostituisci(&schermo, &d),
                None => {
                    stato.borrow_mut().0 = 0;
                    disegna_debug();
                }
            }
        }
    };

    let annullato = Arc::new(AtomicBool::new(false));
    {
        let a = annullato.clone();
        finestra.connect_close_request(move |_| {
            a.store(true, Ordering::SeqCst);
            gtk::glib::Propagation::Proceed
        });
    }
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    // Per le prove dell'interfaccia: `PHONESTRA_PROVA_PASSO=debug|consenti|xiaomi|wifi|fatto`
    // mostra quel passo senza guardare il cavo.
    match std::env::var("PHONESTRA_PROVA_PASSO").ok().and_then(|p| passo_di_prova(&p)) {
        Some(p) => {
            let _ = tx.send(p);
        }
        None => {
            let a = annullato.clone();
            std::thread::spawn(move || osserva(tx, a));
        }
    }
    let telefono_fatto: std::rc::Rc<std::cell::RefCell<Option<Telefono>>> = std::rc::Rc::default();
    // Da quando si è al passo 1 (None: a un altro passo).
    let al_passo_uno: std::rc::Rc<std::cell::Cell<Option<std::time::Instant>>> = std::rc::Rc::default();
    {
        let (t, uno, d) = (telefono_fatto.clone(), al_passo_uno.clone(), diagnosi.clone());
        gtk::glib::spawn_future_local(async move {
            while let Some(p) = rx.recv().await {
                if let Passo::Fatto(tel) = &p {
                    *t.borrow_mut() = Some(tel.clone());
                }
                if matches!(p, Passo::Collega) {
                    if uno.get().is_none() {
                        uno.set(Some(std::time::Instant::now()));
                    }
                } else {
                    uno.set(None);
                    d.set_visible(false);
                }
                mostra(&p);
            }
        });
    }
    {
        let (uno, d) = (al_passo_uno.clone(), diagnosi.downgrade());
        gtk::glib::timeout_add_local(Duration::from_secs(2), move || {
            let Some(d) = d.upgrade() else { return gtk::glib::ControlFlow::Break };
            if uno.get().is_some_and(|t| t.elapsed() >= Duration::from_secs(20)) {
                d.set_markup(&t!("<b>Cosa vede il PC</b>\n{}", gtk::glib::markup_escape_text(&testo_diagnosi(&crate::usb::tutti()))));
                d.set_visible(true);
            }
            gtk::glib::ControlFlow::Continue
        });
    }
    {
        let f = finestra.downgrade();
        inizia.connect_clicked(move |_| {
            if let Some(t) = telefono_fatto.borrow_mut().take() {
                fatto(t);
            }
            if let Some(f) = f.upgrade() {
                f.close();
            }
        });
    }
    finestra.present();
    crate::foto::prendi(&finestra, "procedura");
    finestra
}

fn passo_di_prova(nome: &str) -> Option<Passo> {
    Some(match nome {
        "debug" => Passo::DebugUsb(famiglia("SAMSUNG")),
        "consenti" => Passo::Consenti,
        "xiaomi" => Passo::Sicurezza(famiglia("Xiaomi")),
        "wifi" => Passo::ReteWifi,
        "fatto" => Passo::Fatto(Telefono {
            seriale: "prova".into(),
            nome: "Telefono di prova".into(),
            modello: String::new(),
            android: String::new(),
            ultimo_indirizzo: None,
            spegnimento_originale: None,
            volume_originale: None,
            preferiti: Vec::new(),
        }),
        _ => return None,
    })
}

fn sostituisci(contenitore: &gtk::Box, nuovo: &impl IsA<gtk::Widget>) {
    while let Some(figlio) = contenitore.first_child() {
        contenitore.remove(&figlio);
    }
    contenitore.append(nuovo);
}

/// Una schermata delle Impostazioni di Android: titolo e righe, una
/// evidenziata con l'etichetta `azione` («tocca 7 volte», «accendi»).
/// Il riquadro «Cosa vede il PC» del passo 1.
fn testo_diagnosi(dispositivi: &[String]) -> String {
    let telefono = dispositivi.iter().any(|d| d.ends_with("← telefono"));
    let mut t = String::new();
    t += if telefono {
        t!("Il PC vede il telefono, ma il telefono non offre né file né debug. Sbloccalo, apri la tendina, tocca la notifica «USB» e scegli «Trasferimento file».\n")
    } else {
        t!("Il PC non riceve niente dal telefono. Prova, in quest'ordine:\n\
         1. un altro cavo: molti servono solo a ricaricare;\n\
         2. un'altra porta USB del PC, senza hub né prolunghe;\n\
         3. su Samsung: Impostazioni › Sicurezza e privacy › Blocco automatico: spegnilo, comprese le «Restrizioni massime»;\n\
         4. telefono acceso e sbloccato mentre colleghi il cavo.\n")
    };
    t += t!("\nDispositivi USB collegati:\n");
    if dispositivi.is_empty() {
        t += t!("(nessuno)");
    }
    t += &dispositivi.iter().map(|d| format!("• {d}")).collect::<Vec<_>>().join("\n");
    t
}

fn schermata_impostazioni(titolo: &str, righe: &[(&str, &str)], evidenziata: usize, azione: &str) -> gtk::Box {
    let s = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(6).vexpand(true).css_classes(["schermo-chiaro"]).build();
    s.append(&gtk::Label::builder().label(format!("‹ {titolo}")).xalign(0.0).wrap(true).max_width_chars(18).css_classes(["titolo-imp"]).build());
    for (i, (voce, valore)) in righe.iter().enumerate() {
        let testi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).hexpand(true).build();
        testi.append(&gtk::Label::builder().label(*voce).xalign(0.0).wrap(true).max_width_chars(18).css_classes(["voce-imp"]).build());
        if !valore.is_empty() {
            testi.append(&gtk::Label::builder().label(*valore).xalign(0.0).css_classes(["valore-imp"]).build());
        }
        let riga = gtk::Box::builder().spacing(6).css_classes(["riga-imp"]).build();
        riga.append(&testi);
        if i == evidenziata {
            riga.add_css_class("evidenziata");
            riga.append(&gtk::Label::builder().label(azione).valign(gtk::Align::Center).css_classes(["azione-imp"]).build());
        }
        s.append(&riga);
    }
    s
}

/// Una richiesta di Android sullo schermo scuro, con la casella da spuntare
/// evidenziata.
fn schermata_dialogo(titolo: &str, testo: &str, spunta: &str) -> gtk::Box {
    let d = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).valign(gtk::Align::Center).vexpand(true).css_classes(["dialogo-tel"]).build();
    d.append(&gtk::Label::builder().label(titolo).xalign(0.0).wrap(true).max_width_chars(18).css_classes(["titolo-imp"]).build());
    d.append(&gtk::Label::builder().label(testo).xalign(0.0).wrap(true).max_width_chars(18).css_classes(["valore-imp"]).build());
    let casella = gtk::Box::builder().spacing(8).css_classes(["spunta"]).build();
    casella.append(&gtk::Label::builder().label("✓").css_classes(["casella"]).build());
    casella.append(&gtk::Label::builder().label(spunta).xalign(0.0).wrap(true).max_width_chars(18).css_classes(["voce-imp"]).build());
    d.append(&casella);
    let pulsanti = gtk::Box::builder().homogeneous(true).css_classes(["pulsanti-tel"]).build();
    pulsanti.append(&gtk::Label::new(Some(t!("Annulla"))));
    pulsanti.append(&gtk::Label::new(Some(t!("Consenti"))));
    d.append(&pulsanti);
    let s = gtk::Box::builder().orientation(gtk::Orientation::Vertical).vexpand(true).css_classes(["schermo-scuro"]).build();
    s.append(&d);
    s
}

/// Un messaggio al centro del telefono disegnato.
fn schermata_messaggio(testo: &str) -> gtk::Box {
    let s = gtk::Box::builder().orientation(gtk::Orientation::Vertical).vexpand(true).build();
    s.append(&gtk::Label::builder().label(testo).wrap(true).justify(gtk::Justification::Center).vexpand(true).css_classes(["titolo-velo"]).build());
    s
}

fn stile() {
    thread_local!(static FATTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    if FATTO.with(|f| f.replace(true)) {
        return;
    }
    let css = gtk::CssProvider::new();
    css.load_from_string(
        "window.procedura .passo { padding: 10px 12px; border-radius: 12px; opacity: 0.6; }
         window.procedura .passo .num { min-width: 26px; min-height: 26px; border-radius: 13px; background: rgba(0,0,0,0.07); font-size: 12px; font-weight: 700; }
         window.procedura.scuro .passo .num { background: rgba(255,255,255,0.1); }
         window.procedura .passo .titolo { font-weight: 700; }
         window.procedura .passo .sotto { font-size: 12px; }
         window.procedura .passo.fatto { opacity: 1; }
         window.procedura .passo.fatto .num { background: #26a269; color: white; }
         window.procedura .passo.adesso { opacity: 1; background: rgba(255,255,255,0.75); box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
         window.procedura.scuro .passo.adesso { background: rgba(255,255,255,0.08); }
         window.procedura .passo.adesso .num { background: #3584e4; color: white; }
         window.procedura .istruzioni { border-radius: 18px; padding: 26px 28px; background: rgba(255,255,255,0.62); box-shadow: inset 0 0 0 1px rgba(255,255,255,0.75); }
         window.procedura.scuro .istruzioni { background: rgba(255,255,255,0.06); box-shadow: inset 0 0 0 1px rgba(255,255,255,0.08); }
         window.procedura .titolo-istruzioni { font-size: 22px; font-weight: 700; }
         window.procedura .spiega { font-size: 14px; opacity: 0.75; }
         window.procedura .elenco-istruzioni { font-size: 14.5px; }
         window.procedura .attesa { padding: 12px 14px; border-radius: 12px; background: rgba(53,132,228,0.12); color: #1c5aa8; font-weight: 600; }
         window.procedura.scuro .attesa { color: #99c1f1; }
         window.procedura .diagnosi { padding: 12px 14px; border-radius: 12px; background: rgba(229,165,10,0.14); font-size: 0.92em; }
         window.procedura .schermo { color: white; }
         window.procedura .schermo-chiaro { background: #f4f4f6; color: #222; padding: 44px 14px 14px 14px; }
         window.procedura .schermo-scuro { background: rgba(20,20,30,0.85); padding: 12px; }
         window.procedura .titolo-imp { font-size: 15px; font-weight: 700; margin: 0 2px 8px 2px; }
         window.procedura .riga-imp { background: white; border-radius: 12px; padding: 9px 12px; }
         window.procedura .riga-imp.evidenziata { box-shadow: 0 0 0 2.5px #3584e4, 0 0 0 7px rgba(53,132,228,0.2); }
         window.procedura .voce-imp { font-size: 12.5px; font-weight: 600; }
         window.procedura .valore-imp { font-size: 11px; opacity: 0.65; }
         window.procedura .azione-imp { background: #3584e4; color: white; font-size: 10px; font-weight: 700; padding: 3px 7px; border-radius: 8px; }
         window.procedura .dialogo-tel { background: white; color: #222; border-radius: 22px; padding: 18px 16px 12px 16px; }
         window.procedura .spunta { padding: 8px; border-radius: 10px; box-shadow: 0 0 0 2.5px #3584e4, 0 0 0 7px rgba(53,132,228,0.2); }
         window.procedura .casella { background: #3584e4; color: white; border-radius: 4px; min-width: 18px; min-height: 18px; font-size: 12px; }
         window.procedura .pulsanti-tel { color: #3584e4; font-weight: 700; margin-top: 6px; }",
    );
    if let Some(schermo) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&schermo, &css, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

#[cfg(test)]
mod prove {
    use super::*;

    use crate::lingua::Lingua;

    #[test]
    fn famiglie_dal_produttore() {
        assert_eq!(famiglia_in("SAMSUNG", Lingua::Italiano).nome, "Samsung");
        assert!(famiglia_in("Xiaomi", Lingua::Italiano).sicurezza.is_some());
        assert_eq!(famiglia_in("Sconosciuto S.p.A.", Lingua::Italiano).nome, "Altri telefoni");
    }

    /// Le istruzioni inglesi hanno le stesse famiglie, marche e passi.
    #[test]
    fn istruzioni_nelle_due_lingue() {
        let leggi = |testo: &str| toml::from_str::<Istruzioni>(testo).unwrap().famiglia;
        let it = leggi(include_str!("../data/instructions.toml"));
        let en = leggi(include_str!("../data/en/instructions.toml"));
        assert_eq!(it.len(), en.len());
        for (a, b) in it.iter().zip(&en) {
            assert_eq!(a.marche, b.marche);
            assert_eq!(a.verificata, b.verificata);
            assert_eq!(a.percorso_build.len(), b.percorso_build.len());
            assert_eq!(a.percorso_debug.len(), b.percorso_debug.len());
            assert_eq!(a.sicurezza.is_some(), b.sicurezza.is_some());
            assert_eq!(a.grigio.is_some(), b.grigio.is_some());
            assert_eq!(a.prima.as_ref().map(|p| p.percorso.len()), b.prima.as_ref().map(|p| p.percorso.len()));
        }
    }
}
