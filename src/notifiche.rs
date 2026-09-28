//! Notifiche e stato del telefono (batteria, rete), letti con i permessi della
//! shell (SPECIFICHE §6, §8): `dumpsys notification --noredact` mostra titolo e
//! testo anche delle notifiche private; senza `--noredact` sarebbero nascosti.

/// Comando per le notifiche attive: solo la sezione «Notification List».
pub const COMANDO_NOTIFICHE: &str =
    "dumpsys notification --noredact | sed -n '/^  Notification List:/,/^  [A-Z]/p'";

/// Comando per le finestre visibili con i loro flag: serve a riconoscere le
/// schermate protette (FLAG_SECURE), che in cattura risultano nere.
pub const COMANDO_FINESTRE: &str =
    "dumpsys window windows | grep -E '^  Window #|^    mDisplayId=|^      fl=|mHasSurface='";

/// `WindowManager.LayoutParams.FLAG_SECURE`.
const FLAG_SECURE: u32 = 0x2000;

/// Comando per l'orientamento chiesto dalle app, display per display
/// (attività dalla più in alto alla più in basso).
pub const COMANDO_ORIENTAMENTI: &str =
    "dumpsys activity activities | grep -E '^[A-Z]|\\* Task\\{|\\* Hist|requestedOrientation='";

/// Per ogni display con un'app in vista: se l'app accetta solo il verticale
/// (dall'uscita di [`COMANDO_ORIENTAMENTI`]). Conta l'attività in cima al
/// primo gruppo visibile e opaco; uno trasparente (un dialogo) vale solo se
/// sotto non ce n'è uno opaco.
pub fn display_verticali(uscita: &str) -> Vec<(i32, bool)> {
    let mut esito = Vec::new();
    // Display attuale, scelta fatta (da un gruppo opaco), riserva (da uno trasparente).
    let mut display: Option<i32> = None;
    let (mut scelta, mut riserva) = (None::<bool>, None::<bool>);
    // Gruppo visibile in esame (trasparente o no), in attesa della sua prima attività.
    let mut gruppo: Option<bool> = None;
    let mut chiudi = |display: Option<i32>, scelta: Option<bool>, riserva: Option<bool>| {
        if let (Some(d), Some(v)) = (display, scelta.or(riserva)) {
            esito.push((d, v));
        }
    };
    for riga in uscita.lines() {
        if !riga.starts_with(' ') {
            chiudi(display, scelta, riserva);
            display = riga
                .strip_prefix("Display #")
                .and_then(|r| r.split_whitespace().next())
                .and_then(|n| n.parse().ok());
            (scelta, riserva, gruppo) = (None, None, None);
            continue;
        }
        if display.is_none() || scelta.is_some() {
            continue;
        }
        let t = riga.trim_start();
        if t.starts_with("* Task{") {
            gruppo = t.contains(" visible=true").then(|| t.contains("translucent=true"));
        } else if let Some(o) = t.strip_prefix("requestedOrientation=")
            && let Some(trasparente) = gruppo.take()
        {
            let verticale = o.contains("PORTRAIT");
            if trasparente {
                riserva = riserva.or(Some(verticale));
            } else {
                scelta = Some(verticale);
            }
        }
    }
    chiudi(display, scelta, riserva);
    esito
}

/// Display con almeno una finestra protetta visibile (dall'uscita di
/// [`COMANDO_FINESTRE`]).
pub fn display_protetti(uscita: &str) -> Vec<i32> {
    let mut protetti = Vec::new();
    let (mut display, mut flag) = (None::<i32>, 0u32);
    for riga in uscita.lines() {
        let t = riga.trim_start();
        if t.starts_with("Window #") {
            (display, flag) = (None, 0);
        } else if let Some(v) = t.strip_prefix("mDisplayId=") {
            display = v.split_whitespace().next().and_then(|d| d.parse().ok());
        } else if let Some(v) = t.strip_prefix("fl=") {
            flag = u32::from_str_radix(v.trim(), 16).unwrap_or(0);
        } else if t.starts_with("mHasSurface=true")
            && flag & FLAG_SECURE != 0
            && let Some(d) = display
            && !protetti.contains(&d)
        {
            protetti.push(d);
        }
    }
    protetti
}

/// Comando per batteria e rete.
pub const COMANDO_INFO: &str =
    "dumpsys battery | grep -E '^  (level|status):'; cmd wifi status 2>/dev/null | grep -m1 'connected to'";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notifica {
    /// Chiave unica di Android (`utente|pacchetto|id|tag|uid`).
    pub chiave: String,
    pub pacchetto: String,
    pub titolo: String,
    pub testo: String,
    /// Momento della notifica, in millisecondi dal 1970.
    pub quando: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Info {
    pub batteria: Option<u8>,
    pub in_carica: bool,
    /// Nome della rete Wi-Fi del telefono.
    pub rete: Option<String>,
}

/// Notifiche da mostrare, la più recente per prima. Si scartano quelle
/// permanenti (servizi in primo piano, «in corso») e i riepiloghi di gruppo.
pub fn leggi(uscita: &str) -> Vec<Notifica> {
    let mut elenco: Vec<Notifica> = blocchi(uscita).filter_map(|b| notifica(&b)).collect();
    elenco.sort_by_key(|n| std::cmp::Reverse(n.quando));
    elenco
}

/// Le righe di ogni `NotificationRecord(…)`.
fn blocchi(uscita: &str) -> impl Iterator<Item = Vec<&str>> {
    let mut blocchi: Vec<Vec<&str>> = Vec::new();
    for riga in uscita.lines() {
        if riga.trim_start().starts_with("NotificationRecord(") {
            blocchi.push(vec![riga]);
        } else if let Some(b) = blocchi.last_mut() {
            b.push(riga);
        }
    }
    blocchi.into_iter()
}

fn notifica(righe: &[&str]) -> Option<Notifica> {
    let intestazione = righe.first()?;
    let pacchetto = campo_in_riga(intestazione, "pkg=")?.to_string();
    let valore = |nome: &str| righe.iter().find_map(|r| r.trim_start().strip_prefix(nome)).map(str::trim);
    let chiave = valore("key=")?.to_string();
    let flag = valore("flags=").unwrap_or("");
    if ["ONGOING_EVENT", "FOREGROUND_SERVICE", "GROUP_SUMMARY"].iter().any(|f| flag.split('|').any(|x| x == *f)) {
        return None;
    }
    let quando = valore("when=").and_then(|w| w.split('/').next_back()).and_then(|w| w.parse().ok()).unwrap_or(0);
    let titolo = extra(righe, "android.title=").unwrap_or_default();
    let testo = extra(righe, "android.text=").or_else(|| extra(righe, "android.bigText=")).unwrap_or_default();
    if titolo.is_empty() && testo.is_empty() {
        return None;
    }
    Some(Notifica { chiave, pacchetto, titolo, testo, quando })
}

/// `nome=valore` dentro una riga, fino al primo spazio.
fn campo_in_riga<'a>(riga: &'a str, nome: &str) -> Option<&'a str> {
    let inizio = riga.find(nome)? + nome.len();
    riga[inizio..].split_whitespace().next()
}

/// Un valore di `extras` come `android.text=String (ciao)`, anche su più righe.
/// Si prende la prima occorrenza: quella della notifica, non della versione
/// pubblica (che viene dopo).
fn extra(righe: &[&str], nome: &str) -> Option<String> {
    let i = righe.iter().position(|r| r.trim_start().starts_with(nome))?;
    let primo = &righe[i].trim_start()[nome.len()..];
    if primo == "null" {
        return None;
    }
    // «Tipo (valore)»: il valore comincia dopo la prima parentesi.
    let mut testo = primo.split_once(" (").map(|(_, v)| v.to_string())?;
    // Le righe seguenti fanno parte del valore finché non inizia un'altra chiave.
    for r in &righe[i + 1..] {
        let t = r.trim_start();
        if t == "}" || t.split_once('=').is_some_and(|(k, _)| !k.is_empty() && k.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_')) {
            break;
        }
        testo.push('\n');
        testo.push_str(r);
    }
    let testo = testo.strip_suffix(')').unwrap_or(&testo).trim().to_string();
    (!testo.is_empty()).then_some(testo)
}

/// Batteria e rete dall'uscita di [`COMANDO_INFO`].
pub fn leggi_info(uscita: &str) -> Info {
    let mut info = Info::default();
    for riga in uscita.lines() {
        let t = riga.trim();
        if let Some(v) = t.strip_prefix("level:") {
            info.batteria = v.trim().parse().ok();
        } else if let Some(v) = t.strip_prefix("status:") {
            // BatteryManager: 2 in carica, 5 carica completa.
            info.in_carica = matches!(v.trim(), "2" | "5");
        } else if let Some(v) = t.strip_prefix("Wifi is connected to ") {
            info.rete = Some(v.trim().trim_matches('"').to_string());
        }
    }
    info
}

#[cfg(test)]
mod prove {
    use super::*;

    // Struttura reale di `dumpsys notification --noredact` (Android 16), testi inventati.
    const ESEMPIO: &str = "  Notification List:
    NotificationRecord(0x0fffa43f: pkg=android user=UserHandle{-1} id=62 tag=null importance=4 key=-1|android|62|null|1000: Notification(channel=X))
      flags=ONGOING_EVENT|CAN_COLORIZE
      key=-1|android|62|null|1000
      notification=
            when=0/1790438293647
            extras={
                android.title=String (Debug USB collegato)
                android.text=String (Tocca per disattivare)
            }
    NotificationRecord(0x0390135b: pkg=com.esempio.chat user=UserHandle{0} id=7 tag=null importance=4 key=0|com.esempio.chat|7|null|10244: Notification(channel=Y))
      flags=AUTO_CANCEL
      key=0|com.esempio.chat|7|null|10244
      notification=
            when=0/1790438300000
            extras={
                android.title=String (Anna)
                android.reduced.images=Boolean (true)
                android.text=SpannableString (Prima riga
seconda riga (con parentesi))
                android.showWhen=Boolean (true)
            }
      publicNotification=
            extras={
                android.title=String (Nuovo messaggio)
            }
    NotificationRecord(0x01: pkg=com.esempio.posta user=UserHandle{0} id=1 tag=null importance=3 key=0|com.esempio.posta|1|null|10300: Notification(channel=Z))
      flags=0
      key=0|com.esempio.posta|1|null|10300
      notification=
            when=0/1790438200000
            extras={
                android.title=String (Fattura)
                android.text=null
            }
  TimeoutPendingIntent:
";

    #[test]
    fn notifiche_da_mostrare() {
        let n = leggi(ESEMPIO);
        assert_eq!(n.len(), 2, "la notifica permanente va scartata");
        assert_eq!(n[0].pacchetto, "com.esempio.chat");
        assert_eq!(n[0].titolo, "Anna");
        assert_eq!(n[0].testo, "Prima riga\nseconda riga (con parentesi)");
        assert_eq!(n[0].quando, 1790438300000);
        assert_eq!((n[1].titolo.as_str(), n[1].testo.as_str()), ("Fattura", ""));
    }

    #[test]
    fn finestre_protette() {
        // Struttura reale (Android 16), nomi inventati.
        let uscita = "  Window #0 Window{1 u0 com.esempio.password/.Principale}:
    mDisplayId=103 taskId=1 mSession=Session{a 1:u0a1} mClient=x
      fl=81812180
    mHasSurface=true isReadyForDisplay()=true
  Window #1 Window{2 u0 com.esempio.nascosta/.Principale}:
    mDisplayId=104 taskId=2 mSession=Session{b 2:u0a2} mClient=y
      fl=2000
    mHasSurface=false isReadyForDisplay()=false
  Window #2 Window{3 u0 NavigationBar0}:
    mDisplayId=0 mSession=Session{c 3:u0a3} mClient=z
      fl=20040028
    mHasSurface=true isReadyForDisplay()=true
";
        assert_eq!(display_protetti(uscita), vec![103]);
    }

    #[test]
    fn batteria_e_rete() {
        let i = leggi_info("  status: 2\n  level: 59\nWifi is connected to \"CASA\"\n");
        assert_eq!(i, Info { batteria: Some(59), in_carica: true, rete: Some("CASA".into()) });
    }

    #[test]
    fn app_solo_verticali() {
        // Estratto vero (Galaxy S23+): un dialogo trasparente sopra Facebook
        // sul display virtuale 198.
        let uscita = "ACTIVITY MANAGER ACTIVITIES (dumpsys activity activities)
Display #0 (activities from top to bottom):
  * Task{1a47e4d #1 type=home U=0 visible=true visibleRequested=true mode=fullscreen translucent=false sz=1}
    * Task{31c46d #14424 type=home U=0 rootTaskId=1 visible=true visibleRequested=true mode=fullscreen translucent=false sz=1}
      * Hist  #0: ActivityRecord{174108733 u0 com.sec.android.app.launcher/.activities.LauncherActivity t14424}
        requestedOrientation=SCREEN_ORIENTATION_NOSENSOR
  * Task{3bdcd69 #14590 type=standard U=0 visible=false visibleRequested=false mode=fullscreen translucent=true sz=1}
    * Hist  #0: ActivityRecord{235571501 u0 com.freestylelibre.app.it/com.librelink.app.ui.HomeActivity t14590}
      requestedOrientation=SCREEN_ORIENTATION_PORTRAIT
Display #198 (activities from top to bottom):
  * Task{876c7d6 #14725 type=standard U=0 visible=true visibleRequested=true mode=fullscreen translucent=true sz=1
    * Hist  #0: ActivityRecord{146760970 u0 android/com.android.internal.app.ResolverActivity t14725}
      requestedOrientation=SCREEN_ORIENTATION_UNSPECIFIED
  * Task{c9c8784 #14715 type=standard A=10057:com.facebook.katana U=0 visible=true visibleRequested=true mode=fullscreen translucent=false sz=2}
    * Hist  #1: ActivityRecord{176341242 u0 com.facebook.katana/com.facebook.fbreact.fragment.ReactActivity t14715}
      requestedOrientation=SCREEN_ORIENTATION_PORTRAIT
    * Hist  #0: ActivityRecord{218319255 u0 com.facebook.katana/.LoginActivity t14715}
      requestedOrientation=SCREEN_ORIENTATION_LANDSCAPE
Display #200 (activities from top to bottom):
  * Task{c9c8785 #14716 type=standard U=0 visible=true visibleRequested=true mode=fullscreen translucent=false sz=1}
    * Hist  #0: ActivityRecord{1 u0 com.google.android.youtube/.WatchWhileActivity t14716}
      requestedOrientation=SCREEN_ORIENTATION_SENSOR_LANDSCAPE
ActivityTaskSupervisor state:
      * Task{c9c8784 #14715 type=standard U=0 visible=true visibleRequested=true mode=fullscreen translucent=false sz=2}
";
        assert_eq!(display_verticali(uscita), vec![(0, false), (198, true), (200, false)]);
    }
}
