//! `phonestra-prova video-componente app|schermo [opzioni]`: prova del video
//! col componente nostro, senza bisogno dell'utente (telefono sbloccato).
//!
//! - `app [--app PACCHETTO] [--secondi S] [--codec h264|h265] [--senza-pannello]`:
//!   schermo virtuale 1120×1992 a 448 dpi con l'app (predefinita l'Orologio);
//! - `schermo [--secondi S] [--codec …]`: specchio dello schermo principale.
//!
//! Riceve il video e lo salva in `phonestra-prova.video-componente.<codec>`
//! (flusso Annex B, leggibile con ffprobe), chiede fotogrammi chiave e ne
//! misura il ritardo, ridimensiona (app), spegne e riaccende il pannello
//! (app), stampa eventi e schermata protetta, chiude e controlla che sul
//! telefono non resti niente.

use std::io::Write;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};
use tokio::sync::mpsc;

use super::{Evento, SessioneNostra, Video};
use crate::adb::Adb;
use crate::componente::{self, Componente, Processo};
use crate::sessione::{Opzioni, Pacchetto, leggi_pacchetto, nome_codec};

/// Un pacchetto arrivato, per le misure.
enum Arrivo {
    Misura { quando: Instant, larghezza: u32, altezza: u32 },
    Config { quando: Instant },
    Fotogramma { quando: Instant, chiave: bool, byte: usize },
    Fine(String),
}

struct Prova {
    specchio: bool,
    app: String,
    secondi: u64,
    codec: &'static str,
    pannello: bool,
}

fn leggi_argomenti(argomenti: &[String]) -> Result<Prova> {
    let uso = "video-componente app|schermo [--app P] [--secondi S] [--codec h264|h265] [--senza-pannello]";
    let specchio = match argomenti.first().map(String::as_str) {
        Some("app") => false,
        Some("schermo") => true,
        _ => bail!("uso: {uso}"),
    };
    let mut p = Prova {
        specchio,
        app: "com.sec.android.app.clockpackage".into(),
        secondi: 15,
        codec: "h264",
        pannello: !specchio,
    };
    let mut i = 1;
    while i < argomenti.len() {
        let valore = || argomenti.get(i + 1).cloned().ok_or_else(|| anyhow!("manca il valore di {}", argomenti[i]));
        match argomenti[i].as_str() {
            "--app" => {
                p.app = valore()?;
                i += 1;
            }
            "--secondi" => {
                p.secondi = valore()?.parse()?;
                i += 1;
            }
            "--codec" => {
                p.codec = match valore()?.as_str() {
                    "h265" | "hevc" => "h265",
                    "h264" | "avc" => "h264",
                    c => bail!("codec sconosciuto: {c}"),
                };
                i += 1;
            }
            "--senza-pannello" => p.pannello = false,
            a => bail!("argomento sconosciuto: {a} (uso: {uso})"),
        }
        i += 1;
    }
    Ok(p)
}

/// Aspetta il primo arrivo che soddisfa `vale`, al massimo `limite`; gli
/// altri arrivi intanto si contano in `conta`.
async fn aspetta(
    rx: &mut mpsc::UnboundedReceiver<Arrivo>,
    limite: Duration,
    conta: &mut Conteggi,
    vale: impl Fn(&Arrivo) -> bool,
) -> Option<Instant> {
    let fine = tokio::time::Instant::now() + limite;
    loop {
        let a = tokio::time::timeout_at(fine, rx.recv()).await.ok()??;
        conta.aggiungi(&a);
        if vale(&a) {
            return Some(match a {
                Arrivo::Misura { quando, .. } | Arrivo::Config { quando } | Arrivo::Fotogramma { quando, .. } => quando,
                Arrivo::Fine(_) => Instant::now(),
            });
        }
        if let Arrivo::Fine(e) = &a {
            println!("  flusso video finito: {e}");
            return None;
        }
    }
}

#[derive(Default)]
struct Conteggi {
    fotogrammi: u32,
    chiave: u32,
    config: u32,
    misure: Vec<(u32, u32)>,
    byte: usize,
    /// Arrivo dell'ultimo fotogramma (per sapere se lo schermo era fermo).
    ultimo: Option<Instant>,
}

impl Conteggi {
    fn aggiungi(&mut self, a: &Arrivo) {
        match a {
            Arrivo::Misura { larghezza, altezza, .. } => self.misure.push((*larghezza, *altezza)),
            Arrivo::Config { .. } => self.config += 1,
            Arrivo::Fotogramma { quando, chiave, byte } => {
                self.ultimo = Some(*quando);
                self.fotogrammi += 1;
                self.chiave += *chiave as u32;
                self.byte += byte;
            }
            Arrivo::Fine(_) => {}
        }
    }
}

/// Lascia passare gli arrivi per `durata`, contandoli.
async fn scorri(rx: &mut mpsc::UnboundedReceiver<Arrivo>, durata: Duration, conta: &mut Conteggi) {
    let _ = aspetta(rx, durata, conta, |_| false).await;
}

pub async fn esegui(adb: &Adb, argomenti: &[String]) -> Result<()> {
    let p = leggi_argomenti(argomenti)?;
    let inizio = Instant::now();
    let componente = Componente::avvia(adb).await?;
    println!("servizio avviato in {} ms (pid {})", inizio.elapsed().as_millis(), componente.pid.map_or("?".into(), |p| p.to_string()));
    let mancanti = componente.ciao.mancanti();
    if !mancanti.is_empty() {
        println!("  autotest: mancano {}", mancanti.join(", "));
    }
    let (video, mut altri) = Video::avvia(componente);
    tokio::spawn(async move {
        while let Some(m) = altri.recv().await {
            println!("  (messaggio non del video: tipo {:#04x})", m.tipo);
        }
    });

    let opzioni = Opzioni { display: (1120, 1992, 448), codec: p.codec, specchio: p.specchio, ..Opzioni::default() };
    let t0 = Instant::now();
    let SessioneNostra { codec, display, video: mut flusso, mut comandi, mut eventi, .. } =
        SessioneNostra::avvia(&video, &opzioni).await?;
    println!(
        "sessione {} aperta in {} ms: schermo {display}, codec {}",
        comandi.id(),
        t0.elapsed().as_millis(),
        nome_codec(codec)
    );
    if !p.specchio {
        comandi.avvia_app(&p.app).await?;
        println!("avvio di {} chiesto", p.app);
    }

    // Eventi: stampati appena arrivano e conservati per il riepilogo.
    let (tx_ev, mut rx_ev) = mpsc::unbounded_channel();
    let inizio_eventi = Instant::now();
    tokio::spawn(async move {
        while let Some(e) = eventi.recv().await {
            println!("  [{:5.1} s] evento: {e:?}", inizio_eventi.elapsed().as_secs_f32());
            let _ = tx_ev.send(e);
        }
    });

    // Video in un compito a sé (una lettura interrotta a metà perderebbe byte).
    let file = format!("phonestra-prova.video-componente.{}", nome_codec(codec));
    let mut uscita = std::fs::File::create(&file)?;
    let (tx, mut rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let a = match leggi_pacchetto(&mut flusso).await {
                Ok(Pacchetto::Dimensione { larghezza, altezza }) => Arrivo::Misura { quando: Instant::now(), larghezza, altezza },
                Ok(Pacchetto::Dati { config, chiave, dati, .. }) => {
                    let quando = Instant::now();
                    if let Err(e) = uscita.write_all(&dati) {
                        let _ = tx.send(Arrivo::Fine(format!("scrittura del file: {e}")));
                        return;
                    }
                    if config { Arrivo::Config { quando } } else { Arrivo::Fotogramma { quando, chiave, byte: dati.len() } }
                }
                Err(e) => {
                    let _ = tx.send(Arrivo::Fine(format!("{e:#}")));
                    return;
                }
            };
            if tx.send(a).is_err() {
                return;
            }
        }
    });

    let mut controlli: Vec<(String, bool)> = Vec::new();
    let mut c = Conteggi::default();
    // Primo fotogramma.
    let primo = aspetta(&mut rx, Duration::from_secs(5), &mut c, |a| matches!(a, Arrivo::Fotogramma { .. })).await;
    let misura_iniziale = c.misure.first().copied();
    match primo {
        Some(q) => println!(
            "primo fotogramma dopo {} ms dall'apertura (misura {:?}, parametri {})",
            q.duration_since(t0).as_millis(),
            misura_iniziale,
            c.config
        ),
        None => println!("nessun fotogramma entro 5 s"),
    }
    controlli.push(("misura, parametri e primo fotogramma arrivati".into(), primo.is_some() && misura_iniziale.is_some() && c.config > 0));

    // Fotogrammi chiave a comando.
    scorri(&mut rx, Duration::from_secs(2), &mut c).await;
    let mut ritardi = Vec::new();
    let mut fermi = Vec::new();
    for n in 1..=5 {
        // Schermo fermo: nessun fotogramma negli ultimi 500 ms (l'Orologio è quasi sempre fermo).
        let fermo = c.ultimo.is_none_or(|u| u.elapsed() >= Duration::from_millis(500));
        let chiesto = Instant::now();
        comandi.ricomincia_video().await?;
        let arrivato = aspetta(&mut rx, Duration::from_secs(1), &mut c, |a| matches!(a, Arrivo::Fotogramma { chiave: true, .. })).await;
        match arrivato {
            Some(q) => {
                let ms = q.duration_since(chiesto).as_secs_f64() * 1000.0;
                println!("  fotogramma chiave {n}: dopo {ms:.0} ms{}", if fermo { " (schermo fermo)" } else { "" });
                ritardi.push(ms);
                if fermo {
                    fermi.push(ms);
                }
            }
            None => println!("  fotogramma chiave {n}: non arrivato entro 1 s{}", if fermo { " (schermo fermo)" } else { "" }),
        }
        scorri(&mut rx, Duration::from_millis(700), &mut c).await;
    }
    if !ritardi.is_empty() {
        let medio = ritardi.iter().sum::<f64>() / ritardi.len() as f64;
        let massimo = ritardi.iter().cloned().fold(0.0, f64::max);
        println!("ritardo richiesta→fotogramma chiave (rete compresa): medio {medio:.0} ms, massimo {massimo:.0} ms");
    }
    if !fermi.is_empty() {
        let medio = fermi.iter().sum::<f64>() / fermi.len() as f64;
        println!("  di cui a schermo fermo: {} richieste, medio {medio:.0} ms", fermi.len());
    }
    controlli.push(("5 fotogrammi chiave a comando".into(), ritardi.len() == 5));

    // Ridimensionamento (solo schermo di un'app).
    if !p.specchio {
        let prima = c.misure.len();
        let chiesto = Instant::now();
        comandi.ridimensiona(800, 1400).await?;
        let misura = aspetta(&mut rx, Duration::from_secs(3), &mut c, |a| matches!(a, Arrivo::Misura { .. })).await;
        let dopo = aspetta(&mut rx, Duration::from_secs(3), &mut c, |a| matches!(a, Arrivo::Fotogramma { .. })).await;
        match (misura, dopo) {
            (Some(m), Some(f)) => println!(
                "ridimensionato a {:?}: misura dopo {} ms, primo fotogramma dopo {} ms",
                c.misure.last(),
                m.duration_since(chiesto).as_millis(),
                f.duration_since(chiesto).as_millis()
            ),
            _ => println!("ridimensionamento: misura {} fotogramma {}", misura.is_some(), dopo.is_some()),
        }
        controlli.push(("ridimensionamento a 800×1400".into(), c.misure.len() == prima + 1 && dopo.is_some()));
        // Stessa misura: niente codificatore nuovo, niente pacchetto di misura.
        comandi.ridimensiona(800, 1400).await?;
        scorri(&mut rx, Duration::from_secs(1), &mut c).await;
        controlli.push(("stessa misura: codificatore non ricreato".into(), c.misure.len() == prima + 1));
    }

    // Pannello fisico spento e riacceso.
    if p.pannello {
        match video.pannello_atteso(false).await {
            Ok(r) => println!("pannello spento ({r}): per 3 s lo schermo del telefono è nero"),
            Err(e) => println!("pannello non spento: {e:#}"),
        }
        scorri(&mut rx, Duration::from_secs(3), &mut c).await;
        let acceso = video.pannello_atteso(true).await;
        println!("pannello riacceso: {}", acceso.as_ref().map_or_else(|e| format!("{e:#}"), |r| r.clone()));
        controlli.push(("pannello spento e riacceso".into(), acceso.is_ok()));
    }

    // Il resto del tempo.
    let trascorso = t0.elapsed();
    let resto = Duration::from_secs(p.secondi).saturating_sub(trascorso);
    let da = Instant::now();
    let fotogrammi_prima = c.fotogrammi;
    let byte_prima = c.byte;
    scorri(&mut rx, resto, &mut c).await;
    let s = da.elapsed().as_secs_f64().max(0.001);
    if resto > Duration::from_secs(1) {
        println!(
            "ultimi {:.0} s: {:.1} fotogrammi/s, {:.0} KB/s",
            s,
            (c.fotogrammi - fotogrammi_prima) as f64 / s,
            (c.byte - byte_prima) as f64 / 1024.0 / s
        );
    }
    println!(
        "in tutto: {} fotogrammi ({} chiave), {} parametri, misure {:?}, {} KB → {file}",
        c.fotogrammi,
        c.chiave,
        c.config,
        c.misure,
        c.byte / 1024
    );

    // Eventi ricevuti.
    let mut ricevuti = Vec::new();
    while let Ok(e) = rx_ev.try_recv() {
        ricevuti.push(e);
    }
    let protetta = ricevuti.iter().rev().find_map(|e| match e {
        Evento::Protetta { protetta, .. } => Some(*protetta),
        _ => None,
    });
    println!("schermata protetta: {}", protetta.map_or("nessuna risposta".into(), |p| if p { "sì".to_string() } else { "no".to_string() }));
    controlli.push(("stato della schermata protetta ricevuto".into(), protetta.is_some()));
    if !p.specchio {
        let orientamento = ricevuti.iter().rev().find_map(|e| match e {
            Evento::Orientamento { verticale, valore, .. } => Some((*verticale, *valore)),
            _ => None,
        });
        println!("orientamento: {}", orientamento.map_or("nessuna risposta".into(), |(v, o)| format!("solo verticale {v} (valore {o})")));
        controlli.push(("orientamento dell'app ricevuto".into(), orientamento.is_some()));
    }

    // Chiusura: la sessione (con l'app via dalle recenti), poi il servizio.
    let chiusa = comandi.chiudi(!p.specchio).await;
    println!("sessione chiusa: {}", chiusa.as_ref().map_or_else(|e| format!("{e:#}"), |_| "ok".into()));
    controlli.push(("sessione chiusa dal telefono".into(), chiusa.is_ok()));
    let fine_flusso = aspetta(&mut rx, Duration::from_secs(3), &mut c, |a| matches!(a, Arrivo::Fine(_))).await;
    controlli.push(("canale video chiuso dal telefono".into(), fine_flusso.is_some()));
    if !p.specchio {
        // removeTask è asincrono: il task sparisce qualche centinaio di ms dopo
        // la chiusura (visto sul S23+), quindi si ricontrolla per 3 s.
        let mut rimasti: Vec<String> = Vec::new();
        for _ in 0..15 {
            let stack = adb.esegui("am stack list").await.unwrap_or_default();
            rimasti = stack
                .lines()
                .filter(|r| r.starts_with("RootTask id=") && r.split_whitespace().any(|v| v == format!("displayId={display}")))
                .map(str::to_string)
                .collect();
            if rimasti.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        for r in &rimasti {
            println!("    rimasto: {r}");
        }
        controlli.push((format!("nessun task sullo schermo {display}"), rimasti.is_empty()));
    }
    let schermi = adb.esegui("dumpsys display | grep -c '\"phonestra-'").await.unwrap_or_default();
    controlli.push(("nessuno schermo phonestra-* in dumpsys display".into(), schermi.trim() == "0"));
    let esito = video.chiudi().await?;
    println!("servizio: {esito:?}");
    controlli.push(("servizio uscito con codice 0".into(), esito == Processo::Uscito(0)));
    let limite = Instant::now() + Duration::from_secs(5);
    let residui = loop {
        let r = componente::residui(adb).await?;
        if r.pulito() || Instant::now() >= limite {
            break r;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };
    for p in &residui.processi {
        println!("    rimasto: {}", p.chars().take(110).collect::<String>());
    }
    controlli.push(("nessun processo né jar del componente".into(), residui.pulito()));

    // Il file, letto da ffprobe come lo leggerebbe un programma qualsiasi.
    drop(rx);
    let formato = if nome_codec(codec) == "h265" { "hevc" } else { "h264" };
    match std::process::Command::new("ffprobe")
        .args(["-v", "error", "-f", formato, "-count_frames", "-select_streams", "v:0"])
        .args(["-show_entries", "stream=codec_name,width,height,nb_read_frames", "-of", "default=nw=1", &file])
        .output()
    {
        Ok(o) => {
            let testo = String::from_utf8_lossy(&o.stdout);
            println!("ffprobe {file}: {}", testo.trim().replace('\n', ", "));
            let letti = testo.lines().find_map(|r| r.strip_prefix("nb_read_frames=")).and_then(|n| n.parse::<u32>().ok());
            controlli.push(("file leggibile da ffprobe".into(), o.status.success() && letti.is_some_and(|n| n > 0)));
        }
        Err(e) => println!("ffprobe non avviato: {e}"),
    }

    for (nome, ok) in &controlli {
        println!("  {} {nome}", if *ok { "ok" } else { "NO" });
    }
    if controlli.iter().any(|(_, ok)| !ok) {
        bail!("prova del video non riuscita");
    }
    println!("prova riuscita");
    Ok(())
}
