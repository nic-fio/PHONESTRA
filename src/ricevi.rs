//! «Ricevi file…» (SPECIFICHE §11.1; mockup `mockup/proposte/ricevi-file.html`):
//! una finestra per navigare nella memoria condivisa del telefono e scegliere
//! i file da copiare sul PC.
//!
//! - a sinistra i posti: Recenti (gli ultimi file delle cartelle solite),
//!   Fotocamera, Screenshot, Download, WhatsApp, Documenti, poi «Memoria del
//!   telefono» e, se c'è, «Scheda SD»; i posti che sul telefono non esistono
//!   non compaiono;
//! - a destra la cartella: un clic entra nelle cartelle e mette o toglie la
//!   spunta ai file, il doppio clic riceve subito quel file; le spunte restano
//!   cambiando cartella; «Scegli tutti» prende i file della cartella aperta;
//! - Fotocamera e Screenshot si aprono a miniature (le prepara l'aiutante sul
//!   telefono, [`crate::app::miniature`]), le altre cartelle a elenco.
//!
//! Il trasferimento lo fa il drawer, con la stessa scheda dell'invio: qui si
//! sceglie soltanto. Tutto passa dal protocollo `sync:` di ADB
//! ([`crate::adb::sync`]): niente app sul telefono.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;

use crate::adb::{Adb, sync};
use crate::esecutore;

/// Radice della memoria condivisa.
const MEMORIA: &str = "/sdcard";
/// Giorni di «Recenti».
const GIORNI_RECENTI: i64 = 7;
/// Voci mostrate per volta: le cartelle della Fotocamera possono averne migliaia.
const PER_PAGINA: usize = 200;
/// Lato delle miniature chieste al telefono, in pixel.
const LATO_MINIATURA: u32 = 192;
/// Miniature chieste all'aiutante per volta (ogni avvio costa circa un secondo).
const MINIATURE_PER_VOLTA: usize = 40;

/// Cartelle di WhatsApp: quella di Android 11+ e quella dei telefoni più vecchi.
const WHATSAPP: [&str; 2] = ["/sdcard/Android/media/com.whatsapp/WhatsApp/Media", "/sdcard/WhatsApp/Media"];

/// Un posto a sinistra: nome, icona, cartelle candidate (vale la prima che
/// esiste) e se si apre a miniature.
struct Posto {
    nome: &'static str,
    icona: &'static str,
    candidati: &'static [&'static str],
    griglia: bool,
}

const POSTI: [Posto; 5] = [
    Posto { nome: "Fotocamera", icona: "camera-photo-symbolic", candidati: &["/sdcard/DCIM/Camera"], griglia: true },
    Posto {
        nome: "Screenshot",
        icona: "phone-symbolic",
        candidati: &["/sdcard/DCIM/Screenshots", "/sdcard/Pictures/Screenshots"],
        griglia: true,
    },
    Posto { nome: "Download", icona: "folder-download-symbolic", candidati: &["/sdcard/Download"], griglia: false },
    Posto { nome: "WhatsApp", icona: "chat-message-new-symbolic", candidati: &WHATSAPP, griglia: false },
    Posto { nome: "Documenti", icona: "folder-documents-symbolic", candidati: &["/sdcard/Documents"], griglia: false },
];

/// Cartelle lette per «Recenti», col nome del posto da mostrare sotto il file.
fn fonti_recenti() -> Vec<(&'static str, String)> {
    let mut v: Vec<(&str, String)> = vec![
        ("Fotocamera", "/sdcard/DCIM/Camera".into()),
        ("Screenshot", "/sdcard/DCIM/Screenshots".into()),
        ("Screenshot", "/sdcard/Pictures/Screenshots".into()),
        ("Download", "/sdcard/Download".into()),
        ("Documenti", "/sdcard/Documents".into()),
    ];
    for radice in WHATSAPP {
        for sotto in ["WhatsApp Images", "WhatsApp Video", "WhatsApp Documents", "WhatsApp Audio"] {
            v.push(("WhatsApp", format!("{radice}/{sotto}")));
        }
    }
    v
}

/// Un file del telefono scelto (o da scegliere).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTelefono {
    pub percorso: String,
    pub nome: String,
    pub dimensione: u64,
    /// Ultima modifica, in secondi dal 1970.
    pub modificato: i64,
    /// Il posto da cui viene, in «Recenti».
    da: Option<&'static str>,
}

/// Una voce della cartella aperta.
#[derive(Debug, Clone)]
enum Elemento {
    Cartella { nome: String, percorso: String, modificato: i64 },
    File(FileTelefono),
}

impl Elemento {
    fn nome(&self) -> &str {
        match self {
            Elemento::Cartella { nome, .. } => nome,
            Elemento::File(f) => &f.nome,
        }
    }
}

/// Le voci di una cartella come elementi: senza i nascosti (quelli col punto
/// davanti, come `.thumbnails` o i file nel cestino di Samsung).
fn elementi(cartella: &str, voci: Vec<sync::Voce>, da: Option<&'static str>) -> Vec<Elemento> {
    voci.into_iter()
        .filter(|v| !v.nome.starts_with('.'))
        .map(|v| {
            let percorso = format!("{}/{}", cartella.trim_end_matches('/'), v.nome);
            if v.cartella {
                Elemento::Cartella { nome: v.nome, percorso, modificato: v.modificato }
            } else {
                Elemento::File(FileTelefono { percorso, nome: v.nome, dimensione: v.dimensione, modificato: v.modificato, da })
            }
        })
        .collect()
}

/// Prima le cartelle in ordine alfabetico, poi i file dal più nuovo (o dal
/// più vecchio).
fn ordina(v: &mut [Elemento], recenti_in_alto: bool) {
    v.sort_by(|a, b| match (a, b) {
        (Elemento::Cartella { nome: x, .. }, Elemento::Cartella { nome: y, .. }) => x.to_lowercase().cmp(&y.to_lowercase()),
        (Elemento::Cartella { .. }, Elemento::File(_)) => std::cmp::Ordering::Less,
        (Elemento::File(_), Elemento::Cartella { .. }) => std::cmp::Ordering::Greater,
        (Elemento::File(x), Elemento::File(y)) => {
            let o = x.modificato.cmp(&y.modificato).then_with(|| x.nome.cmp(&y.nome));
            if recenti_in_alto { o.reverse() } else { o }
        }
    });
}

/// I file di «Recenti»: modificati negli ultimi [`GIORNI_RECENTI`] giorni,
/// senza doppioni (la stessa cartella può arrivare da due candidati).
fn recenti(fonti: Vec<Vec<Elemento>>, adesso: i64) -> Vec<Elemento> {
    let limite = adesso - GIORNI_RECENTI * 86_400;
    let mut visti = HashSet::new();
    fonti
        .into_iter()
        .flatten()
        .filter(|e| matches!(e, Elemento::File(f) if f.modificato >= limite))
        .filter(|e| visti.insert(match e {
            Elemento::File(f) => f.percorso.clone(),
            Elemento::Cartella { percorso, .. } => percorso.clone(),
        }))
        .collect()
}

/// Giorni di calendario passati da `t` (0 = oggi), nell'ora locale.
fn giorni_fa(t: i64, adesso: &gtk::glib::DateTime) -> i64 {
    let mezzanotte = gtk::glib::DateTime::from_local(adesso.year(), adesso.month(), adesso.day_of_month(), 0, 0, 0.0)
        .map(|d| d.to_unix())
        .unwrap_or_else(|_| adesso.to_unix());
    if t >= mezzanotte { 0 } else { (mezzanotte - t - 1) / 86_400 + 1 }
}

/// Titolo del gruppo di «Recenti».
fn gruppo(giorni: i64) -> &'static str {
    match giorni {
        0 => "Oggi",
        1 => "Ieri",
        _ => "Questa settimana",
    }
}

/// «oggi 14:12», «ieri 18:30», «28/9», «28/9/2025».
fn quando(t: i64, adesso: &gtk::glib::DateTime) -> String {
    let Ok(d) = gtk::glib::DateTime::from_unix_local(t) else { return String::new() };
    let ora = d.format("%H:%M").map(|s| s.to_string()).unwrap_or_default();
    match giorni_fa(t, adesso) {
        0 => format!("oggi {ora}"),
        1 => format!("ieri {ora}"),
        _ if d.year() == adesso.year() => format!("{}/{}", d.day_of_month(), d.month()),
        _ => format!("{}/{}/{}", d.day_of_month(), d.month(), d.year()),
    }
}

/// Misura leggibile: «820 kB», «4,1 MB», «1,2 GB».
pub fn misura(byte: u64) -> String {
    let b = byte as f64;
    if b < 1e6 {
        format!("{} kB", (b / 1e3).round().max(if byte > 0 { 1.0 } else { 0.0 }))
    } else if b < 1e9 {
        format!("{:.1} MB", b / 1e6).replace('.', ",")
    } else {
        format!("{:.1} GB", b / 1e9).replace('.', ",")
    }
}

fn estensione(nome: &str) -> String {
    nome.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default()
}

/// Foto e video: Android sa farne una miniatura.
fn ha_miniatura(nome: &str) -> bool {
    matches!(
        estensione(nome).as_str(),
        "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "gif" | "bmp" | "mp4" | "mkv" | "webm" | "3gp" | "mov"
    )
}

/// Icona del tipo di file (quando non c'è una miniatura).
fn icona_tipo(nome: &str) -> &'static str {
    match estensione(nome).as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "heic" | "heif" | "gif" | "bmp" => "image-x-generic-symbolic",
        "mp4" | "mkv" | "webm" | "3gp" | "mov" => "video-x-generic-symbolic",
        "mp3" | "m4a" | "ogg" | "opus" | "aac" | "wav" | "flac" | "amr" => "audio-x-generic-symbolic",
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "xls" | "xlsx" | "ods" | "ppt" | "pptx" | "odp" => "x-office-document-symbolic",
        "zip" | "rar" | "7z" | "apk" | "gz" | "tar" => "package-x-generic-symbolic",
        _ => "text-x-generic-symbolic",
    }
}

/// Nome italiano delle cartelle note nella radice (come per l'invio), se diverso.
fn nome_italiano(cartella: &str) -> Option<&'static str> {
    crate::azioni::CARTELLE.iter().find(|(c, n)| *c == cartella && c != n).map(|(_, n)| *n)
}

/// Dove scrivere `nome` in `cartella` del PC senza sovrascrivere niente:
/// `foto.jpg`, poi `foto (1).jpg`… Le barre nel nome diventano trattini.
pub fn arrivo(cartella: &Path, nome: &str) -> std::path::PathBuf {
    let nome: String = nome.chars().map(|c| if c == '/' || c == '\0' { '-' } else { c }).collect();
    let nome = if nome.is_empty() || nome == "." || nome == ".." { "file".to_string() } else { nome };
    let esistenti: HashSet<String> = std::fs::read_dir(cartella)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    cartella.join(crate::azioni::nome_libero(&nome, &esistenti))
}

/// Nome da mostrare della cartella di arrivo: «Scaricati», «Documenti/Telefono»…
pub fn nome_cartella(cartella: &Path) -> String {
    if gtk::glib::user_special_dir(gtk::glib::UserDirectory::Downloads).as_deref() == Some(cartella) {
        return cartella.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    }
    match cartella.strip_prefix(gtk::glib::home_dir()) {
        Ok(r) if r.as_os_str().is_empty() => "Cartella personale".into(),
        Ok(r) => r.display().to_string(),
        Err(_) => cartella.display().to_string(),
    }
}

/// Un posto disponibile su questo telefono.
#[derive(Clone)]
struct PostoAperto {
    /// `None` = Recenti.
    cartella: Option<String>,
    griglia: bool,
}

struct Ricevi {
    adb: Adb,
    dialogo: adw::Dialog,
    posti: gtk::ListBox,
    righe_posti: RefCell<Vec<(gtk::ListBoxRow, PostoAperto)>>,
    /// Cartella aperta; `None` = Recenti.
    cartella: RefCell<Option<String>>,
    /// Radici della navigazione (memoria, scheda SD): «su» si ferma lì.
    radici: RefCell<Vec<(String, String)>>,
    briciole: gtk::Box,
    su: gtk::Button,
    tutti: gtk::Button,
    ricerca: gtk::SearchEntry,
    vista_griglia: gtk::ToggleButton,
    intestazione: gtk::Box,
    ordine: gtk::Button,
    pagine: gtk::Stack,
    elenco: gtk::ListBox,
    griglia: gtk::FlowBox,
    stato: adw::StatusPage,
    quanti: gtk::Label,
    pulsante_ricevi: gtk::Button,
    /// Voci della cartella aperta, già ordinate.
    voci: RefCell<Vec<Elemento>>,
    mostrate: Cell<usize>,
    recenti_in_alto: Cell<bool>,
    /// File spuntati, nell'ordine in cui sono stati scelti.
    scelti: RefCell<Vec<FileTelefono>>,
    /// Righe e riquadri mostrati, con l'elemento di ciascuno.
    elementi_mostrati: RefCell<Vec<(gtk::Widget, Elemento)>>,
    /// La finestra è chiusa: le miniature in arrivo non servono più.
    chiusa: Cell<bool>,
    /// Spunte visibili per percorso, per aggiornarle senza ridisegnare.
    spunte: RefCell<HashMap<String, Vec<gtk::CheckButton>>>,
    /// Riquadri delle miniature per percorso, e miniature già arrivate.
    cornici: RefCell<HashMap<String, Vec<gtk::Box>>>,
    miniature: RefCell<HashMap<String, Option<gtk::gdk::Texture>>>,
    /// Cambia a ogni cartella aperta: le letture vecchie si scartano.
    generazione: Cell<u64>,
    al_termine: Box<dyn Fn(Vec<FileTelefono>)>,
}

/// Apre la finestra sopra `finestra`; con i file scelti chiama `al_termine`.
pub fn apri(finestra: Option<&adw::ApplicationWindow>, adb: Adb, arrivo: &str, al_termine: impl Fn(Vec<FileTelefono>) + 'static) {
    let posti = gtk::ListBox::builder().selection_mode(gtk::SelectionMode::Single).css_classes(["navigation-sidebar"]).build();
    posti.set_header_func(|riga, _| {
        if riga.has_css_class("dopo-separatore") && riga.header().is_none() {
            riga.set_header(Some(&gtk::Separator::builder().margin_top(6).margin_bottom(6).margin_start(8).margin_end(8).build()));
        }
    });
    let colonna_posti = gtk::Box::builder().orientation(gtk::Orientation::Vertical).width_request(196).css_classes(["posti-ricevi"]).build();
    colonna_posti.append(&gtk::ScrolledWindow::builder().child(&posti).vexpand(true).hscrollbar_policy(gtk::PolicyType::Never).build());
    colonna_posti.append(
        &gtk::Label::builder()
            .label("Si vede la memoria condivisa del telefono, la stessa che si vede col cavo.")
            .wrap(true)
            .xalign(0.0)
            .margin_start(14)
            .margin_end(14)
            .margin_bottom(12)
            .css_classes(["caption", "dim-label"])
            .build(),
    );

    let su = gtk::Button::builder().icon_name("go-previous-symbolic").tooltip_text("Cartella superiore").css_classes(["flat"]).build();
    let briciole = gtk::Box::builder().spacing(2).hexpand(true).build();
    let scorri_briciole = gtk::ScrolledWindow::builder()
        .child(&briciole)
        .hexpand(true)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .hscrollbar_policy(gtk::PolicyType::External)
        .build();
    // Se il percorso non ci sta, resta in vista la fine: la cartella aperta.
    scorri_briciole.hadjustment().connect_changed(|a| a.set_value(a.upper() - a.page_size()));
    let tutti = gtk::Button::builder().label("Scegli tutti").valign(gtk::Align::Center).build();
    let cerca = gtk::ToggleButton::builder().icon_name("system-search-symbolic").tooltip_text("Cerca in questa cartella").css_classes(["flat"]).build();
    let vista_griglia = gtk::ToggleButton::builder().icon_name("view-grid-symbolic").tooltip_text("Miniature o elenco").css_classes(["flat"]).build();
    let strumenti = gtk::Box::builder().spacing(6).margin_start(12).margin_end(12).margin_top(8).margin_bottom(4).build();
    strumenti.append(&su);
    strumenti.append(&scorri_briciole);
    strumenti.append(&tutti);
    strumenti.append(&cerca);
    strumenti.append(&vista_griglia);

    let ricerca = gtk::SearchEntry::builder().placeholder_text("Cerca per nome").hexpand(true).build();
    let barra_ricerca = gtk::SearchBar::builder().child(&ricerca).show_close_button(false).build();
    barra_ricerca.connect_entry(&ricerca);
    cerca.bind_property("active", &barra_ricerca, "search-mode-enabled").bidirectional().build();

    let ordine = gtk::Button::builder().label("Modificato ▾").css_classes(["flat", "intestazione-ricevi"]).build();
    let intestazione = gtk::Box::builder().spacing(0).margin_start(12).margin_end(12).css_classes(["intestazione-ricevi"]).build();
    intestazione.append(&gtk::Label::builder().label("Nome").xalign(0.0).hexpand(true).margin_start(84).build());
    intestazione.append(&gtk::Label::builder().label("Dimensione").xalign(1.0).width_request(90).build());
    ordine.set_width_request(120);
    intestazione.append(&ordine);

    let elenco = gtk::ListBox::builder().selection_mode(gtk::SelectionMode::None).css_classes(["elenco-ricevi"]).build();
    let griglia = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .homogeneous(true)
        .min_children_per_line(3)
        .max_children_per_line(8)
        .row_spacing(6)
        .column_spacing(6)
        .valign(gtk::Align::Start)
        .css_classes(["griglia-ricevi"])
        .build();
    let stato = adw::StatusPage::builder().css_classes(["compact"]).build();
    let pagine = gtk::Stack::builder().vexpand(true).build();
    let scorri = |w: &gtk::Widget| {
        gtk::ScrolledWindow::builder()
            .child(w)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .margin_start(12)
            .margin_end(12)
            .build()
    };
    pagine.add_named(&scorri(elenco.upcast_ref()), Some("elenco"));
    pagine.add_named(&scorri(griglia.upcast_ref()), Some("griglia"));
    pagine.add_named(&stato, Some("stato"));

    let dove = gtk::Box::builder().spacing(8).hexpand(true).build();
    dove.append(&gtk::Image::from_icon_name("folder-symbolic"));
    dove.append(&gtk::Label::builder().use_markup(true).label(format!("Arrivano in <b>{}</b>", gtk::glib::markup_escape_text(arrivo))).css_classes(["dim-label"]).build());
    let quanti = gtk::Label::builder().css_classes(["dim-label"]).build();
    let annulla = gtk::Button::builder().label("Annulla").build();
    let pulsante_ricevi = gtk::Button::builder().label("Ricevi").sensitive(false).css_classes(["suggested-action"]).build();
    let piede = gtk::Box::builder().spacing(10).margin_start(14).margin_end(14).margin_top(10).margin_bottom(10).build();
    piede.append(&dove);
    piede.append(&quanti);
    piede.append(&annulla);
    piede.append(&pulsante_ricevi);

    let destra = gtk::Box::builder().orientation(gtk::Orientation::Vertical).hexpand(true).build();
    destra.append(&strumenti);
    destra.append(&barra_ricerca);
    destra.append(&intestazione);
    destra.append(&pagine);
    destra.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    destra.append(&piede);

    let corpo = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    corpo.append(&colonna_posti);
    corpo.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    corpo.append(&destra);
    let vista = adw::ToolbarView::new();
    vista.add_top_bar(&adw::HeaderBar::new());
    vista.set_content(Some(&corpo));
    let dialogo = adw::Dialog::builder()
        .title("Ricevi file dal telefono")
        .content_width(820)
        .content_height(580)
        .child(&vista)
        .css_classes(["ricevi-file"])
        .build();
    barra_ricerca.set_key_capture_widget(Some(&dialogo));

    let r = Rc::new(Ricevi {
        adb,
        dialogo: dialogo.clone(),
        posti,
        righe_posti: RefCell::new(Vec::new()),
        cartella: RefCell::new(None),
        radici: RefCell::new(vec![(MEMORIA.to_string(), "Memoria del telefono".to_string())]),
        briciole,
        su: su.clone(),
        tutti: tutti.clone(),
        ricerca: ricerca.clone(),
        vista_griglia: vista_griglia.clone(),
        intestazione,
        ordine: ordine.clone(),
        pagine,
        elenco,
        griglia,
        stato,
        quanti,
        pulsante_ricevi: pulsante_ricevi.clone(),
        voci: RefCell::new(Vec::new()),
        mostrate: Cell::new(PER_PAGINA),
        recenti_in_alto: Cell::new(true),
        scelti: RefCell::new(Vec::new()),
        elementi_mostrati: RefCell::new(Vec::new()),
        chiusa: Cell::new(false),
        spunte: RefCell::new(HashMap::new()),
        cornici: RefCell::new(HashMap::new()),
        miniature: RefCell::new(HashMap::new()),
        generazione: Cell::new(0),
        al_termine: Box::new(al_termine),
    });
    {
        let r = r.clone();
        r.clone().posti.connect_row_activated(move |_, riga| {
            let posto = r.righe_posti.borrow().iter().find(|(x, _)| x == riga).map(|(_, p)| p.clone());
            if let Some(p) = posto {
                r.vista_griglia.set_active(p.griglia);
                r.apri_cartella(p.cartella.clone());
            }
        });
    }
    {
        let r2 = r.clone();
        su.connect_clicked(move |_| r2.sali());
        let r2 = r.clone();
        tutti.connect_clicked(move |_| r2.scegli_tutti());
        let r2 = r.clone();
        ordine.connect_clicked(move |_| {
            r2.recenti_in_alto.set(!r2.recenti_in_alto.get());
            r2.ordine.set_label(if r2.recenti_in_alto.get() { "Modificato ▾" } else { "Modificato ▴" });
            ordina(&mut r2.voci.borrow_mut(), r2.recenti_in_alto.get());
            r2.mostra();
        });
        let r2 = r.clone();
        vista_griglia.connect_toggled(move |_| r2.mostra());
        let r2 = r.clone();
        ricerca.connect_search_changed(move |_| {
            r2.mostrate.set(PER_PAGINA);
            r2.mostra();
        });
        let d = dialogo.clone();
        annulla.connect_clicked(move |_| {
            d.close();
        });
        let r2 = r.clone();
        pulsante_ricevi.connect_clicked(move |_| r2.ricevi());
    }
    {
        let r2 = r.clone();
        r.elenco.connect_row_activated(move |_, riga| r2.attiva(riga.upcast_ref()));
        let r2 = r.clone();
        r.griglia.connect_child_activated(move |_, figlio| r2.attiva(figlio.upcast_ref()));
    }
    {
        // Alla chiusura si liberano voci e miniature (i pulsanti della
        // finestra tengono un riferimento allo stato).
        let r2 = r.clone();
        dialogo.connect_closed(move |_| {
            r2.chiusa.set(true);
            r2.generazione.set(r2.generazione.get() + 1);
            r2.voci.borrow_mut().clear();
            r2.elementi_mostrati.borrow_mut().clear();
            r2.miniature.borrow_mut().clear();
            r2.cornici.borrow_mut().clear();
            r2.spunte.borrow_mut().clear();
        });
    }
    r.aggiorna_scelti();
    r.prepara_posti();
    dialogo.present(finestra);
}

impl Ricevi {
    /// Cerca quali posti esistono su questo telefono, poi apre Recenti.
    fn prepara_posti(self: &Rc<Self>) {
        self.aggiungi_posto("Recenti", "document-open-recent-symbolic", None, false);
        self.posti.select_row(self.righe_posti.borrow().first().map(|(r, _)| r));
        self.apri_cartella(None);
        let adb = self.adb.clone();
        let r = self.clone();
        gtk::glib::spawn_future_local(async move {
            let trovati = esecutore()
                .spawn(async move {
                    let mut trovati = Vec::new();
                    for (i, p) in POSTI.iter().enumerate() {
                        for c in p.candidati {
                            if sync::e_cartella(&adb, c).await.unwrap_or(false) {
                                trovati.push((i, c.to_string()));
                                break;
                            }
                        }
                    }
                    // Schede SD: in /storage, oltre a «emulated» e «self».
                    let schede: Vec<String> = sync::elenca(&adb, "/storage")
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|v| v.cartella && v.nome != "emulated" && v.nome != "self" && !v.nome.starts_with('.'))
                        .map(|v| format!("/storage/{}", v.nome))
                        .collect();
                    (trovati, schede)
                })
                .await;
            let Ok((trovati, schede)) = trovati else { return };
            for (i, cartella) in trovati {
                let p = &POSTI[i];
                r.aggiungi_posto(p.nome, p.icona, Some(cartella), p.griglia);
            }
            r.aggiungi_posto("Memoria del telefono", "drive-harddisk-symbolic", Some(MEMORIA.to_string()), false);
            // Una riga separatore la barra laterale la disegnerebbe come una
            // voce (blocco grigio): la linea va nell'intestazione della riga.
            if let Some((memoria, _)) = r.righe_posti.borrow().last() {
                memoria.add_css_class("dopo-separatore");
            }
            let piu_schede = schede.len() > 1;
            for (n, s) in schede.into_iter().enumerate() {
                let nome = if piu_schede { format!("Scheda SD {}", n + 1) } else { "Scheda SD".to_string() };
                r.radici.borrow_mut().push((s.clone(), nome.clone()));
                r.aggiungi_posto(&nome, "media-flash-symbolic", Some(s), false);
            }
            r.posti.invalidate_headers();
            // Prove dell'interfaccia: `PHONESTRA_PROVA_RICEVI=<posto>` lo apre.
            let prova = std::env::var("PHONESTRA_PROVA_RICEVI").unwrap_or_default();
            let posto = r.righe_posti.borrow().iter().find(|(riga, _)| {
                riga.child().and_then(|b| b.last_child()).and_downcast::<gtk::Label>().is_some_and(|l| l.label() == prova)
            }).map(|(riga, _)| riga.clone());
            if let Some(riga) = posto {
                riga.activate();
            }
        });
    }

    fn aggiungi_posto(&self, nome: &str, icona: &str, cartella: Option<String>, griglia: bool) {
        let riga = gtk::Box::builder().spacing(10).build();
        riga.append(&gtk::Image::from_icon_name(icona));
        riga.append(&gtk::Label::builder().label(nome).xalign(0.0).build());
        let voce = gtk::ListBoxRow::builder().child(&riga).build();
        self.posti.append(&voce);
        self.righe_posti.borrow_mut().push((voce, PostoAperto { cartella, griglia }));
    }

    /// Il posto a sinistra che corrisponde alla cartella aperta.
    fn segna_posto(&self) {
        let cartella = self.cartella.borrow().clone();
        let righe = self.righe_posti.borrow();
        match righe.iter().find(|(_, p)| p.cartella == cartella) {
            Some((riga, _)) => self.posti.select_row(Some(riga)),
            None => self.posti.unselect_all(),
        }
    }

    /// Legge `cartella` (o Recenti) dal telefono e la mostra.
    fn apri_cartella(self: &Rc<Self>, cartella: Option<String>) {
        *self.cartella.borrow_mut() = cartella.clone();
        let generazione = self.generazione.get() + 1;
        self.generazione.set(generazione);
        self.mostrate.set(PER_PAGINA);
        self.ricerca.set_text("");
        self.voci.borrow_mut().clear();
        self.segna_posto();
        self.mostra_briciole();
        self.mostra_stato("Lettura…", "", "content-loading-symbolic");
        let adb = self.adb.clone();
        let r = self.clone();
        gtk::glib::spawn_future_local(async move {
            let letto = esecutore()
                .spawn(async move {
                    match cartella {
                        Some(c) => sync::elenca(&adb, &c).await.map(|v| v.map(|v| elementi(&c, v, None))),
                        None => {
                            let mut fonti = Vec::new();
                            for (da, c) in fonti_recenti() {
                                if let Ok(Some(v)) = sync::elenca(&adb, &c).await {
                                    fonti.push(elementi(&c, v, Some(da)));
                                }
                            }
                            let adesso = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map_or(0, |d| d.as_secs() as i64);
                            Ok(Some(recenti(fonti, adesso)))
                        }
                    }
                })
                .await;
            if r.generazione.get() != generazione {
                return;
            }
            match letto {
                Ok(Ok(Some(mut voci))) => {
                    ordina(&mut voci, r.recenti_in_alto.get());
                    *r.voci.borrow_mut() = voci;
                    r.mostra();
                }
                Ok(Ok(None)) => r.mostra_stato("Cartella non leggibile", "Il telefono non la mostra al PC.", "folder-symbolic"),
                Ok(Err(e)) => r.mostra_stato("Cartella non letta", &format!("{e:#}"), "dialog-warning-symbolic"),
                Err(e) => r.mostra_stato("Cartella non letta", &e.to_string(), "dialog-warning-symbolic"),
            }
        });
    }

    fn mostra_stato(&self, titolo: &str, testo: &str, icona: &str) {
        self.stato.set_title(titolo);
        self.stato.set_description(Some(testo).filter(|t| !t.is_empty()));
        self.stato.set_icon_name(Some(icona));
        self.pagine.set_visible_child_name("stato");
        self.intestazione.set_visible(false);
        self.tutti.set_visible(false);
    }

    /// Il percorso in alto: «Memoria del telefono › DCIM › Camera», ogni
    /// pezzo cliccabile; per Recenti solo il titolo.
    fn mostra_briciole(self: &Rc<Self>) {
        while let Some(f) = self.briciole.first_child() {
            self.briciole.remove(&f);
        }
        let Some(cartella) = self.cartella.borrow().clone() else {
            self.briciole.append(&gtk::Label::builder().label("Recenti").css_classes(["heading"]).margin_start(8).build());
            self.briciole.append(&gtk::Label::builder().label(format!("· ultimi {GIORNI_RECENTI} giorni")).css_classes(["dim-label"]).build());
            self.su.set_visible(false);
            return;
        };
        let radici = self.radici.borrow().clone();
        let (radice, nome_radice) = radici
            .iter()
            .filter(|(r, _)| cartella == *r || cartella.starts_with(&format!("{r}/")))
            .max_by_key(|(r, _)| r.len())
            .cloned()
            .unwrap_or_else(|| ("/".into(), "Telefono".into()));
        let mut pezzi = vec![(nome_radice, radice.clone())];
        let mut percorso = radice.clone();
        for p in cartella[radice.len()..].split('/').filter(|p| !p.is_empty()) {
            percorso = format!("{percorso}/{p}");
            pezzi.push((p.to_string(), percorso.clone()));
        }
        let ultimo = pezzi.len() - 1;
        for (i, (nome, percorso)) in pezzi.into_iter().enumerate() {
            if i > 0 {
                self.briciole.append(&gtk::Label::builder().label("›").css_classes(["dim-label"]).build());
            }
            if i == ultimo {
                self.briciole.append(&gtk::Label::builder().label(&nome).css_classes(["heading"]).margin_start(8).margin_end(8).build());
            } else {
                let b = gtk::Button::builder().label(&nome).css_classes(["flat"]).build();
                let r = self.clone();
                b.connect_clicked(move |_| r.apri_cartella(Some(percorso.clone())));
                self.briciole.append(&b);
            }
        }
        self.su.set_visible(cartella != radice);
    }

    fn sali(self: &Rc<Self>) {
        let Some(c) = self.cartella.borrow().clone() else { return };
        if let Some((sopra, _)) = c.rsplit_once('/') {
            self.apri_cartella(Some(sopra.to_string()));
        }
    }

    /// Le voci visibili (con la ricerca applicata).
    fn visibili(&self) -> Vec<Elemento> {
        let cerca = self.ricerca.text().to_lowercase();
        self.voci.borrow().iter().filter(|e| cerca.is_empty() || e.nome().to_lowercase().contains(&cerca)).cloned().collect()
    }

    /// Ridisegna la cartella aperta (elenco o miniature).
    fn mostra(self: &Rc<Self>) {
        self.spunte.borrow_mut().clear();
        self.cornici.borrow_mut().clear();
        self.elementi_mostrati.borrow_mut().clear();
        while let Some(r) = self.elenco.first_child() {
            self.elenco.remove(&r);
        }
        self.griglia.remove_all();
        let tutte = self.visibili();
        let griglia = self.vista_griglia.is_active();
        let recenti = self.cartella.borrow().is_none();
        let file: Vec<&FileTelefono> = tutte.iter().filter_map(|e| if let Elemento::File(f) = e { Some(f) } else { None }).collect();
        self.tutti.set_visible(!file.is_empty());
        self.aggiorna_tutti();
        if tutte.is_empty() {
            let (titolo, icona) = if self.ricerca.text().is_empty() {
                (if recenti { "Nessun file negli ultimi giorni" } else { "Cartella vuota" }, "folder-symbolic")
            } else {
                ("Nessun file con questo nome", "system-search-symbolic")
            };
            self.mostra_stato(titolo, "", icona);
            self.tutti.set_visible(false);
            return;
        }
        let adesso = gtk::glib::DateTime::now_local().expect("ora locale");
        let quante = tutte.len().min(self.mostrate.get());
        let mut ultimo_gruppo = None;
        for e in &tutte[..quante] {
            if griglia {
                let w = self.riquadro(e);
                self.griglia.append(&w);
                self.elementi_mostrati.borrow_mut().push((w.upcast(), e.clone()));
            } else {
                if recenti && let Elemento::File(f) = e {
                    let g = gruppo(giorni_fa(f.modificato, &adesso));
                    if ultimo_gruppo != Some(g) {
                        ultimo_gruppo = Some(g);
                        let titolo = gtk::Label::builder().label(g.to_uppercase()).xalign(0.0).css_classes(["titolo-gruppo-ricevi"]).build();
                        self.elenco.append(&gtk::ListBoxRow::builder().child(&titolo).activatable(false).selectable(false).build());
                    }
                }
                let w = self.riga(e, &adesso);
                self.elenco.append(&w);
                self.elementi_mostrati.borrow_mut().push((w.upcast(), e.clone()));
            }
        }
        if quante < tutte.len() {
            let altri = (tutte.len() - quante).min(PER_PAGINA);
            let b = gtk::Button::builder().label(format!("Mostra altri {altri}")).halign(gtk::Align::Center).margin_top(8).margin_bottom(8).build();
            let r = self.clone();
            b.connect_clicked(move |_| {
                r.mostrate.set(r.mostrate.get() + PER_PAGINA);
                r.mostra();
            });
            if griglia {
                let figlio = gtk::FlowBoxChild::builder().child(&b).build();
                self.griglia.append(&figlio);
            } else {
                self.elenco.append(&gtk::ListBoxRow::builder().child(&b).activatable(false).selectable(false).build());
            }
        }
        self.pagine.set_visible_child_name(if griglia { "griglia" } else { "elenco" });
        self.intestazione.set_visible(!griglia);
        self.carica_miniature(tutte[..quante].iter().filter_map(|e| if let Elemento::File(f) = e { Some(f.clone()) } else { None }).collect());
    }

    /// Clic su una riga o un riquadro: entra nella cartella o cambia la spunta.
    fn attiva(self: &Rc<Self>, w: &gtk::Widget) {
        let elemento = self.elementi_mostrati.borrow().iter().find(|(x, _)| x == w).map(|(_, e)| e.clone());
        match elemento {
            Some(Elemento::Cartella { percorso, .. }) => self.apri_cartella(Some(percorso)),
            Some(Elemento::File(f)) => self.cambia_scelta(&f),
            None => {}
        }
    }

    /// Riga dell'elenco: spunta, miniatura o icona, nome (e posto), misura, data.
    fn riga(self: &Rc<Self>, e: &Elemento, adesso: &gtk::glib::DateTime) -> gtk::ListBoxRow {
        let riga = gtk::Box::builder().spacing(10).css_classes(["riga-ricevi"]).build();
        let (nome, sotto, misura_testo, data) = match e {
            Elemento::Cartella { nome, modificato, .. } => {
                riga.append(&gtk::Box::builder().width_request(22).build());
                riga.append(&self.cornice(None, nome, 32));
                let italiano = if self.cartella.borrow().as_deref() == Some(MEMORIA) { nome_italiano(nome) } else { None };
                (nome.clone(), italiano.map(str::to_string), String::new(), quando(*modificato, adesso))
            }
            Elemento::File(f) => {
                riga.append(&self.spunta(f));
                riga.append(&self.cornice(Some(f), &f.nome, 32));
                (f.nome.clone(), f.da.map(str::to_string), misura(f.dimensione), quando(f.modificato, adesso))
            }
        };
        let testi = gtk::Box::builder().orientation(gtk::Orientation::Vertical).valign(gtk::Align::Center).hexpand(true).build();
        testi.append(&gtk::Label::builder().label(&nome).xalign(0.0).ellipsize(gtk::pango::EllipsizeMode::Middle).build());
        if let Some(s) = sotto {
            testi.append(&gtk::Label::builder().label(s).xalign(0.0).css_classes(["caption", "dim-label"]).build());
        }
        riga.append(&testi);
        riga.append(&gtk::Label::builder().label(misura_testo).xalign(1.0).width_request(90).css_classes(["dim-label"]).build());
        riga.append(&gtk::Label::builder().label(data).xalign(1.0).width_request(110).css_classes(["dim-label"]).build());
        let voce = gtk::ListBoxRow::builder().child(&riga).build();
        self.doppio_clic(&voce, e);
        voce
    }

    /// Riquadro delle miniature: immagine grande, spunta nell'angolo, nome.
    fn riquadro(self: &Rc<Self>, e: &Elemento) -> gtk::FlowBoxChild {
        let colonna = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(5).css_classes(["riquadro-ricevi"]).build();
        let sopra = gtk::Overlay::new();
        match e {
            Elemento::Cartella { nome, .. } => {
                sopra.set_child(Some(&self.cornice(None, nome, 92)));
            }
            Elemento::File(f) => {
                sopra.set_child(Some(&self.cornice(Some(f), &f.nome, 92)));
                let s = self.spunta(f);
                s.set_halign(gtk::Align::Start);
                s.set_valign(gtk::Align::Start);
                s.set_margin_start(6);
                s.set_margin_top(6);
                sopra.add_overlay(&s);
            }
        }
        colonna.append(&sopra);
        colonna.append(&gtk::Label::builder().label(e.nome()).ellipsize(gtk::pango::EllipsizeMode::Middle).max_width_chars(14).css_classes(["caption"]).build());
        let figlio = gtk::FlowBoxChild::builder().child(&colonna).build();
        self.doppio_clic(&figlio, e);
        figlio
    }

    /// Doppio clic su un file: si riceve subito solo quello.
    fn doppio_clic(self: &Rc<Self>, w: &impl IsA<gtk::Widget>, e: &Elemento) {
        let Elemento::File(f) = e else { return };
        let clic = gtk::GestureClick::new();
        let (r, f) = (self.clone(), f.clone());
        clic.connect_pressed(move |_, n, _, _| {
            if n == 2 {
                *r.scelti.borrow_mut() = vec![f.clone()];
                r.ricevi();
            }
        });
        w.add_controller(clic);
    }

    /// Spunta di un file: solo da vedere, il clic lo prende la riga.
    fn spunta(&self, f: &FileTelefono) -> gtk::CheckButton {
        let s = gtk::CheckButton::builder()
            .active(self.scelti.borrow().iter().any(|x| x.percorso == f.percorso))
            .can_target(false)
            .can_focus(false)
            .valign(gtk::Align::Center)
            .build();
        self.spunte.borrow_mut().entry(f.percorso.clone()).or_default().push(s.clone());
        s
    }

    /// Il posto della miniatura (o dell'icona) di un file o di una cartella.
    fn cornice(&self, f: Option<&FileTelefono>, nome: &str, lato: i32) -> gtk::Box {
        let larghezza = if lato > 40 { 104 } else { lato };
        // Nella griglia il riquadro riempie la cella: le miniature ritagliate
        // hanno tutte la stessa misura.
        let (allinea, espandi) = if lato > 40 { (gtk::Align::Fill, true) } else { (gtk::Align::Center, false) };
        let c = gtk::Box::builder()
            .width_request(larghezza)
            .height_request(lato)
            .halign(allinea)
            .hexpand(espandi)
            .overflow(gtk::Overflow::Hidden)
            .css_classes(["cornice-ricevi"])
            .build();
        let icona = if f.is_none() { "folder" } else { icona_tipo(nome) };
        match f.and_then(|f| self.miniature.borrow().get(&f.percorso).cloned().flatten()) {
            Some(t) => metti_miniatura(&c, &t),
            None => {
                let i = gtk::Image::builder().icon_name(icona).pixel_size(if lato > 40 { 48 } else { 24 }).hexpand(true).build();
                if f.is_none() {
                    i.set_pixel_size(if lato > 40 { 64 } else { 32 });
                }
                c.append(&i);
            }
        }
        if let Some(f) = f
            && ha_miniatura(&f.nome)
            && !self.miniature.borrow().contains_key(&f.percorso)
        {
            self.cornici.borrow_mut().entry(f.percorso.clone()).or_default().push(c.clone());
        }
        c
    }

    /// Chiede al telefono le miniature che mancano, a gruppi, e le mette
    /// man mano nei riquadri (se la cartella è ancora quella).
    fn carica_miniature(self: &Rc<Self>, file: Vec<FileTelefono>) {
        let mancanti: Vec<String> = {
            let m = self.miniature.borrow();
            file.into_iter().filter(|f| ha_miniatura(&f.nome) && !m.contains_key(&f.percorso)).map(|f| f.percorso).collect()
        };
        if mancanti.is_empty() {
            return;
        }
        let generazione = self.generazione.get();
        let r = self.clone();
        gtk::glib::spawn_future_local(async move {
            for gruppo in mancanti.chunks(MINIATURE_PER_VOLTA) {
                if r.generazione.get() != generazione || r.chiusa.get() {
                    return;
                }
                let (adb, percorsi) = (r.adb.clone(), gruppo.to_vec());
                let risposta = esecutore().spawn(async move { crate::app::miniature(&adb, LATO_MINIATURA, &percorsi).await }).await;
                let Ok(Ok(immagini)) = risposta else {
                    return;
                };
                for (percorso, jpeg) in gruppo.iter().zip(immagini) {
                    let t = jpeg.and_then(|j| gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from(&j[..])).ok());
                    r.miniature.borrow_mut().insert(percorso.clone(), t.clone());
                    if let (Some(t), Some(cornici)) = (t, r.cornici.borrow_mut().remove(percorso)) {
                        for c in cornici {
                            metti_miniatura(&c, &t);
                        }
                    }
                }
            }
        });
    }

    /// Mette o toglie la spunta a un file.
    fn cambia_scelta(&self, f: &FileTelefono) {
        {
            let mut scelti = self.scelti.borrow_mut();
            match scelti.iter().position(|x| x.percorso == f.percorso) {
                Some(i) => {
                    scelti.remove(i);
                }
                None => scelti.push(f.clone()),
            }
        }
        self.aggiorna_scelti();
    }

    /// «Scegli tutti» / «Togli tutti» sui file visibili della cartella.
    fn scegli_tutti(&self) {
        let file: Vec<FileTelefono> = self.visibili().into_iter().filter_map(|e| if let Elemento::File(f) = e { Some(f) } else { None }).collect();
        {
            let mut scelti = self.scelti.borrow_mut();
            let tutti = file.iter().all(|f| scelti.iter().any(|x| x.percorso == f.percorso));
            if tutti {
                scelti.retain(|x| !file.iter().any(|f| f.percorso == x.percorso));
            } else {
                for f in file {
                    if !scelti.iter().any(|x| x.percorso == f.percorso) {
                        scelti.push(f);
                    }
                }
            }
        }
        self.aggiorna_scelti();
    }

    /// Spunte, contatore in basso, «Ricevi» e «Scegli tutti» secondo i file scelti.
    fn aggiorna_scelti(&self) {
        let scelti = self.scelti.borrow();
        for (percorso, spunte) in self.spunte.borrow().iter() {
            let si = scelti.iter().any(|x| &x.percorso == percorso);
            for s in spunte {
                s.set_active(si);
            }
        }
        let totale: u64 = scelti.iter().map(|f| f.dimensione).sum();
        self.quanti.set_label(&if scelti.is_empty() { "Scegli uno o più file".to_string() } else { format!("{} file · {}", scelti.len(), misura(totale)) });
        self.pulsante_ricevi.set_sensitive(!scelti.is_empty());
        drop(scelti);
        self.aggiorna_tutti();
    }

    fn aggiorna_tutti(&self) {
        let scelti = self.scelti.borrow();
        let visibili = self.visibili();
        let mut file = visibili.iter().filter_map(|e| if let Elemento::File(f) = e { Some(f) } else { None }).peekable();
        let tutti = file.peek().is_some() && file.all(|f| scelti.iter().any(|x| x.percorso == f.percorso));
        self.tutti.set_label(if tutti { "Togli tutti" } else { "Scegli tutti" });
    }

    /// Chiude la finestra e passa i file scelti al drawer.
    fn ricevi(&self) {
        let scelti = self.scelti.borrow().clone();
        if scelti.is_empty() {
            return;
        }
        self.dialogo.close();
        (self.al_termine)(scelti);
    }
}

/// Sostituisce l'icona del riquadro con la miniatura, ritagliata a riempirlo.
fn metti_miniatura(c: &gtk::Box, t: &gtk::gdk::Texture) {
    while let Some(f) = c.first_child() {
        c.remove(&f);
    }
    let p = gtk::Picture::builder().paintable(t).content_fit(gtk::ContentFit::Cover).hexpand(true).vexpand(true).build();
    c.append(&p);
}

/// Stile della finestra: righe e riquadri come nel mockup.
pub(crate) const CSS: &str = "
    dialog.ricevi-file .posti-ricevi { background: alpha(currentColor, 0.04); }
    dialog.ricevi-file .intestazione-ricevi { font-size: 11.5px; font-weight: 600; opacity: 0.6; min-height: 22px; padding: 0; }
    dialog.ricevi-file button.intestazione-ricevi { opacity: 1; padding: 0 6px; }
    dialog.ricevi-file list.elenco-ricevi { background: none; }
    dialog.ricevi-file list.elenco-ricevi > row { border-radius: 8px; padding: 0 6px; }
    dialog.ricevi-file .riga-ricevi { min-height: 44px; }
    dialog.ricevi-file .titolo-gruppo-ricevi { font-size: 11px; font-weight: 700; letter-spacing: 1px; opacity: 0.6; margin: 10px 4px 2px 4px; }
    dialog.ricevi-file .cornice-ricevi { border-radius: 6px; }
    dialog.ricevi-file .riquadro-ricevi .cornice-ricevi { border-radius: 8px; background: alpha(currentColor, 0.05); }
    dialog.ricevi-file flowboxchild { border-radius: 10px; padding: 6px; }
    dialog.ricevi-file .riquadro-ricevi checkbutton check { background-color: rgba(255,255,255,0.85); }
    dialog.ricevi-file .riquadro-ricevi checkbutton check:checked { background-color: @accent_bg_color; }
";

#[cfg(test)]
mod prove {
    use super::*;

    fn file(nome: &str, modificato: i64) -> Elemento {
        Elemento::File(FileTelefono { percorso: format!("/sdcard/Download/{nome}"), nome: nome.into(), dimensione: 1, modificato, da: None })
    }

    fn voce(nome: &str, cartella: bool) -> sync::Voce {
        sync::Voce { nome: nome.into(), cartella, dimensione: 5, modificato: 10 }
    }

    #[test]
    fn elementi_senza_nascosti() {
        let e = elementi("/sdcard/DCIM/", vec![voce(".thumbnails", true), voce("Camera", true), voce("a.jpg", false)], None);
        assert_eq!(e.len(), 2);
        assert!(matches!(&e[0], Elemento::Cartella { percorso, .. } if percorso == "/sdcard/DCIM/Camera"));
        assert!(matches!(&e[1], Elemento::File(f) if f.percorso == "/sdcard/DCIM/a.jpg"));
    }

    #[test]
    fn cartelle_prima_poi_file_dal_piu_nuovo() {
        let mut v = vec![file("vecchio", 1), Elemento::Cartella { nome: "b".into(), percorso: "/b".into(), modificato: 0 }, file("nuovo", 9),
            Elemento::Cartella { nome: "A".into(), percorso: "/A".into(), modificato: 0 }];
        ordina(&mut v, true);
        let nomi: Vec<&str> = v.iter().map(Elemento::nome).collect();
        assert_eq!(nomi, ["A", "b", "nuovo", "vecchio"]);
        ordina(&mut v, false);
        assert_eq!(v[2].nome(), "vecchio");
    }

    #[test]
    fn recenti_ultimi_sette_giorni_senza_doppioni() {
        let adesso = 100 * 86_400;
        let v = recenti(vec![vec![file("oggi", adesso - 10), file("vecchio", adesso - 8 * 86_400)], vec![file("oggi", adesso - 10)]], adesso);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].nome(), "oggi");
    }

    #[test]
    fn gruppi_e_giorni() {
        let adesso = gtk::glib::DateTime::from_local(2026, 9, 30, 14, 32, 0.0).unwrap();
        let t = |g: i32, h: i32| gtk::glib::DateTime::from_local(2026, 9, g, h, 0, 0.0).unwrap().to_unix();
        assert_eq!(giorni_fa(t(30, 1), &adesso), 0);
        assert_eq!(giorni_fa(t(29, 23), &adesso), 1);
        assert_eq!(giorni_fa(t(27, 12), &adesso), 3);
        assert_eq!(gruppo(1), "Ieri");
        assert_eq!(quando(t(29, 18), &adesso), "ieri 18:00");
        assert_eq!(quando(t(12, 9), &adesso), "12/9");
        let anno_scorso = gtk::glib::DateTime::from_local(2025, 2, 3, 9, 0, 0.0).unwrap().to_unix();
        assert_eq!(quando(anno_scorso, &adesso), "3/2/2025");
    }

    #[test]
    fn misure() {
        assert_eq!(misura(0), "0 kB");
        assert_eq!(misura(300), "1 kB");
        assert_eq!(misura(212_000), "212 kB");
        assert_eq!(misura(4_100_000), "4,1 MB");
        assert_eq!(misura(1_200_000_000), "1,2 GB");
    }

    #[test]
    fn tipi_di_file() {
        assert!(ha_miniatura("IMG_1.JPG") && ha_miniatura("VID.mp4") && !ha_miniatura("fattura.pdf"));
        assert_eq!(icona_tipo("fattura.pdf"), "x-office-document-symbolic");
        assert_eq!(icona_tipo("nota.opus"), "audio-x-generic-symbolic");
        assert_eq!(nome_italiano("Documents"), Some("Documenti"));
        assert_eq!(nome_italiano("Download"), None);
    }

    #[test]
    fn arrivo_senza_sovrascrivere() {
        let d = std::env::temp_dir().join(format!("phonestra-ricevi-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("foto.jpg"), b"x").unwrap();
        assert_eq!(arrivo(&d, "foto.jpg"), d.join("foto (1).jpg"));
        assert_eq!(arrivo(&d, "a/b.txt"), d.join("a-b.txt"));
        assert_eq!(arrivo(&d, ".."), d.join("file"));
        std::fs::remove_dir_all(&d).unwrap();
    }
}
