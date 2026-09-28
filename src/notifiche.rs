//! Notifiche e stato del telefono (batteria, rete), letti con i permessi della
//! shell (SPECIFICHE §6, §8): `dumpsys notification --noredact` mostra titolo e
//! testo anche delle notifiche private; senza `--noredact` sarebbero nascosti.

/// Comando per le notifiche attive: solo la sezione «Notification List».
pub const COMANDO_NOTIFICHE: &str =
    "dumpsys notification --noredact | sed -n '/^  Notification List:/,/^  [A-Z]/p'";

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
    fn batteria_e_rete() {
        let i = leggi_info("  status: 2\n  level: 59\nWifi is connected to \"CASA\"\n");
        assert_eq!(i, Info { batteria: Some(59), in_carica: true, rete: Some("CASA".into()) });
    }
}
