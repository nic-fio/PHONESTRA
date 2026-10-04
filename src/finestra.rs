//! Una app del telefono in una finestra GTK4/libadwaita (SPECIFICATION §7.3–7.5).
//!
//! - video: pacchetti H.264 del componente → GStreamer (appsrc → h264parse →
//!   decodebin → gtk4paintablesink) → `GtkPicture`;
//! - mouse: trascinamento/clic → tocchi alle coordinate del display virtuale;
//! - Indietro: pulsante, Esc, tasto «indietro» del mouse;
//! - il display virtuale prende la forma della finestra;
//! - collegamento perso: fascia «Riconnetti ora»; la sessione riparte da sola
//!   quando il [`Collegamento`] torna.

use std::sync::{Arc, Mutex};

use adw::prelude::*;
use anyhow::Result;
use gst::prelude::*;

use crate::adb::{Adb, Chiusore};
use crate::collegamento::{Collegamento, Stato};
use crate::componente::Condiviso;
use crate::esecutore;
use crate::t;
use crate::input_nostro::InputNostro;
use crate::video_nostro::{ComandiVideo, Evento, Opzioni, Pacchetto, SessioneNostra, leggi_pacchetto};

/// Comandi dalla finestra alla sessione.
enum Comando {
    Tocco { azione: u8, x: i32, y: i32 },
    Indietro,
    /// Clic destro: pressione lunga del dito nel punto (selezione, menu).
    PressioneLunga { x: i32, y: i32 },
    /// Zoom col pizzico a due dita nel punto (Ctrl+rotellina, Ctrl +/−, touchpad).
    Zoom { x: i32, y: i32, ingrandisci: bool },
    /// Rotellina nel punto `x`,`y` (scatti; verticale positivo = su).
    Scorri { x: i32, y: i32, orizzontale: f32, verticale: f32 },
    /// Testo scritto sulla tastiera del PC, già interpretato col suo layout.
    Testo(String),
    /// Ctrl+V: testo degli appunti del PC da incollare nell'app.
    Incolla(String),
    /// Tasto speciale o scorciatoia: codice Android, giù/su, modificatori.
    Tasto { giu: bool, codice: u32, meta: u32 },
    /// La finestra ha cambiato dimensione: pixel del display virtuale.
    Ridimensiona { larghezza: u32, altezza: u32 },
    /// Serve un fotogramma completo subito (inizio di una registrazione).
    RicominciaVideo,
    Chiudi,
}

/// Una registrazione in corso: il flusso del telefono salvato in MP4 così
/// com'è, senza ricodifica (SPECIFICATION §13).
struct Registratore {
    sorgente: gst_app::AppSrc,
    /// L'audio del telefono (Opus, senza ricodifica).
    audio: gst_app::AppSrc,
    pipeline: gst::Pipeline,
    /// Si comincia dal primo fotogramma completo.
    iniziata: bool,
    percorso: std::path::PathBuf,
}

type Registrazione = Arc<Mutex<Option<Registratore>>>;

/// Perché una sessione è finita.
enum FineSessione {
    /// L'utente ha chiuso la finestra.
    Chiusa,
    /// Il collegamento è caduto o il componente si è fermato.
    Caduta,
    /// La finestra è cambiata tanto da richiedere un display con un'altra
    /// densità (per esempio ingrandita a tutto schermo): si ricrea subito.
    Ricrea,
}

/// Se l'app della finestra accetta solo il verticale (`None` finché non si sa).
type Verticale = Arc<Mutex<Option<bool>>>;

/// Schermata protetta nella finestra, come la segnala il componente nostro
/// (`None` finché non l'ha detto).
type Protetta = Arc<Mutex<Option<bool>>>;

fn debug() -> bool {
    std::env::var_os("PHONESTRA_DEBUG").is_some()
}

/// Prefisso di `pacchetto` per aprire, al posto dell'app, la sua pagina
/// «Informazioni app» nelle impostazioni di Android (permessi, spazio…).
pub const INFORMAZIONI: &str = "informazioni:";

/// `pacchetto` per lo schermo del telefono nel drawer: lo schermo principale
/// com'è (SPECIFICATION §7.2).
pub const SCHERMO: &str = "schermo-del-telefono";

/// Spiegazione nella fascia del collegamento perso.
fn testo_perso() -> &'static str {
    t!("Riprovo da solo. Se il telefono è bloccato, sbloccalo: l'app torna qui dov'era.")
}

/// Spiegazione quando il componente sul telefono non parte (vedi
/// [`Collegamento::guasto`]), con il motivo tecnico in fondo.
pub fn testo_guasto(dettaglio: &str) -> String {
    t!(
        "Il telefono non riesce ad avviare la parte di Phonestra che mostra le app. \
         Tocca «Riconnetti ora» per riprovare; se non basta, riavvia il telefono.\n\n({})",
        dettaglio
    )
}

/// Il video del telefono con mouse e tastiera: il cuore di una finestra di
/// app, usato anche per lo schermo vero nel drawer.
pub struct Vista {
    /// L'immagine (riceve mouse e, nel drawer, tastiera).
    pub immagine: gtk::Picture,
    /// Da mettere nella finestra: immagine, schermata protetta e avvisi.
    pub contenuto: adw::ToastOverlay,
    tx: tokio::sync::mpsc::UnboundedSender<Comando>,
    pipeline: gst::Pipeline,
    registrazione: Registrazione,
    verticale: Verticale,
}

impl Vista {
    /// L'immagine attuale alla risoluzione del telefono.
    pub fn fotografa(&self) -> Option<gtk::gdk::Texture> {
        let immagine = self.immagine.paintable()?.current_image();
        let (l, a) = (immagine.intrinsic_width(), immagine.intrinsic_height());
        if l <= 0 || a <= 0 {
            return None;
        }
        let istantanea = gtk::Snapshot::new();
        immagine.snapshot(&istantanea, l as f64, a as f64);
        let nodo = istantanea.to_node()?;
        let disegnatore = self.immagine.native()?.renderer()?;
        Some(disegnatore.render_texture(nodo, Some(&gtk::graphene::Rect::new(0.0, 0.0, l as f32, a as f32))))
    }

    pub fn registra(&self) -> bool {
        self.registrazione.lock().unwrap().is_some()
    }

    /// Comincia a registrare in `percorso` (MP4).
    pub fn inizia_registrazione(&self, percorso: &std::path::Path) -> Result<(), String> {
        let pipeline = gst::parse::launch(
            "appsrc name=sorgente is-live=true do-timestamp=true format=time \
             caps=video/x-h264,stream-format=byte-stream,alignment=au \
             ! h264parse ! mp4mux name=mux ! filesink name=file \
             appsrc name=audio is-live=true format=time ! mux.",
        )
        .map_err(|e| e.to_string())?
        .downcast::<gst::Pipeline>()
        .map_err(|_| t!("pipeline di registrazione").to_string())?;
        let file = pipeline.by_name("file").ok_or(t!("filesink mancante"))?;
        file.set_property("location", percorso.to_string_lossy().to_string());
        let sorgente = pipeline.by_name("sorgente").and_then(|s| s.downcast::<gst_app::AppSrc>().ok()).ok_or(t!("appsrc mancante"))?;
        let audio = pipeline.by_name("audio").and_then(|s| s.downcast::<gst_app::AppSrc>().ok()).ok_or(t!("appsrc audio mancante"))?;
        // L'audio del componente nostro (AAC) va nel file così com'è. Il PCM
        // (riserva per le prove) non va in MP4: senza AAC in corso la
        // registrazione resta senza audio.
        let formato_audio = crate::audio_nostro::formato_in_corso().filter(|f| *f == crate::audio_nostro::Formato::Aac);
        let caps_audio = formato_audio.and_then(|_| crate::audio_nostro::caps_registrazione());
        audio.set_caps(caps_audio.as_ref());
        pipeline.set_state(gst::State::Playing).map_err(|e| e.to_string())?;
        *self.registrazione.lock().unwrap() =
            Some(Registratore { sorgente, audio, pipeline: pipeline.clone(), iniziata: false, percorso: percorso.to_path_buf() });
        let _ = self.tx.send(Comando::RicominciaVideo);

        // Audio: dal primo fotogramma completo del video. Gli orari vengono
        // dal telefono (regolari, 20 ms a pacchetto), ancorati all'arrivo del
        // primo pacchetto sull'orologio della registrazione, come il video.
        let registrazione = self.registrazione.clone();
        let mut pacchetti = crate::audio_nostro::ascolta();
        esecutore().spawn(async move {
            let mut inizio: Option<(u64, gst::ClockTime)> = None;
            loop {
                let (pts, dati) = match pacchetti.recv().await {
                    Ok(p) => p,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => return,
                };
                let guardia = registrazione.lock().unwrap();
                let Some(r) = guardia.as_ref() else { return };
                if r.pipeline != pipeline {
                    return; // un'altra registrazione: questa è finita
                }
                let Some(formato) = formato_audio else { return };
                if !r.iniziata {
                    continue;
                }
                let adesso = || -> Option<gst::ClockTime> { Some(pipeline.clock()?.time().saturating_sub(pipeline.base_time()?)) };
                let (pts0, t0) = match inizio {
                    Some(i) => i,
                    None => match adesso() {
                        Some(t) => *inizio.insert((pts, t)),
                        None => continue,
                    },
                };
                let mut buffer = gst::Buffer::from_mut_slice(dati);
                {
                    let b = buffer.get_mut().unwrap();
                    b.set_pts(t0 + gst::ClockTime::from_useconds(pts.saturating_sub(pts0)));
                    b.set_duration(gst::ClockTime::from_useconds(crate::audio_nostro::durata_pacchetto(formato, b.size())));
                }
                let _ = r.audio.push_buffer(buffer);
            }
        });
        Ok(())
    }

    /// Ferma la registrazione e chiude bene il file; restituisce dove sta.
    pub async fn ferma_registrazione(&self) -> Option<std::path::PathBuf> {
        let r = self.registrazione.lock().unwrap().take()?;
        let _ = r.sorgente.end_of_stream();
        let _ = r.audio.end_of_stream();
        let pipeline = r.pipeline.clone();
        // mp4mux scrive l'indice alla fine: si aspetta che il file sia chiuso.
        let _ = esecutore()
            .spawn_blocking(move || {
                if let Some(bus) = pipeline.bus() {
                    bus.timed_pop_filtered(gst::ClockTime::from_seconds(5), &[gst::MessageType::Eos, gst::MessageType::Error]);
                }
                let _ = pipeline.set_state(gst::State::Null);
            })
            .await;
        Some(r.percorso)
    }

    pub fn indietro(&self) {
        let _ = self.tx.send(Comando::Indietro);
    }

    /// Ferma video e sessione (la finestra o il drawer si chiude).
    pub fn chiudi(&self) {
        let _ = self.pipeline.set_state(gst::State::Null);
        let _ = self.tx.send(Comando::Chiudi);
    }
}

/// Crea la vista di `pacchetto` (un'app, [`SCHERMO`] o [`INFORMAZIONI`]…) e
/// avvia la sua sessione. I tasti si ascoltano su `tasti_su` (la finestra);
/// con `None` sull'immagine stessa, dopo un clic.
pub fn vista(collegamento: Arc<Collegamento>, pacchetto: &str, tasti_su: Option<&gtk::Widget>) -> Result<Vista, String> {
    let immagine = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::Contain)
        .can_shrink(true)
        .hexpand(true)
        .vexpand(true)
        .build();
    // Schermate protette (password, banche): Android le cattura nere; al loro
    // posto un messaggio chiaro (SPECIFICATION §7.6). Non si aggirano.
    let protetta = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .css_classes(["velo-app", "nero"])
        .visible(false)
        .build();
    {
        let centro = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).valign(gtk::Align::Center).vexpand(true).build();
        centro.append(&gtk::Image::builder().icon_name("system-lock-screen-symbolic").pixel_size(28).halign(gtk::Align::Center).css_classes(["cerchio"]).build());
        centro.append(&gtk::Label::builder().label(t!("Schermata protetta")).css_classes(["titolo-velo-app"]).build());
        centro.append(
            &gtk::Label::builder()
                .label(t!("Questa app non permette di mostrare questa schermata fuori dal telefono. Le altre schermate dell'app funzionano normalmente."))
                .wrap(true)
                .max_width_chars(34)
                .justify(gtk::Justification::Center)
                .css_classes(["testo-velo-app"])
                .build(),
        );
        protetta.append(&centro);
    }
    let sovrapposto = gtk::Overlay::builder().child(&immagine).build();
    sovrapposto.add_overlay(&protetta);
    let avvisi = adw::ToastOverlay::new();
    avvisi.set_child(Some(&sovrapposto));
    let pipeline = match gst::parse::launch(
        "appsrc name=sorgente is-live=true do-timestamp=true format=time \
         caps=video/x-h264,stream-format=byte-stream,alignment=au \
         ! h264parse ! decodebin ! videoconvert ! gtk4paintablesink name=schermo sync=false",
    ) {
        Ok(p) => p.downcast::<gst::Pipeline>().expect("pipeline"),
        Err(e) => return Err(t!("video non disponibile: {} (manca gstreamer1.0-gtk4?)", e)),
    };
    let sorgente = pipeline.by_name("sorgente").unwrap().downcast::<gst_app::AppSrc>().unwrap();
    let schermo = pipeline.by_name("schermo").unwrap();
    immagine.set_paintable(Some(&schermo.property::<gtk::gdk::Paintable>("paintable")));

    // Dimensione del video, per convertire le coordinate del mouse.
    let dimensione = Arc::new(Mutex::new((0u32, 0u32)));
    // Numero del display virtuale della sessione attuale (-1 finché non c'è).
    let display = Arc::new(std::sync::atomic::AtomicI32::new(-1));
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Comando>();

    // Il display virtuale segue la finestra, misurata in punti (1 punto del
    // PC = 1 dp del telefono). Finché la finestra non si è misurata vale
    // `None`: la sessione aspetta, perché il ridimensionamento cambia la
    // misura ma non la densità.
    let formato: Arc<Mutex<Option<(u32, u32)>>> = Arc::new(Mutex::new(None));
    {
        let (tx, formato) = (tx.clone(), formato.clone());
        immagine.add_tick_callback(move |immagine, _| {
            let (l, a) = (immagine.width().max(0) as u32, immagine.height().max(0) as u32);
            let mut attuale = formato.lock().unwrap();
            if l >= 150 && a >= 150 && *attuale != Some((l, a)) {
                *attuale = Some((l, a));
                let _ = tx.send(Comando::Ridimensiona { larghezza: l, altezza: a });
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    // Mouse → tocchi.
    let trascina = gtk::GestureDrag::new();
    trascina.set_button(gtk::gdk::BUTTON_PRIMARY);
    let posizione = {
        let immagine = immagine.clone();
        let dimensione = dimensione.clone();
        move |x: f64, y: f64| -> Option<(i32, i32)> {
            let (vl, va) = *dimensione.lock().unwrap();
            if vl == 0 {
                return None;
            }
            let (l, a) = (immagine.width() as f64, immagine.height() as f64);
            let scala = (l / vl as f64).min(a / va as f64);
            let (ox, oy) = ((l - vl as f64 * scala) / 2.0, (a - va as f64 * scala) / 2.0);
            let (px, py) = ((x - ox) / scala, (y - oy) / scala);
            (px >= 0.0 && py >= 0.0 && px < vl as f64 && py < va as f64).then_some((px as i32, py as i32))
        }
    };
    // «Sto scrivendo»: dopo una lettera, Backspace o Canc le frecce su/giù
    // sono tasti veri (cursore); dopo un clic o la rotellina fanno scorrere,
    // perché molte app Android con le frecce spostano solo la selezione.
    let scrivendo = std::rc::Rc::new(std::cell::Cell::new(false));
    let inizio = Arc::new(Mutex::new((0.0, 0.0)));
    {
        let (tx, pos, inizio, scrivendo) = (tx.clone(), posizione.clone(), inizio.clone(), scrivendo.clone());
        trascina.connect_drag_begin(move |_, x, y| {
            scrivendo.set(false);
            *inizio.lock().unwrap() = (x, y);
            if let Some((x, y)) = pos(x, y) {
                let _ = tx.send(Comando::Tocco { azione: 0, x, y });
            }
        });
    }
    {
        let (tx, pos, inizio) = (tx.clone(), posizione.clone(), inizio.clone());
        trascina.connect_drag_update(move |_, dx, dy| {
            let (x0, y0) = *inizio.lock().unwrap();
            if let Some((x, y)) = pos(x0 + dx, y0 + dy) {
                let _ = tx.send(Comando::Tocco { azione: 2, x, y });
            }
        });
    }
    {
        let (tx, pos, inizio) = (tx.clone(), posizione.clone(), inizio.clone());
        trascina.connect_drag_end(move |_, dx, dy| {
            let (x0, y0) = *inizio.lock().unwrap();
            let (x, y) = pos(x0 + dx, y0 + dy).unwrap_or((0, 0));
            let _ = tx.send(Comando::Tocco { azione: 1, x, y });
        });
    }
    immagine.add_controller(trascina);

    // Rotellina (e scorrimento a due dita del touchpad) nel punto del puntatore.
    // La posizione dell'evento è relativa alla finestra: si segue il puntatore
    // sopra l'immagine.
    let puntatore = Arc::new(Mutex::new(None::<(f64, f64)>));
    let movimento = gtk::EventControllerMotion::new();
    {
        let p = puntatore.clone();
        movimento.connect_motion(move |_, x, y| *p.lock().unwrap() = Some((x, y)));
    }
    immagine.add_controller(movimento);
    let rotellina = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
    {
        let (tx, pos, scrivendo) = (tx.clone(), posizione.clone(), scrivendo.clone());
        rotellina.connect_scroll(move |controllo, dx, dy| {
            scrivendo.set(false);
            let Some((px, py)) = *puntatore.lock().unwrap() else {
                return gtk::glib::Propagation::Proceed;
            };
            // Ctrl + rotellina: zoom, come nei programmi del PC.
            if controllo.current_event_state().contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                if let Some((x, y)) = pos(px, py)
                    && dy != 0.0
                {
                    let _ = tx.send(Comando::Zoom { x, y, ingrandisci: dy < 0.0 });
                }
                return gtk::glib::Propagation::Stop;
            }
            if let Some((x, y)) = pos(px, py) {
                // GTK: dy positivo = giù; Android: positivo = su.
                let _ = tx.send(Comando::Scorri { x, y, orizzontale: dx as f32, verticale: -dy as f32 });
            }
            gtk::glib::Propagation::Stop
        });
    }
    immagine.add_controller(rotellina);

    // Pizzico a due dita sul touchpad: un passo di zoom ogni 15% di scala.
    let pizzico = gtk::GestureZoom::new();
    {
        let (tx, pos) = (tx.clone(), posizione.clone());
        let ultimo = std::rc::Rc::new(std::cell::Cell::new(1.0f64));
        let u = ultimo.clone();
        pizzico.connect_begin(move |_, _| u.set(1.0));
        pizzico.connect_scale_changed(move |gesto, scala| {
            let passo = scala / ultimo.get();
            if (0.87..=1.15).contains(&passo) {
                return;
            }
            ultimo.set(scala);
            if let Some((cx, cy)) = gesto.bounding_box_center()
                && let Some((x, y)) = pos(cx, cy)
            {
                let _ = tx.send(Comando::Zoom { x, y, ingrandisci: passo > 1.0 });
            }
        });
    }
    immagine.add_controller(pizzico);

    // Fase di cattura: i tasti arrivano qui prima che ai pulsanti della barra.
    let tasti = gtk::EventControllerKey::new();
    tasti.set_propagation_phase(gtk::PropagationPhase::Capture);
    {
        let tx = tx.clone();
        let (immagine_tasti, dimensione_tasti, scrivendo) = (immagine.clone(), dimensione.clone(), scrivendo.clone());
        let avvisi_tasti = avvisi.clone();
        tasti.connect_key_pressed(move |_, tasto, _, stato| {
            if tasto == gtk::gdk::Key::Escape && crate::configurazione::Preferenze::attuali().esc_indietro {
                let _ = tx.send(Comando::Indietro);
                return gtk::glib::Propagation::Stop;
            }
            // Scorciatoie della finestra (Ruota, Chiudi, Copia screenshot).
            {
                use gtk::gdk::{Key, ModifierType};
                let ctrl = stato.contains(ModifierType::CONTROL_MASK);
                let maiusc = stato.contains(ModifierType::SHIFT_MASK);
                let lettera = tasto.to_lower();
                if ctrl && (lettera == Key::r || lettera == Key::w || (maiusc && lettera == Key::c)) {
                    return gtk::glib::Propagation::Proceed;
                }
            }
            // Ctrl + «+» / «−»: zoom al centro, come nei programmi del PC.
            if stato.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                use gtk::gdk::Key;
                let ingrandisci = match tasto {
                    Key::plus | Key::equal | Key::KP_Add => Some(true),
                    Key::minus | Key::KP_Subtract => Some(false),
                    _ => None,
                };
                if let Some(ingrandisci) = ingrandisci {
                    let (l, a) = *dimensione_tasti.lock().unwrap();
                    if l > 0 {
                        let _ = tx.send(Comando::Zoom { x: l as i32 / 2, y: a as i32 / 2, ingrandisci });
                    }
                    return gtk::glib::Propagation::Stop;
                }
            }
            // Ctrl+V (o Maiusc+Ins): gli appunti del PC vanno nell'app.
            let ctrl = stato.contains(gtk::gdk::ModifierType::CONTROL_MASK);
            let maiusc = stato.contains(gtk::gdk::ModifierType::SHIFT_MASK);
            if (ctrl && tasto.to_lower() == gtk::gdk::Key::v) || (maiusc && tasto == gtk::gdk::Key::Insert) {
                incolla_dal_pc(&immagine_tasti, &avvisi_tasti, &tx);
                return gtk::glib::Propagation::Stop;
            }
            let mut azione = tastiera(tasto, stato);
            match azione {
                Some(Tastiera::Testo(_)) | Some(Tastiera::Tasto(67 | 112, _)) => scrivendo.set(true),
                // Freccia su/giù senza modificatori e senza scrivere: un passo di scorrimento.
                Some(Tastiera::Tasto(codice @ (19 | 20), 0)) if !scrivendo.get() => {
                    azione = Some(Tastiera::Scorrimento(if codice == 19 { 1.0 } else { -1.0 }));
                }
                _ => {}
            }
            match azione {
                Some(Tastiera::Tasto(codice, meta)) => {
                    let _ = tx.send(Comando::Tasto { giu: true, codice, meta });
                }
                Some(Tastiera::Scorciatoia(codice, meta)) => {
                    let _ = tx.send(Comando::Tasto { giu: true, codice, meta });
                    let _ = tx.send(Comando::Tasto { giu: false, codice, meta });
                }
                Some(Tastiera::Testo(c)) => {
                    let _ = tx.send(Comando::Testo(c.to_string()));
                }
                Some(azione @ (Tastiera::Pagina(_) | Tastiera::Scorrimento(_))) => {
                    // Uno scatto di rotellina scorre circa 64 dp (= punti):
                    // una pagina è l'80% dell'altezza della finestra.
                    let (verso, scatti) = match azione {
                        Tastiera::Pagina(v) => (v, (immagine_tasti.height() as f32 * 0.8 / 64.0).clamp(1.0, 16.0)),
                        Tastiera::Scorrimento(v) => (v, 1.0),
                        _ => unreachable!(),
                    };
                    let (l, a) = *dimensione_tasti.lock().unwrap();
                    if l > 0 {
                        let _ = tx.send(Comando::Scorri {
                            x: l as i32 / 2,
                            y: a as i32 / 2,
                            orizzontale: 0.0,
                            verticale: verso * scatti,
                        });
                    }
                }
                None => return gtk::glib::Propagation::Proceed,
            }
            gtk::glib::Propagation::Stop
        });
    }
    {
        let tx = tx.clone();
        tasti.connect_key_released(move |_, tasto, _, stato| {
            if let Some(Tastiera::Tasto(codice, meta)) = tastiera(tasto, stato)
                && !(matches!(codice, 19 | 20) && meta == 0 && !scrivendo.get())
            {
                let _ = tx.send(Comando::Tasto { giu: false, codice, meta });
            }
        });
    }
    match tasti_su {
        Some(w) => w.add_controller(tasti),
        None => {
            // Nel drawer: i tasti arrivano solo dopo un clic sull'immagine.
            immagine.set_focusable(true);
            let fuoco = gtk::GestureClick::new();
            fuoco.connect_pressed(|g, _, _, _| {
                if let Some(w) = g.widget() {
                    w.grab_focus();
                }
            });
            immagine.add_controller(fuoco);
            tasti.set_propagation_phase(gtk::PropagationPhase::Bubble);
            immagine.add_controller(tasti);
        }
    }
    let tasto_mouse = gtk::GestureClick::new();
    tasto_mouse.set_button(8);
    {
        let tx = tx.clone();
        tasto_mouse.connect_pressed(move |_, _, _, _| {
            let _ = tx.send(Comando::Indietro);
        });
    }
    immagine.add_controller(tasto_mouse);

    // Clic destro = pressione lunga del dito: seleziona la parola e apre il
    // menu di Android (Copia, Seleziona tutto…).
    let clic_destro = gtk::GestureClick::new();
    clic_destro.set_button(gtk::gdk::BUTTON_SECONDARY);
    {
        let (tx, pos) = (tx.clone(), posizione.clone());
        clic_destro.connect_pressed(move |_, _, x, y| {
            if let Some((x, y)) = pos(x, y) {
                let _ = tx.send(Comando::PressioneLunga { x, y });
            }
        });
    }
    immagine.add_controller(clic_destro);

    // Errori e avvisi di GStreamer sul terminale.
    if let Some(bus) = pipeline.bus()
        && let Ok(guardia) = bus.add_watch_local(|_, messaggio| {
            use gst::MessageView;
            match messaggio.view() {
                MessageView::Error(e) => eprintln!("[gstreamer] errore: {} ({:?})", e.error(), e.debug()),
                MessageView::Warning(w) => eprintln!("[gstreamer] avviso: {} ({:?})", w.error(), w.debug()),
                _ => {}
            }
            gtk::glib::ControlFlow::Continue
        })
    {
        std::mem::forget(guardia);
    }

    if pipeline.set_state(gst::State::Playing).is_err() {
        eprintln!("la pipeline video non parte");
    }

    let pacchetto = pacchetto.to_string();
    // Ogni secondo: messaggio delle schermate protette e (con PHONESTRA_DEBUG)
    // fotogrammi disegnati e scartati dal PC.
    let protetta_segnalata = Protetta::default();
    {
        let (f, protetta) = (immagine.downgrade(), protetta.clone());
        let segnalata = protetta_segnalata.clone();
        let schermo = schermo.clone();
        let (mut secondi, mut disegnati_prec, mut scartati_prec) = (0u32, 0u64, 0u64);
        gtk::glib::timeout_add_seconds_local(1, move || {
            if f.upgrade().is_none() {
                return gtk::glib::ControlFlow::Break;
            }
            protetta.set_visible(segnalata.lock().unwrap().unwrap_or(false));
            secondi += 1;
            if debug() && secondi.is_multiple_of(5) {
                let stats = schermo.property::<gst::Structure>("stats");
                let disegnati = stats.get::<u64>("rendered").unwrap_or(0);
                let scartati = stats.get::<u64>("dropped").unwrap_or(0);
                eprintln!(
                    "[pc] {:.0} fotogrammi/s disegnati, {} scartati",
                    // Il video ripartito (nuova sessione) azzera i contatori.
                    disegnati.saturating_sub(disegnati_prec) as f32 / 5.0,
                    scartati.saturating_sub(scartati_prec)
                );
                (disegnati_prec, scartati_prec) = (disegnati, scartati);
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    let registrazione: Registrazione = Arc::default();
    let verticale = Verticale::default();
    let stato = StatoVista { sorgente, dimensione, formato, display, verticale: verticale.clone(), protetta: protetta_segnalata, registrazione: registrazione.clone() };
    esecutore().spawn(gestisci(collegamento, pacchetto, stato, rx));
    Ok(Vista { immagine, contenuto: avvisi, tx, pipeline, registrazione, verticale })
}

/// Apre `pacchetto` in una nuova finestra; `nome` è il nome dell'app.
pub fn apri(app: &adw::Application, collegamento: Arc<Collegamento>, pacchetto: &str, nome: &str) -> adw::ApplicationWindow {
    let titolo = adw::WindowTitle::new(nome, t!("collegamento…"));
    let indietro = gtk::Button::from_icon_name("go-previous-symbolic");
    indietro.set_tooltip_text(Some(t!("Indietro (Esc)")));
    // Col fuoco, Invio e Spazio lo attiverebbero invece di arrivare all'app.
    indietro.set_focusable(false);
    let barra = adw::HeaderBar::new();
    barra.set_title_widget(Some(&titolo));
    barra.pack_start(&indietro);
    crate::cassetto::stile();
    let barre = adw::ToolbarView::new();
    barre.add_top_bar(&barra);
    let finestra = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(400)
        .default_height(760)
        .title(nome)
        .content(&barre)
        .css_classes(["phonestra-app"])
        .build();
    crate::cassetto::segui_tema(&finestra);
    let v = match vista(collegamento.clone(), pacchetto, Some(finestra.upcast_ref())) {
        Ok(v) => v,
        Err(e) => {
            titolo.set_subtitle(&e);
            finestra.present();
            return finestra;
        }
    };
    // Collegamento perso: l'ultima immagine resta, sfocata, e i pulsanti ci
    // sono subito (SPECIFICATION §5.7).
    let velo = gtk::Box::builder().orientation(gtk::Orientation::Vertical).css_classes(["velo-app", "chiaro"]).visible(false).build();
    let riconnetti = gtk::Button::builder().label(t!("Riconnetti ora")).css_classes(["suggested-action", "pill"]).build();
    let chiudi = gtk::Button::builder().label(t!("Chiudi")).css_classes(["pill"]).build();
    // Titolo e testo cambiano col motivo: collegamento perso o componente
    // che non parte sul telefono.
    let icona_velo = gtk::Image::builder()
        .icon_name("network-wireless-offline-symbolic")
        .pixel_size(30)
        .halign(gtk::Align::Center)
        .css_classes(["cerchio"])
        .build();
    let titolo_velo = gtk::Label::builder().label(t!("Riconnessione…")).css_classes(["titolo-velo-app"]).build();
    let testo_velo = gtk::Label::builder()
        .label(testo_perso())
        .wrap(true)
        .max_width_chars(32)
        .justify(gtk::Justification::Center)
        .css_classes(["testo-velo-app"])
        .build();
    {
        let centro = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(10).valign(gtk::Align::Center).vexpand(true).build();
        centro.append(&icona_velo);
        centro.append(&titolo_velo);
        centro.append(&testo_velo);
        let pulsanti = gtk::Box::builder().spacing(8).halign(gtk::Align::Center).margin_top(6).build();
        pulsanti.append(&riconnetti);
        pulsanti.append(&chiudi);
        centro.append(&pulsanti);
        velo.append(&centro);
    }
    // La misura della finestra la decide `misura_fissa`, non l'immagine (che
    // chiederebbe i pixel del telefono): l'immagine sta sopra, senza contare.
    let misura_fissa = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let strati = gtk::Overlay::builder().child(&misura_fissa).build();
    strati.add_overlay(&v.contenuto);
    strati.add_overlay(&velo);
    barre.set_content(Some(&strati));

    // App solo verticale (Facebook): finestra a misura fissa, niente
    // ingrandimento né bordi da trascinare (decisione dell'utente). Queste app
    // hanno una forma sola: allargate mostrano bande ai lati, e a tutto
    // schermo servirebbe una densità diversa da quella del telefono, che
    // disegnano male. Se la finestra era più larga che alta torna verticale.
    {
        let (f, s, m) = (finestra.downgrade(), strati.downgrade(), misura_fissa.downgrade());
        let verticale = v.verticale.clone();
        let mut prima = false;
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            let (Some(finestra), Some(strati), Some(misura_fissa)) = (f.upgrade(), s.upgrade(), m.upgrade()) else {
                return gtk::glib::ControlFlow::Break;
            };
            let verticale = *verticale.lock().unwrap() == Some(true);
            if verticale && (finestra.is_maximized() || finestra.is_fullscreen()) {
                // Prima si torna alla misura normale, poi si fissa.
                finestra.unmaximize();
                finestra.unfullscreen();
                return gtk::glib::ControlFlow::Continue;
            }
            if verticale != prima {
                prima = verticale;
                if verticale {
                    let (l, a) = (strati.width(), strati.height());
                    // 9:16: la forma verticale più larga che le app accettano senza bande.
                    let l = if l > a { a * 9 / 16 } else { l };
                    misura_fissa.set_size_request(l, a);
                    finestra.set_resizable(false);
                } else {
                    misura_fissa.set_size_request(-1, -1);
                    finestra.set_resizable(true);
                }
            }
            gtk::glib::ControlFlow::Continue
        });
    }

    // Screenshot, Registra e (per le app) il menu ⋮, a destra.
    let registra_icona = gtk::Image::from_icon_name("media-record-symbolic");
    let registra_tempo = gtk::Label::new(None);
    let registra_pillola = gtk::Box::builder().spacing(6).build();
    registra_pillola.append(&gtk::Box::builder().valign(gtk::Align::Center).css_classes(["punto-rec"]).build());
    registra_pillola.append(&registra_tempo);
    let registra_dentro = gtk::Stack::new();
    registra_dentro.add_named(&registra_icona, Some("icona"));
    registra_dentro.add_named(&registra_pillola, Some("pillola"));
    let registra = gtk::Button::builder().child(&registra_dentro).tooltip_text(t!("Registra lo schermo")).focusable(false).css_classes(["registra"]).build();
    let screenshot = gtk::Button::builder().icon_name("camera-photo-symbolic").tooltip_text(t!("Screenshot (salvato e copiato)")).focusable(false).build();
    let v = std::rc::Rc::new(v);
    {
        let menu = gtk::Box::builder().orientation(gtk::Orientation::Vertical).spacing(2).build();
        let popover = gtk::Popover::new();
        let voce = |icona: &str, testo: &str, tasti: &str, azione: &str| {
            let riga = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            riga.append(&gtk::Image::from_icon_name(icona));
            riga.append(&gtk::Label::builder().label(testo).xalign(0.0).hexpand(true).build());
            riga.append(&gtk::Label::builder().label(tasti).css_classes(["dim-label", "caption"]).build());
            let b = gtk::Button::builder().child(&riga).action_name(azione).css_classes(["flat"]).build();
            let p = popover.downgrade();
            b.connect_clicked(move |_| {
                if let Some(p) = p.upgrade() {
                    p.popdown();
                }
            });
            menu.append(&b);
        };
        voce("object-rotate-right-symbolic", t!("Ruota"), "Ctrl+R", "win.ruota");
        voce("edit-copy-symbolic", t!("Copia screenshot"), t!("Ctrl+Maiusc+C"), "win.copia-screenshot");
        menu.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        voce("window-close-symbolic", t!("Chiudi app"), "Ctrl+W", "win.chiudi-app");
        popover.set_child(Some(&menu));
        barra.pack_end(&gtk::MenuButton::builder().icon_name("view-more-symbolic").tooltip_text(t!("Altri comandi")).popover(&popover).build());
    }
    barra.pack_end(&registra);
    barra.pack_end(&screenshot);

    // Azioni della finestra, con le scorciatoie.
    let azione = |nome: &str, fai: Box<dyn Fn()>| {
        let a = gtk::gio::SimpleAction::new(nome, None);
        a.connect_activate(move |_, _| fai());
        finestra.add_action(&a);
    };
    {
        let (f, v2) = (finestra.downgrade(), v.clone());
        azione(
            "ruota",
            Box::new(move || {
                // Le app in finestra girano come la finestra: si scambiano i lati.
                if let Some(f) = f.upgrade()
                    && !f.is_maximized()
                    && !v2.registra()
                {
                    f.set_default_size(f.height(), f.width());
                }
            }),
        );
        let (v2, nome_app) = (v.clone(), nome.to_string());
        azione("screenshot", Box::new(move || salva_screenshot(&v2, &nome_app, true)));
        let (v2, nome_app) = (v.clone(), nome.to_string());
        azione("copia-screenshot", Box::new(move || salva_screenshot(&v2, &nome_app, false)));
        let f = finestra.downgrade();
        azione(
            "chiudi-app",
            Box::new(move || {
                if let Some(f) = f.upgrade() {
                    f.close();
                }
            }),
        );
        let scorciatoie = gtk::ShortcutController::new();
        scorciatoie.set_scope(gtk::ShortcutScope::Managed);
        for (tasti, nome) in [("<Control>r", "win.ruota"), ("<Control><Shift>c", "win.copia-screenshot"), ("<Control>w", "win.chiudi-app")] {
            scorciatoie.add_shortcut(gtk::Shortcut::new(gtk::ShortcutTrigger::parse_string(tasti), Some(gtk::NamedAction::new(nome))));
        }
        finestra.add_controller(scorciatoie);
    }
    screenshot.set_action_name(Some("win.screenshot"));
    {
        // Registra: il pulsante diventa «● 0:42»; un altro clic ferma e salva.
        let (v2, nome_app, dentro, tempo) = (v.clone(), nome.to_string(), registra_dentro.clone(), registra_tempo.clone());
        registra.connect_clicked(move |b| {
            if v2.registra() {
                let (v3, b, dentro) = (v2.clone(), b.clone(), dentro.clone());
                gtk::glib::spawn_future_local(async move {
                    if let Some(percorso) = v3.ferma_registrazione().await {
                        avvisa(&v3, &t!("Registrazione salvata in {}", mostra_percorso(&percorso)));
                    }
                    dentro.set_visible_child_name("icona");
                    b.remove_css_class("in-corso");
                });
                return;
            }
            let percorso = file_nuovo(gtk::glib::UserDirectory::Videos, &nome_app, "mp4");
            match v2.inizia_registrazione(&percorso) {
                Ok(()) => {
                    dentro.set_visible_child_name("pillola");
                    b.add_css_class("in-corso");
                    tempo.set_label("0:00");
                    let (v3, tempo, inizio) = (v2.clone(), tempo.clone(), std::time::Instant::now());
                    gtk::glib::timeout_add_seconds_local(1, move || {
                        if !v3.registra() {
                            return gtk::glib::ControlFlow::Break;
                        }
                        let s = inizio.elapsed().as_secs();
                        tempo.set_label(&format!("{}:{:02}", s / 60, s % 60));
                        gtk::glib::ControlFlow::Continue
                    });
                }
                Err(e) => avvisa(&v2, &t!("Registrazione non avviata: {}", e)),
            }
        });
    }
    {
        let collegamento = collegamento.clone();
        riconnetti.connect_clicked(move |_| collegamento.riconnetti_ora());
        let f = finestra.downgrade();
        chiudi.connect_clicked(move |_| {
            if let Some(f) = f.upgrade() {
                f.close();
            }
        });
    }

    // Indietro: pulsante (Esc e tasto «indietro» del mouse li gestisce la vista).
    {
        let tx = v.tx.clone();
        indietro.connect_clicked(move |_| {
            let _ = tx.send(Comando::Indietro);
        });
    }

    // Chiusura: la sessione si chiude in sottofondo, la finestra sparisce subito.
    {
        let v2 = v.clone();
        finestra.connect_close_request(move |_| {
            // Una registrazione in corso si chiude bene prima di fermare tutto.
            let v3 = v2.clone();
            gtk::glib::spawn_future_local(async move {
                if let Some(percorso) = v3.ferma_registrazione().await {
                    eprintln!("[finestra] registrazione salvata in {}", percorso.display());
                }
                v3.chiudi();
            });
            gtk::glib::Propagation::Proceed
        });
    }

    // Stato del collegamento e del componente → sottotitolo e fascia.
    {
        let (titolo, velo, immagine) = (titolo.clone(), velo.clone(), v.immagine.clone());
        let mut stato = collegamento.stato();
        let mut guasto = collegamento.guasto();
        let nome_telefono = collegamento.nome.clone();
        gtk::glib::spawn_future_local(async move {
            loop {
                let s = *stato.borrow_and_update();
                let problema = guasto.borrow_and_update().clone();
                let (sottotitolo, perso) = match s {
                    Stato::Cerco => (t!("cerco {}…", nome_telefono), false),
                    Stato::Collegato if problema.is_some() => (t!("Phonestra non parte sul telefono").into(), true),
                    Stato::Collegato => (nome_telefono.clone(), false),
                    Stato::Bloccato => (t!("Telefono bloccato: sbloccalo per continuare").into(), false),
                    Stato::Perso => (t!("scollegato").into(), true),
                    Stato::Chiuso => (t!("chiuso").into(), false),
                };
                match problema.filter(|_| s == Stato::Collegato) {
                    Some(dettaglio) => {
                        icona_velo.set_icon_name(Some("dialog-warning-symbolic"));
                        titolo_velo.set_label(t!("Phonestra non parte sul telefono"));
                        testo_velo.set_label(&testo_guasto(&dettaglio));
                    }
                    None => {
                        icona_velo.set_icon_name(Some("network-wireless-offline-symbolic"));
                        titolo_velo.set_label(t!("Riconnessione…"));
                        testo_velo.set_label(testo_perso());
                    }
                }
                titolo.set_subtitle(&sottotitolo);
                velo.set_visible(perso);
                if perso {
                    immagine.add_css_class("sfocata");
                } else {
                    immagine.remove_css_class("sfocata");
                }
                tokio::select! {
                    r = stato.changed() => if r.is_err() { break },
                    r = guasto.changed() => if r.is_err() { break },
                }
            }
        });
    }

    finestra.present();
    crate::foto::prendi(&finestra, &format!("app-{pacchetto}"));
    finestra
}

/// Un file nuovo in `~/<cartella>/Phonestra`: «<app> 2026-09-27 10.42.05.<estensione>».
fn file_nuovo(cartella: gtk::glib::UserDirectory, nome_app: &str, estensione: &str) -> std::path::PathBuf {
    let base = crate::configurazione::cartella_in(&gtk::glib::user_special_dir(cartella).unwrap_or_else(gtk::glib::home_dir));
    let _ = std::fs::create_dir_all(&base);
    let quando = gtk::glib::DateTime::now_local().and_then(|t| t.format("%Y-%m-%d %H.%M.%S")).map(|s| s.to_string()).unwrap_or_default();
    let nome: String = nome_app.chars().map(|c| if c == '/' { '-' } else { c }).collect();
    base.join(format!("{nome} {quando}.{estensione}"))
}

/// Percorso da mostrare: «Immagini/Phonestra» invece di «/home/…/Immagini/Phonestra/…».
fn mostra_percorso(percorso: &std::path::Path) -> String {
    let cartella = percorso.parent().unwrap_or(percorso);
    match cartella.strip_prefix(gtk::glib::home_dir()) {
        Ok(relativo) => relativo.display().to_string(),
        Err(_) => cartella.display().to_string(),
    }
}

fn avvisa(v: &Vista, testo: &str) {
    v.contenuto.add_toast(adw::Toast::builder().title(testo).timeout(4).build());
}

/// Screenshot alla risoluzione del telefono: negli appunti del PC come
/// immagine e, con `salva`, anche in `~/Immagini/Phonestra` (SPECIFICATION §13).
fn salva_screenshot(v: &Vista, nome_app: &str, salva: bool) {
    let Some(immagine) = v.fotografa() else {
        avvisa(v, t!("Nessuna immagine da fotografare"));
        return;
    };
    v.immagine.clipboard().set_texture(&immagine);
    if !salva {
        avvisa(v, t!("Screenshot copiato negli appunti"));
        return;
    }
    let percorso = file_nuovo(gtk::glib::UserDirectory::Pictures, nome_app, "png");
    match immagine.save_to_png(&percorso) {
        Ok(()) => avvisa(v, &t!("Screenshot in {} e negli appunti", mostra_percorso(&percorso))),
        Err(e) => avvisa(v, &t!("Screenshot non salvato: {}", e)),
    }
}

/// Legge gli appunti del PC e li manda all'app, salvo password e testi troppo
/// lunghi (SPECIFICATION §9).
fn incolla_dal_pc(w: &impl IsA<gtk::Widget>, avvisi: &adw::ToastOverlay, tx: &tokio::sync::mpsc::UnboundedSender<Comando>) {
    let appunti = w.clipboard();
    if appunti.formats().contain_mime_type(crate::appunti::SEGNO_PASSWORD) {
        avvisi.add_toast(adw::Toast::new(t!("Password non inviata al telefono")));
        return;
    }
    let (avvisi, tx) = (avvisi.clone(), tx.clone());
    appunti.read_text_async(None::<&gtk::gio::Cancellable>, move |esito| match esito {
        Ok(Some(testo)) if testo.len() > crate::appunti::MASSIMO => {
            avvisi.add_toast(adw::Toast::new(t!("Testo troppo lungo: usa il trasferimento file")));
        }
        Ok(Some(testo)) if !testo.is_empty() => {
            let _ = tx.send(Comando::Incolla(testo.to_string()));
        }
        _ => avvisi.add_toast(adw::Toast::new(t!("Negli appunti del PC non c'è testo"))),
    });
}

/// Cosa diventa un tasto premuto sul PC.
enum Tastiera {
    /// Tasto speciale: giù alla pressione, su al rilascio.
    Tasto(u32, u32),
    /// Ctrl+lettera: premuto e rilasciato subito.
    Scorciatoia(u32, u32),
    Testo(char),
    /// Pagina su (+1) o giù (-1): scorrimento di una schermata, perché molte
    /// app Android ignorano i tasti di pagina.
    Pagina(f32),
    /// Freccia su (+1) o giù (-1) fuori dalla scrittura: un passo di scorrimento.
    Scorrimento(f32),
}

// Modificatori di Android (`KeyEvent.META_*`).
const META_SHIFT: u32 = 0x1 | 0x40;
const META_CTRL: u32 = 0x1000 | 0x2000;

fn tastiera(tasto: gtk::gdk::Key, stato: gtk::gdk::ModifierType) -> Option<Tastiera> {
    use gtk::gdk::{Key, ModifierType};
    let ctrl = stato.contains(ModifierType::CONTROL_MASK);
    let alt = stato.contains(ModifierType::ALT_MASK);
    let mut meta = 0;
    if stato.contains(ModifierType::SHIFT_MASK) {
        meta |= META_SHIFT;
    }
    if ctrl {
        meta |= META_CTRL;
    }
    // Codici di `KeyEvent.KEYCODE_*`.
    let speciale = match tasto {
        Key::Return | Key::KP_Enter => Some(66),
        Key::BackSpace => Some(67),
        Key::Delete | Key::KP_Delete => Some(112),
        Key::Tab | Key::ISO_Left_Tab => Some(61),
        Key::Left | Key::KP_Left => Some(21),
        Key::Right | Key::KP_Right => Some(22),
        Key::Up | Key::KP_Up => Some(19),
        Key::Down | Key::KP_Down => Some(20),
        Key::Home | Key::KP_Home => Some(122),
        Key::End | Key::KP_End => Some(123),
        Key::Page_Up | Key::KP_Page_Up => return Some(Tastiera::Pagina(1.0)),
        Key::Page_Down | Key::KP_Page_Down => return Some(Tastiera::Pagina(-1.0)),
        _ => None,
    };
    if let Some(codice) = speciale {
        return Some(Tastiera::Tasto(codice, meta));
    }
    if alt {
        return None; // scorciatoie del desktop (Alt+F4…)
    }
    let carattere = tasto.to_unicode()?;
    if ctrl {
        let lettera = carattere.to_ascii_lowercase();
        return lettera.is_ascii_lowercase().then(|| Tastiera::Scorciatoia(29 + (lettera as u32 - 'a' as u32), meta));
    }
    (!carattere.is_control()).then_some(Tastiera::Testo(carattere))
}

/// Tocchi, tasti e comandi verso lo schermo di una sessione: input e video
/// del servizio condiviso del componente nostro.
struct Telefono {
    input: InputNostro,
    video: ComandiVideo,
}

impl Telefono {
    async fn tocco(&mut self, azione: u8, x: i32, y: i32, l: u16, a: u16) -> Result<()> {
        self.input.tocco(azione, x, y, l, a).await
    }

    async fn dito(&mut self, azione: u8, x: i32, y: i32, l: u16, a: u16) -> Result<()> {
        self.input.dito(azione, x, y, l, a).await
    }

    async fn dita_insieme(&mut self, tocchi: &[(i64, u8, i32, i32)], l: u16, a: u16) -> Result<()> {
        self.input.dita_insieme(tocchi, l, a).await
    }

    async fn scorri(&mut self, x: i32, y: i32, l: u16, a: u16, orizzontale: f32, verticale: f32) -> Result<()> {
        self.input.scorri(x, y, l, a, orizzontale, verticale).await
    }

    async fn testo(&mut self, testo: &str) -> Result<()> {
        self.input.testo(testo).await
    }

    async fn incolla(&mut self, testo: &str) -> Result<()> {
        self.input.incolla(testo).await
    }

    async fn tasto(&mut self, azione: u8, codice: u32, meta: u32) -> Result<()> {
        self.input.tasto(azione, codice, meta).await
    }

    async fn indietro(&mut self) -> Result<()> {
        self.input.indietro().await
    }

    async fn avvia_app(&mut self, pacchetto: &str) -> Result<()> {
        self.video.avvia_app(pacchetto).await
    }

    async fn pannello(&mut self, acceso: bool) -> Result<()> {
        self.video.pannello(acceso).await
    }

    async fn ridimensiona(&mut self, l: u16, a: u16) -> Result<()> {
        self.video.ridimensiona(l, a).await
    }

    async fn ricomincia_video(&mut self) -> Result<()> {
        self.video.ricomincia_video().await
    }
}

/// Due dita appoggiate per lo zoom: centro e distanza di ciascun dito dal centro.
struct Pizzico {
    x: i32,
    y: i32,
    raggio: f32,
}

/// Un passo di zoom: se non c'è un pizzico in corso appoggia due dita, poi le
/// allarga (ingrandire) o le stringe di un passo, tutto in un solo invio. Ai
/// bordi solleva le dita e ricomincia, così lo zoom può continuare.
async fn zoom(
    telefono: &mut Telefono,
    pizzico: &mut Option<Pizzico>,
    x: i32,
    y: i32,
    l: u16,
    a: u16,
    ingrandisci: bool,
) -> Result<()> {
    let lato = l.min(a) as f32;
    let (minimo, massimo) = (lato * 0.06, lato * 0.45);
    let passo = if ingrandisci { 1.25 } else { 0.8 };
    let fuori = |r: f32| !(minimo..=massimo).contains(&(r * passo));
    if pizzico.as_ref().is_some_and(|p| fuori(p.raggio) || (p.x - x).abs() + (p.y - y).abs() > 40) {
        solleva(telefono, pizzico, l, a).await?;
    }
    let dita = |x: i32, r: f32| ((x - r as i32).max(0), (x + r as i32).min(l as i32 - 1));
    let mut tocchi = Vec::new();
    let p = match pizzico {
        Some(p) => p,
        None => {
            // Si parte stretti per ingrandire, larghi per rimpicciolire.
            let raggio = if ingrandisci { minimo * 1.5 } else { massimo / 1.5 };
            let (s, d) = dita(x, raggio);
            tocchi.push((10, 0, s, y));
            tocchi.push((11, 0, d, y));
            pizzico.insert(Pizzico { x, y, raggio })
        }
    };
    let (da, verso) = (p.raggio, p.raggio * passo);
    for i in 1..=3 {
        let (s, d) = dita(p.x, da + (verso - da) * i as f32 / 3.0);
        tocchi.push((10, 2, s, p.y));
        tocchi.push((11, 2, d, p.y));
    }
    p.raggio = verso;
    telefono.dita_insieme(&tocchi, l, a).await
}

/// Solleva le dita del pizzico in corso.
async fn solleva(telefono: &mut Telefono, pizzico: &mut Option<Pizzico>, l: u16, a: u16) -> Result<()> {
    if let Some(p) = pizzico.take() {
        let (s, d) = ((p.x - p.raggio as i32).max(0), (p.x + p.raggio as i32).min(l as i32 - 1));
        telefono.dita_insieme(&[(11, 1, d, p.y), (10, 1, s, p.y)], l, a).await?;
    }
    Ok(())
}

/// Lato massimo del display virtuale in pixel (limite dei codificatori video
/// e del carico sul telefono).
/// A 3840×1496 il Galaxy S23+ scendeva a 12–20 fotogrammi/s; a 2560 ne
/// regge 24 stabili.
const LATO_MASSIMO: f32 = 2560.0;

/// Pixel del display virtuale per una finestra di `l`×`a` punti: moltiplicati
/// per la densità, entro [`LATO_MASSIMO`], multipli di 8: il componente
/// arrotonda così il display appena creato, e se la misura del PC differisce
/// anche di 2 pixel scarta in silenzio clic e rotellina.
fn pixel(l: u32, a: u32, moltiplicatore: f32) -> (u32, u32) {
    let (mut pl, mut pa) = (l as f32 * moltiplicatore, a as f32 * moltiplicatore);
    let eccesso = pl.max(pa) / LATO_MASSIMO;
    if eccesso > 1.0 {
        (pl, pa) = (pl / eccesso, pa / eccesso);
    }
    ((pl as u32) & !7, (pa as u32) & !7)
}

/// Misura in punti del display per una finestra di `l`×`a` punti: tutta la
/// finestra oppure, in `colonna` (app solo verticale in una finestra larga),
/// una striscia alta quanto la finestra con la forma del telefono. Così l'app
/// ha la densità del telefono anche a tutto schermo, e niente testi tagliati.
fn misura(l: u32, a: u32, colonna: bool, proporzione: f32) -> (u32, u32) {
    if colonna && l > a {
        (((a as f32 * proporzione) as u32).clamp(150, l), a)
    } else {
        (l, a)
    }
}

/// Quello che la vista condivide con le sue sessioni.
struct StatoVista {
    sorgente: gst_app::AppSrc,
    /// Misura del video, per le coordinate di tocchi e rotellina.
    dimensione: Arc<Mutex<(u32, u32)>>,
    /// Misura della finestra in punti (`None` finché non si è misurata).
    formato: Arc<Mutex<Option<(u32, u32)>>>,
    /// Schermo della sessione attuale (-1 finché non si sa, e per lo specchio).
    display: Arc<std::sync::atomic::AtomicI32>,
    verticale: Verticale,
    protetta: Protetta,
    registrazione: Registrazione,
}

/// Aspetta un collegamento e il suo componente nostro (vivo); `None` se
/// intanto la finestra si chiude (tocchi e tasti arrivati nell'attesa si scartano).
async fn aspetta_telefono(
    collegamento: &Collegamento,
    adb_rx: &mut tokio::sync::watch::Receiver<Option<Adb>>,
    comandi: &mut tokio::sync::mpsc::UnboundedReceiver<Comando>,
) -> Option<(Adb, Condiviso)> {
    let mut componente_rx = collegamento.componente();
    loop {
        let adb = adb_rx.borrow_and_update().clone();
        let componente = componente_rx.borrow_and_update().clone().filter(Condiviso::vivo);
        if let (Some(adb), Some(componente)) = (adb, componente) {
            return Some((adb, componente));
        }
        tokio::select! {
            r = adb_rx.changed() => if r.is_err() { return None },
            r = componente_rx.changed() => if r.is_err() { return None },
            // Il componente nostro caduto resta pubblicato finché non riparte.
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
            c = comandi.recv() => if matches!(c, None | Some(Comando::Chiudi)) { return None },
        }
    }
}

/// Le sessioni della finestra: una per collegamento, finché l'utente non chiude.
async fn gestisci(
    collegamento: Arc<Collegamento>,
    pacchetto: String,
    stato: StatoVista,
    mut comandi: tokio::sync::mpsc::UnboundedReceiver<Comando>,
) {
    let mut adb_rx = collegamento.adb();
    // L'app accetta solo il verticale: il display è una colonna (vedi `misura`).
    let mut colonna = false;
    loop {
        // Aspetta il collegamento; intanto tocchi e tasti si scartano.
        let Some((adb, servizio)) = aspetta_telefono(&collegamento, &mut adb_rx, &mut comandi).await else {
            return;
        };
        let (punti_l, punti_a) = loop {
            if let Some(f) = *stato.formato.lock().unwrap() {
                break f;
            }
            if matches!(comandi.try_recv(), Ok(Comando::Chiudi) | Err(tokio::sync::mpsc::error::TryRecvError::Disconnected)) {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };
        let (punti_l, punti_a) = misura(punti_l, punti_a, colonna, collegamento.proporzione());
        // Stessa densità del telefono, con più pixel in proporzione: alcune
        // app (Facebook) disegnano certi elementi con la densità dello schermo
        // del telefono anche sul display virtuale, e con densità diverse
        // uscirebbero giganti. Il PC poi riduce l'immagine (più nitida).
        let densita = match collegamento.densita() {
            0 => 320,
            d => d,
        };
        let (l, a) = pixel(punti_l, punti_a, densita as f32 / 160.0);
        // Col tetto sui pixel la densità effettiva può essere minore; non per
        // le app solo verticali, che la disegnano male: meglio un display più
        // piccolo della finestra, che il PC ingrandisce.
        let moltiplicatore = if colonna { densita as f32 / 160.0 } else { l as f32 / punti_l as f32 };
        let opzioni = Opzioni {
            display: (l, a, (160.0 * moltiplicatore).round() as u32),
            specchio: pacchetto == SCHERMO,
            ..Opzioni::default()
        };
        collegamento.sessione_aperta();
        let esito = sessione(&collegamento, &adb, &servizio, &pacchetto, &opzioni, moltiplicatore, &stato, &mut colonna, &mut comandi).await;
        collegamento.sessione_chiusa();
        match esito {
            Ok(FineSessione::Chiusa) => return,
            Ok(FineSessione::Ricrea) => {
                // Si aspetta che la finestra smetta di cambiare misura.
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                continue;
            }
            Ok(FineSessione::Caduta) => eprintln!("[finestra] sessione di {pacchetto} interrotta"),
            Err(e) => eprintln!("[finestra] sessione di {pacchetto} non riuscita: {e:#}"),
        }
        // Se il collegamento è lo stesso (si è fermato solo il componente) si
        // riprova dopo un attimo; se è caduto, si aspetta il nuovo.
        tokio::select! {
            _ = adb_rx.changed() => {}
            _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {}
            c = comandi.recv() => if matches!(c, None | Some(Comando::Chiudi)) { return },
        }
    }
}

/// Una sessione avviata: comandi verso il telefono, flusso video, canali da
/// chiudere alla fine, eventi dello schermo.
struct Avviata {
    telefono: Telefono,
    flusso: crate::adb::Canale,
    chiusori: Vec<Chiusore>,
    eventi: crate::video_nostro::Eventi,
}

/// Avvia la sessione col componente nostro: schermo e video dal servizio
/// condiviso, che blocca da sé l'orientamento dello schermo virtuale; tocchi e
/// tasti dal suo modulo input.
async fn avvia_nostra(
    servizio: &Condiviso,
    opzioni: &Opzioni,
    informazioni: Option<&str>,
    display: &Arc<std::sync::atomic::AtomicI32>,
) -> Result<Avviata> {
    let SessioneNostra { video: flusso, comandi: mut video, display: schermo, eventi, .. } =
        SessioneNostra::avvia(servizio, opzioni).await?;
    // Lo specchio resta senza numero: niente schermata protetta né
    // orientamento per lo schermo del telefono.
    if !opzioni.specchio {
        display.store(schermo, std::sync::atomic::Ordering::SeqCst);
    }
    if let Some(app) = informazioni
        && let Err(e) = video.informazioni_app(app).await
    {
        eprintln!("[finestra] informazioni di {app} non aperte: {e:#}");
    }
    let chiusori = vec![flusso.chiusore()];
    let input = InputNostro::new(servizio.mittente(), schermo);
    Ok(Avviata { telefono: Telefono { input, video }, flusso, chiusori, eventi })
}

/// Una sessione: display virtuale con l'app, video, comandi.
#[allow(clippy::too_many_arguments)]
async fn sessione(
    collegamento: &Arc<Collegamento>,
    adb: &Adb,
    servizio: &Condiviso,
    pacchetto: &str,
    opzioni: &Opzioni,
    moltiplicatore: f32,
    stato: &StatoVista,
    colonna: &mut bool,
    comandi: &mut tokio::sync::mpsc::UnboundedReceiver<Comando>,
) -> Result<FineSessione> {
    let StatoVista { sorgente, dimensione, formato, display, verticale, protetta, registrazione } = stato;
    let informazioni = pacchetto.strip_prefix(INFORMAZIONI);
    display.store(-1, std::sync::atomic::Ordering::SeqCst);
    *protetta.lock().unwrap() = None;
    let Avviata { mut telefono, mut flusso, chiusori, mut eventi } = avvia_nostra(servizio, opzioni, informazioni, display).await?;
    if opzioni.specchio {
        collegamento.specchio_aperto();
    }
    let preparata = async {
        if informazioni.is_none() && !opzioni.specchio {
            telefono.avvia_app(pacchetto).await?;
        }
        // Il pannello fisico si spegne, salvo che l'utente stia usando il
        // telefono in mano (SPECIFICATION §5.9).
        if !collegamento.pannello_a_mano() {
            telefono.pannello(false).await?;
        }
        anyhow::Ok(())
    }
    .await;
    if let Err(e) = preparata {
        let _ = telefono.video.chiudi(false).await;
        chiudi_tutti(chiusori).await;
        return Err(e);
    }

    // Video in un compito a sé: una lettura interrotta a metà perderebbe byte.
    let dimensione_video = dimensione.clone();
    let sorgente = sorgente.clone();
    let registrazione_video = registrazione.clone();
    let mut video = tokio::spawn(async move {
        let mut parametri: Vec<u8> = Vec::new();
        // Misure per la diagnosi degli scatti (con PHONESTRA_DEBUG).
        let (mut conteggio, mut byte, mut pausa_max) = (0u32, 0usize, std::time::Duration::ZERO);
        let (mut inizio, mut ultimo) = (std::time::Instant::now(), std::time::Instant::now());
        // Per la pausa più lunga: intervallo tra gli orari del telefono (pts).
        // Se è simile alla pausa, il telefono non ha prodotto fotogrammi; se è
        // molto più corto, è la rete ad averli trattenuti.
        let (mut pts_prec, mut pts_della_pausa) = (None::<u64>, 0u64);
        loop {
            let pacchetto = leggi_pacchetto(&mut flusso).await?;
            if debug() {
                let adesso = std::time::Instant::now();
                let pausa = adesso - ultimo;
                if let Pacchetto::Dati { pts, config: false, .. } = &pacchetto {
                    if pausa > pausa_max {
                        pts_della_pausa = pts_prec.map_or(0, |p| pts.saturating_sub(p) / 1000);
                    }
                    pts_prec = Some(*pts);
                }
                pausa_max = pausa_max.max(pausa);
                ultimo = adesso;
                conteggio += 1;
                if let Pacchetto::Dati { dati, .. } = &pacchetto {
                    byte += dati.len();
                }
                if adesso - inizio >= std::time::Duration::from_secs(5) {
                    let s = (adesso - inizio).as_secs_f32();
                    eprintln!(
                        "[video] {:.0} fotogrammi/s, {:.0} KB/s, pausa massima {} ms (sul telefono {} ms)",
                        conteggio as f32 / s,
                        byte as f32 / 1024.0 / s,
                        pausa_max.as_millis(),
                        pts_della_pausa,
                    );
                    (conteggio, byte, pausa_max, inizio) = (0, 0, std::time::Duration::ZERO, adesso);
                }
            }
            match pacchetto {
                Pacchetto::Dimensione { larghezza, altezza } => {
                    if debug() {
                        eprintln!("[video] nuova dimensione {larghezza}×{altezza}");
                    }
                    *dimensione_video.lock().unwrap() = (larghezza, altezza);
                }
                // I parametri del codec (SPS/PPS) vanno uniti al fotogramma successivo.
                Pacchetto::Dati { config: true, dati, .. } => parametri = dati,
                Pacchetto::Dati { mut dati, chiave, .. } => {
                    let completo = chiave || !parametri.is_empty();
                    if !parametri.is_empty() {
                        let mut unito = std::mem::take(&mut parametri);
                        unito.append(&mut dati);
                        dati = unito;
                    }
                    // Registrazione: una copia del fotogramma, dal primo completo.
                    if let Some(r) = registrazione_video.lock().unwrap().as_mut() {
                        r.iniziata |= completo;
                        if r.iniziata {
                            let _ = r.sorgente.push_buffer(gst::Buffer::from_slice(dati.clone()));
                        }
                    }
                    if sorgente.push_buffer(gst::Buffer::from_mut_slice(dati)).is_err() {
                        return anyhow::Ok(()); // pipeline fermata: finestra chiusa
                    }
                }
            }
        }
    });

    // Pizzico in corso (dita appoggiate): si solleva 300 ms dopo l'ultimo scatto.
    let mut pizzico: Option<Pizzico> = None;
    let mut fine_pizzico: Option<tokio::time::Instant> = None;
    let mut collegato = collegamento.adb();
    let (orientamento_tx, mut orientamento) = tokio::sync::watch::channel(None::<bool>);
    // Il telefono ha chiuso la sessione da sé.
    let finita = Arc::new(tokio::sync::Notify::new());
    // Orientamento e schermata protetta li manda il telefono all'inizio e
    // quando cambiano.
    let domande = {
        let (protetta, finita) = (protetta.clone(), finita.clone());
        let specchio = opzioni.specchio;
        tokio::spawn(async move {
            while let Some(e) = eventi.recv().await {
                if debug() {
                    eprintln!("[finestra] evento del telefono: {e:?}");
                }
                match e {
                    Evento::Orientamento { verticale, .. } if !specchio => {
                        orientamento_tx.send_replace(Some(verticale));
                    }
                    Evento::Protetta { protetta: p, .. } if !specchio => *protetta.lock().unwrap() = Some(p),
                    Evento::Fine { motivo } => {
                        eprintln!("[finestra] il telefono ha chiuso la sessione: {motivo}");
                        break;
                    }
                    _ => {}
                }
            }
            finita.notify_one();
        })
    };
    let fine = loop {
        tokio::select! {
            // Orientamento dell'app: se in una finestra larga la forma del
            // display deve cambiare, lo si ricrea.
            Ok(()) = orientamento.changed() => {
                let Some(vuole) = *orientamento.borrow_and_update() else { continue };
                *verticale.lock().unwrap() = Some(vuole);
                let larga = formato.lock().unwrap().is_some_and(|(l, a)| l > a);
                if vuole != *colonna && registrazione.lock().unwrap().is_none() {
                    *colonna = vuole;
                    if larga {
                        if debug() {
                            eprintln!("[finestra] {}: display da ricreare", if vuole { "app solo verticale, colonna" } else { "tutta la finestra" });
                        }
                        break FineSessione::Ricrea;
                    }
                }
            }
            // Il video si ferma da sé solo quando la finestra ha già fermato la
            // riproduzione (chiusura): conta come chiusura anche se il comando
            // «chiudi» non è ancora arrivato. Altrimenti è il flusso a essere caduto.
            r = &mut video => break match r {
                Ok(Ok(())) => FineSessione::Chiusa,
                _ => FineSessione::Caduta,
            },
            // Collegamento sostituito o perso (lo decide il Collegamento).
            _ = async { collegato.wait_for(|a| a.is_none()).await.is_ok() } => break FineSessione::Caduta,
            _ = finita.notified() => break FineSessione::Caduta,
            _ = async { tokio::time::sleep_until(fine_pizzico.unwrap()).await }, if fine_pizzico.is_some() => {
                fine_pizzico = None;
                let (l, a) = *dimensione.lock().unwrap();
                if solleva(&mut telefono, &mut pizzico, l as u16, a as u16).await.is_err() {
                    break FineSessione::Caduta;
                }
            }
            c = comandi.recv() => {
                // L'utente torna a usare il telefono dal PC: se lo aveva in
                // mano, ora il pannello si rispegne.
                let dal_pc = matches!(
                    c,
                    Some(
                        Comando::Tocco { azione: 0, .. }
                            | Comando::Tasto { giu: true, .. }
                            | Comando::Scorri { .. }
                            | Comando::Testo(_)
                            | Comando::Incolla(_)
                            | Comando::Zoom { .. }
                            | Comando::PressioneLunga { .. }
                            | Comando::Indietro
                    )
                );
                if dal_pc && collegamento.usa_dal_pc() {
                    if telefono.pannello(false).await.is_err() {
                        break FineSessione::Caduta;
                    }
                    eprintln!("[finestra] usato dal PC: pannello spento");
                }
                let inviato = match c {
                    None | Some(Comando::Chiudi) => break FineSessione::Chiusa,
                    Some(Comando::Tocco { azione, x, y }) => {
                        let (l, a) = *dimensione.lock().unwrap();
                        if debug() {
                            eprintln!("[finestra] tocco {azione} a {x},{y} su {l}×{a}");
                        }
                        telefono.tocco(azione, x, y, l as u16, a as u16).await
                    }
                    Some(Comando::Indietro) => telefono.indietro().await,
                    Some(Comando::Zoom { x, y, ingrandisci }) => {
                        let (l, a) = *dimensione.lock().unwrap();
                        let esito = zoom(&mut telefono, &mut pizzico, x, y, l as u16, a as u16, ingrandisci).await;
                        fine_pizzico = Some(tokio::time::Instant::now() + std::time::Duration::from_millis(300));
                        esito
                    }
                    Some(Comando::PressioneLunga { x, y }) => {
                        let (l, a) = *dimensione.lock().unwrap();
                        let (l, a) = (l as u16, a as u16);
                        match telefono.dito(0, x, y, l, a).await {
                            Ok(()) => {
                                // Android riconosce la pressione lunga dopo ~500 ms.
                                tokio::time::sleep(std::time::Duration::from_millis(650)).await;
                                telefono.dito(1, x, y, l, a).await
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Some(Comando::Scorri { x, y, orizzontale, verticale }) => {
                        let (l, a) = *dimensione.lock().unwrap();
                        telefono.scorri(x, y, l as u16, a as u16, orizzontale, verticale).await
                    }
                    // Android inietta come testo solo l'ASCII: il resto (à è ì ò ù…)
                    // passa dagli appunti del telefono.
                    Some(Comando::Testo(t)) if t.is_ascii() => telefono.testo(&t).await,
                    Some(Comando::Testo(t)) | Some(Comando::Incolla(t)) => {
                        collegamento.ricorda_inviato(&t);
                        telefono.incolla(&t).await
                    }
                    Some(Comando::Tasto { giu, codice, meta }) => telefono.tasto(if giu { 0 } else { 1 }, codice, meta).await,
                    Some(Comando::RicominciaVideo) => telefono.ricomincia_video().await,
                    // Durante la registrazione il display non cambia misura (§13).
                    Some(Comando::Ridimensiona { .. }) if registrazione.lock().unwrap().is_some() => Ok(()),
                    // Lo schermo del telefono ha la sua misura: la finestra lo scala.
                    Some(Comando::Ridimensiona { .. }) if opzioni.specchio => Ok(()),
                    Some(Comando::Ridimensiona { larghezza, altezza }) => {
                        let (larghezza, altezza) = misura(larghezza, altezza, *colonna, collegamento.proporzione());
                        // Col tetto sui pixel (finestra molto grande) la
                        // densità del display dovrebbe cambiare, ma con un
                        // display già creato non si può: se la scala giusta si
                        // allontana di oltre il 15% da quella del display, lo
                        // si ricrea. Altrimenti l'app si vedrebbe gigante.
                        let ideale = match collegamento.densita() {
                            0 => 2.0,
                            d => d as f32 / 160.0,
                        };
                        let giusta = pixel(larghezza, altezza, ideale).0 as f32 / larghezza.max(1) as f32;
                        if (giusta / moltiplicatore - 1.0).abs() > 0.15 && !opzioni.specchio && !*colonna {
                            if debug() {
                                eprintln!("[finestra] scala {moltiplicatore:.2} → {giusta:.2}: display da ricreare");
                            }
                            break FineSessione::Ricrea;
                        }
                        let (l, a) = pixel(larghezza, altezza, moltiplicatore);
                        if debug() {
                            eprintln!("[finestra] ridimensiona a {larghezza}×{altezza} punti, {l}×{a} pixel");
                        }
                        telefono.ridimensiona(l as u16, a as u16).await
                    }
                };
                if inviato.is_err() {
                    break FineSessione::Caduta;
                }
            }
        }
    };
    video.abort();
    domande.abort();
    let chiusa = matches!(fine, FineSessione::Chiusa);
    let ultima = chiusa && collegamento.sessioni() == 1;
    // Finestra chiusa: l'app si chiude anche sul telefono (via dalle recenti).
    let id = display.load(std::sync::atomic::Ordering::SeqCst);
    // Altrimenti (collegamento caduto, display da ricreare) le app restano
    // nelle recenti e tornano sullo schermo del telefono.
    if let Err(e) = telefono.video.chiudi(chiusa && id >= 0).await {
        eprintln!("[finestra] sessione non chiusa sul telefono: {e:#}");
    }
    // Tolta dalle recenti l'app può continuare a suonare (YouTube, Facebook:
    // il lettore resta vivo senza finestra): se è lei a comandare i tasti
    // multimediali, la si mette in pausa. Le altre app non si toccano.
    if chiusa && informazioni.is_none() && !opzioni.specchio {
        let comando = format!(
            "dumpsys media_session | grep -q {} && cmd media_session dispatch pause",
            crate::azioni::virgolette(&format!("Media button session is {pacchetto}/"))
        );
        let _ = adb.esegui(&comando).await;
    }
    // Ultima sessione: si riaccende il pannello del telefono.
    if ultima && crate::video_nostro::pannello(servizio, true).is_ok() {
        collegamento.pannello_acceso_senza_finestre();
        eprintln!("[finestra] ultima finestra chiusa: pannello acceso");
    }
    chiudi_tutti(chiusori).await;
    Ok(fine)
}

async fn chiudi_tutti(chiusori: Vec<Chiusore>) {
    for c in chiusori {
        let _ = c.chiudi().await;
    }
}
