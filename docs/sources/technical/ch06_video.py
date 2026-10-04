from build import c, code, flow, note, p, rif, steps, table

S1 = p("Every app window is a virtual display on the phone, as large as the window, with the app launched on it. The "
       "phone screen drawn in the drawer is instead the mirror of the main screen. The phone encodes "
       "in H.264 in hardware, the PC decodes with GStreamer.", lead=True) + \
    table(["", "Virtual display (an app's window)", "Mirror (drawer)"], [
        ["How it is created", c("createVirtualDisplay(nome, l, a, dpi, null, flag)") + " with the flags of "
         + c("Sistema.FLAG_PROPOSTI") + ": PUBLIC, PRESENTATION, OWN_CONTENT_ONLY, SUPPORTS_TOUCH, "
         "ROTATES_WITH_CONTENT, DESTROY_CONTENT_ON_REMOVAL, TRUSTED, OWN_DISPLAY_GROUP, OWN_FOCUS, "
         "TOUCH_FEEDBACK_DISABLED", "Hidden static " + c("DisplayManager.createVirtualDisplay") + " (permission "
         + c("CAPTURE_VIDEO_OUTPUT") + ")"],
        ["Size", "The one requested by the PC (720×1280 at 320 dpi if missing), aligned to 8 and to the encoder's "
         "alignment before creating it", "That of the main screen, reduced to 1920 per side"],
        ["Orientation", "Locked: " + c("cmd window set-ignore-orientation-request") + " and " + c("user-rotation lock 0")
         + " on the display", "Follows the phone: the size is re-read every 500 ms; if it rotates, new encoder and new mirror"],
        ["Resizable", "Yes (" + c("VirtualDisplay.resize") + ")", "No: the window scales the image; "
         + c("VIDEO_RIDIMENSIONA") + " replies with an error (" + c("specchio: misura dello schermo del telefono") + ", “mirror: the phone's screen size”)"],
        ["Display number", "In the open reply", "0 (the main screen); no orientation "
         "events, but protected-screen events yes"],
    ], "«TAB» — The two kinds of video session") + \
    p("On the PC the size in pixels comes from " + c("finestra::pixel") + ": the window size in points (1 PC point "
      "= 1 phone dp) times the phone's density, within 2560 per side and in multiples of 8. The display has the "
      "same density as the phone, because some apps (Facebook) draw certain elements with the density of the "
      "real screen, and with different densities they would come out huge. For portrait-only apps in a wide window the "
      "display is a column shaped like the phone (" + c("finestra::misura") + ").")

S2 = p("Video messages sit in the " + c("0x40–0x4f") + " range of the command channel, with a content made of "
       + c("chiave=valore") + " lines. Requests are executed in order on a video thread of the service: the command "
       "channel (heartbeat, input) never waits for video.", lead=True) + \
    table(["Type", "Name", "Request (PC → service)", "Reply"], [
        [c("0x40"), c("VIDEO_APRI"), c("larghezza altezza dpi codec") + " or " + c("specchio=1 lato_massimo codec")
         + "; optional " + c("app=") + ", " + c("informazioni=") + "; test switches, "
         "valid for the whole service: " + c("max_fps=") + ", " + c("priorita=") + ", " + c("protetta="),
         c("id display codec larghezza altezza") + " (size already aligned), plus " + c("avvio=<esito>")
         + " if there was " + c("app=") + " or " + c("informazioni=")],
        [c("0x41"), c("VIDEO_CHIUDI"), c("id [togli_task=1]"), "empty, once closed"],
        [c("0x42"), c("VIDEO_AVVIA_APP"), c("id app=<pacchetto>") + " or " + c("id informazioni=<pacchetto>"), "outcome of the launch"],
        [c("0x43"), c("VIDEO_RIDIMENSIONA"), c("id larghezza altezza"), "size " + c("LxA") + " or unchanged size"],
        [c("0x44"), c("VIDEO_CHIAVE"), c("id"), "empty"],
        [c("0x45"), c("VIDEO_PANNELLO"), c("acceso=0|1"), c("schermi=<quanti>")],
        [c("0x46"), c("VIDEO_EVENTO"), "—", "spontaneous: " + c("evento=<nome> id=<sessione> …")],
    ], "«TAB» — The video messages") + \
    p("The PC waits for the reply only for " + c("APRI") + " and " + c("CHIUDI") + "; the other commands do not wait "
      "for it and an error ends up in the log. " + c("informazioni=<pacchetto>") + " opens the “App info” "
      "(App info) page of Settings instead of the app. The default codec is " + c("h264") + "; the component also accepts "
      + c("h265") + ", used only by the tests.")

S3 = p("Each session has its own channel, " + c("video:<id>") + ", which carries the encoder's packets from the "
       "phone to the PC.", lead=True) + \
    p("The PC opens it right after " + c("VIDEO_APRI") + " (within 10 s). The encoder starts when the channel is "
       "open, so the first packet is the size, then the parameters, then the first keyframe, and nothing is "
       "lost. The PC does not write to it; if it closes it, the session closes (without removing the app from recents). Each "
       "packet has a 12-byte big-endian header:") + \
    table(["Packet", "Header", "Then"], [
        ["New size", c("0x80000000 · larghezza u32 · altezza u32"), "nothing"],
        ["Data", c("pts u64") + " (µs since the first frame; bit 62 = codec parameters, bit 61 = "
         "keyframe) · " + c("lunghezza u32"), "the data in Annex B, as it comes out of " + c("MediaCodec")],
    ], "«TAB» — The packets of the video channel") + \
    p("On the PC side " + c("video_nostro::flusso::leggi_pacchetto") + " reads them in a dedicated task: a read "
      "interrupted halfway inside a " + c("select!") + " would lose bytes. The phone's reader thread writes each "
      "whole packet with a single " + c("write") + ".")

S4 = p(c("Codifica.java") + " takes the first hardware encoder (not an alias) for the type, with the measured values "
       "(" + c("notes/connection-tests.md") + " §43): 8 Mbit/s, 60 frames per second declared, keyframe every 10 s, repeat after "
       "100 ms, real-time priority, limited range, plus " + c("prepend-sps-pps-to-idr-frames") + " so that every "
       "keyframe carries the parameters in front; " + c("max-fps-to-encoder") + " only with the test switch "
       + c("max_fps") + ".", lead=True) + \
    p("If " + c("configure") + " rejects the format, it retries in this order: hardware without "
      + c("prepend-sps-pps-to-idr-frames") + ", then Android's default encoder with and without it.") + \
    p("The keyframe is needed when a window restarts or a recording begins ("
      + c("ricomincia_video") + " → " + c("VIDEO_CHIAVE") + "). It is requested with " + c("REQUEST_SYNC_FRAME")
      + ", without recreating anything: about 0.1 s instead of scrcpy's 1–2 s restarts.") + \
    note("the Qualcomm encoder (" + c("c2.qti.avc.encoder") + ") ignores " + c("repeat-previous-frame-after")
         + ": on a still screen nothing comes out, and the keyframe request would wait for the next "
         "change (a freshly opened window would stay black). If the keyframe does not come out within 80 ms, "
         + c("SessioneVideo") + " detaches and reattaches the encoder's " + c("Surface") + " ("
         + c("VirtualDisplay.setSurface(null)") + " and then its own again), which makes a frame get composed at once; if "
         "still nothing, a second time after another 160 ms.", "Still screen: the forced redraw.")

S5 = p("When the window changes size, the virtual display follows it: it is resized if that is enough, recreated if the "
       "scale changes too much.", lead=True) + steps([
    "<b>The window changes size.</b> A function tied to GTK's redraw sends the new size to the session on "
    "every change, without waiting.",
    "<b>The PC decides.</b> During a recording the size does not change (" + rif("Recording") + "); for the "
    "mirror nothing is done. If, with the 2560-pixel cap, the right scale drifts more than 15% from that "
    "of the display, resizing the display is not enough: the session ends with " + c("FineSessione::Ricrea")
    + " and is recreated after 300 ms, once the window has stopped changing size. Otherwise "
    + c("VIDEO_RIDIMENSIONA") + ".",
    "<b>The phone resizes.</b> One size = one encoder. If the aligned size does not change, nothing happens. If "
    "it changes: the new encoder is prepared, the size is sent to the PC, " + c("VirtualDisplay.resize") + " + "
    + c("setSurface") + ", then the old one is closed; its packets still in flight are discarded.",
]) + p("An input event computed on the old size is discarded by the phone (" + rif("Injection") + ").")

S6 = p(c("EventiApp.java") + " registers a " + c("TaskStackListener") + " as long as there is at least one session. Each event "
       "schedules a check 150 ms later (events arrive in bursts); in addition, a check every 3 s, because a "
       "protected window can appear without task events.", lead=True) + \
    table(["Event", "Pairs", "When", "What the PC does"], [
        [c("orientamento"), c("display verticale=0|1 valore=N"), "At the first check, then when the app switches from "
         "portrait to landscape or vice versa (not on every value change); never for the mirror",
         "Fixed-size 9:16 window or column"],
        [c("protetta"), c("display protetta=0|1"), "At the first check, then when it changes",
         "Message in place of the black image"],
        [c("spostata"), c("task display"), "A task moves to another screen (app opened on the phone)",
         "Nothing (diagnostics only)"],
        [c("rimosso"), c("task"), "A task of the display closes", "Nothing (diagnostics only)"],
        [c("fine"), c("motivo"), "The phone closes the session by itself", "The session restarts as after a drop"],
    ], "«TAB» — The app events") + \
    p("The protected screen is recognized without " + c("dumpsys") + " (" + c("Protetta.java") + "): "
      + c("captureDisplay") + " shrunk to 5 % and " + c("containsSecureLayers()") + ", with a 2 s "
      "timeout. Phonestra does not bypass protections: it shows a message.")

S7 = p("On the PC a session is a " + c("SessioneNostra") + ": the video channel, the commands and the events. "
       + c("finestra::vista") + " connects it to a GStreamer pipeline that draws in the window.", lead=True) + code("""
let SessioneNostra { display, video: mut flusso, mut comandi, mut eventi, .. } =
    SessioneNostra::avvia(&servizio, &Opzioni { display: (l, a, dpi), ..Opzioni::default() }).await?;
comandi.avvia_app("com.android.chrome").await?;
while let Ok(p) = leggi_pacchetto(&mut flusso).await { /* Pacchetto::Dimensione or Pacchetto::Dati */ }
comandi.ridimensiona(l, a).await?;
comandi.ricomincia_video().await?;
while let Some(e) = eventi.recv().await { /* Evento::Orientamento, Protetta, Spostata, Rimosso, Fine */ }
comandi.chiudi(true).await?;   // true = remove from recents (the user closed the window)
""", "rust", "A video session from the PC") + \
    flow([("Video session", "display → encoder", "navy"), ("Packet reader", "tokio task", "blue"),
          ("appsrc", "h264parse", "blue"), ("decodebin", "videoconvert", "blue"),
          ("gtk4paintablesink", "GtkPicture", "light")],
         "«FIG» — The path of a frame, from the phone to the window; from the first keyframe on, a copy goes "
         "to the recording (" + c("h264parse ! mp4mux") + ")") + \
    p(c("finestra::vista") + " holds the pipeline and redoes the session when needed: connection dropped, component "
      "restarted, display to be recreated, orientation changed (" + c("FineSessione::{Chiusa, Ricrea, Caduta}")
      + "). The codec parameters (SPS/PPS) must be merged with the following frame before the " + c("appsrc") + ".")

CHAPTER = ("Video", [
    ("Sessions: virtual display and mirror", S1),
    ("Video messages", S2),
    ("The video:<id> channel", S3),
    ("Encoder and keyframe", S4),
    ("Resizing", S5),
    ("App events", S6),
    ("PC side: from session to window", S7),
])
