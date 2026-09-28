//! `phonestra-prova`: la tappa 1 da riga di comando, senza `adb`.
//!
//!   phonestra-prova usb        stato del telefono collegato col cavo
//!   phonestra-prova cerca      telefoni col Debug wireless in rete
//!   phonestra-prova prepara    via cavo: Wi-Fi acceso, telefono salvato
//!   phonestra-prova collega    via Wi-Fi, senza indirizzi, al telefono salvato

use std::time::Duration;

use anyhow::{Result, bail};
use phonestra::configurazione::Telefoni;
use phonestra::telefono::Collegamento;
use phonestra::usb::{self, Stato};
use phonestra::{configurazione, rete};

fn main() {
    let comando = std::env::args().nth(1).unwrap_or_default();
    let esito = match comando.as_str() {
        "usb" => usb(),
        "cerca" => cerca(),
        "abbina" => abbina(std::env::args().nth(2).unwrap_or_default(), std::env::args().nth(3)),
        "prepara" => prepara(),
        "collega" => collega(),
        "shell-usb" => shell_usb(std::env::args().skip(2).collect::<Vec<_>>().join(" ")),
        "canali" => canali(),
        "shell" => shell_wifi(std::env::args().skip(2).collect::<Vec<_>>().join(" ")),
        "tocchi" => tocchi(),
        "app" => app(),
        "notifiche" => notifiche(),
        "appunti" => appunti(),
        "sfondo" => sfondo(),
        "banco" => banco(std::env::args().skip(2).collect()),
        "custode" => custode(std::env::args().nth(2).unwrap_or_default()),
        "procedura" => procedura(),
        "audio" => solo_audio(std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(30)),
        "video" => video(
            std::env::args().nth(2).unwrap_or_else(|| "com.sec.android.app.clockpackage".into()),
            std::env::args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(8),
        ),
        _ => {
            eprintln!("uso: phonestra-prova usb | cerca | abbina <codice> [ip:porta] | prepara | procedura | collega | shell-usb <comando>");
            std::process::exit(2);
        }
    };
    if let Err(e) = esito {
        eprintln!("errore: {e:#}");
        std::process::exit(1);
    }
}

fn descrivi_stato(stato: &Stato) -> &'static str {
    match stato {
        Stato::DebugSpento => "Debug USB spento: attiva Opzioni sviluppatore › Debug USB",
        Stato::DebugAttivo => "Debug USB attivo",
        Stato::SoloRicarica => "In «Solo ricarica» col Debug USB spento: scegli Trasferimento file dalla notifica USB",
        Stato::DebugAttivoSoloRicarica => {
            "Debug USB attivo ma in «Solo ricarica»: imposta la modalità USB su Trasferimento file"
        }
    }
}

fn usb() -> Result<()> {
    let telefoni = usb::telefoni();
    if telefoni.is_empty() {
        println!("Nessun telefono collegato col cavo.");
    }
    for t in telefoni {
        println!("{} {} [{}] seriale {} — {}", t.produttore, t.nome, t.porta, t.seriale, descrivi_stato(&t.stato));
    }
    Ok(())
}

fn cerca() -> Result<()> {
    let trovati = rete::cerca(Duration::from_secs(3))?;
    if trovati.is_empty() {
        println!("Nessun telefono in rete (Debug wireless spento, altra rete o telefono bloccato).");
    }
    for t in trovati {
        println!("{}\t{}\t{}", t.seriale, t.indirizzo, t.istanza);
    }
    Ok(())
}

/// Associazione col codice a 6 cifre (SPECIFICHE §5.2): trova la schermata del
/// codice in rete (o usa l'indirizzo dato), associa la chiave di Phonestra e
/// verifica collegandosi via Wi-Fi. Per provarla con una chiave nuova:
/// `XDG_CONFIG_HOME=<cartella vuota> phonestra-prova abbina 123456`.
fn abbina(codice: String, indirizzo: Option<String>) -> Result<()> {
    use phonestra::adb::{Adb, abbina};
    if codice.len() != 6 || !codice.chars().all(|c| c.is_ascii_digit()) {
        bail!("il codice di associazione è di 6 cifre");
    }
    let chiave = configurazione::chiave()?;
    let (indirizzo, seriale) = match indirizzo {
        Some(i) => (i.parse()?, None),
        None => {
            let Some(t) = rete::cerca_abbinamento(Duration::from_secs(4))?.into_iter().next() else {
                bail!("nessuna schermata del codice aperta in rete (Debug wireless › Associa dispositivo con codice di associazione)");
            };
            println!("Schermata del codice trovata: {} ({})", t.indirizzo, t.seriale);
            (t.indirizzo, Some(t.seriale))
        }
    };
    tokio::runtime::Runtime::new()?.block_on(async move {
        let identita = abbina::abbina(indirizzo, &codice, &chiave).await?;
        println!("Associato: il telefono si presenta come «{identita}».");
        let seriale = seriale.or_else(|| identita.strip_prefix("adb-").and_then(|s| s.rsplit_once('-')).map(|(s, _)| s.to_string()));
        let trovati = rete::cerca(Duration::from_secs(4))?;
        let Some(t) = trovati.iter().find(|t| Some(&t.seriale) == seriale.as_ref()).or(trovati.first()) else {
            bail!("associato, ma il telefono non si annuncia per il collegamento (Debug wireless spento?)");
        };
        let adb = Adb::wifi(t.indirizzo, &chiave).await?;
        println!("Collegato via Wi-Fi a {}: {}", t.indirizzo, adb.esegui("getprop ro.product.model").await?.trim());
        Ok(())
    })
}

fn prepara() -> Result<()> {
    let Some(t) = usb::telefoni().into_iter().find(|t| t.stato != Stato::DebugSpento) else {
        bail!("nessun telefono col Debug USB attivo collegato al cavo (prova «phonestra-prova usb»)");
    };
    println!("Collegamento via cavo a {} {}…", t.produttore, t.nome);
    println!("Se sul telefono compare «Consentire il debug USB?», spunta «Consenti sempre» e tocca Consenti.");
    let mut c = Collegamento::usb(t.vendor, t.prodotto)?;
    let telefono = c.descrivi()?;
    println!("Autorizzato: {} ({}, Android {}).", telefono.nome, telefono.modello, telefono.android);
    println!("Attivo il Debug wireless (se il telefono lo chiede, consenti questa rete)…");
    if !c.prepara_wifi()? {
        bail!("il Debug wireless non si è acceso: sul telefono serve «Consenti» per questa rete");
    }
    println!("Verifico il collegamento Wi-Fi…");
    let Some(trovato) = rete::cerca(Duration::from_secs(4))?.into_iter().find(|t| t.seriale == telefono.seriale) else {
        bail!("il telefono non si annuncia in rete: è sulla stessa rete Wi-Fi del PC?");
    };
    if let Err(e) = Collegamento::wifi(trovato.indirizzo).and_then(|mut c| c.shell("echo ok")) {
        if format!("{e:#}").contains("CertificateUnknown") {
            bail!(
                "il telefono ha autorizzato Phonestra solo per questa volta: scollega e ricollega il cavo \
                 e, quando chiede «Consentire il debug USB?», spunta «Consenti sempre da questo computer»"
            );
        }
        return Err(e);
    }
    let mut telefoni = Telefoni::carica()?;
    telefoni.registra(telefono.clone());
    telefoni.salva()?;
    println!("Fatto: il Wi-Fi funziona, puoi scollegare il cavo.");
    println!("Telefono salvato in {}.", configurazione::cartella()?.join("telefoni.toml").display());
    Ok(())
}

/// Solo la finestra «Aggiungi un telefono», per provarla senza toccare il
/// drawer aperto (un'applicazione a sé, con un altro nome).
fn procedura() -> Result<()> {
    use adw::prelude::*;
    let app = adw::Application::builder().application_id("io.github.nic_fio.Phonestra.Prova").build();
    app.connect_activate(|app| {
        phonestra::prepara::apri(app, |t| println!("configurato: {} ({})", t.nome, t.seriale));
    });
    app.run_with_args::<&str>(&[]);
    Ok(())
}

/// Trova in rete il telefono salvato (ripetendo la ricerca se la porta annunciata
/// è già vecchia) e restituisce l'indirizzo che accetta collegamenti.
/// Le app del launcher lette dall'aiutante; le icone finiscono in
/// `phonestra-prova.icone/` per guardarle.
fn app() -> Result<()> {
    use phonestra::adb::Adb;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let inizio = std::time::Instant::now();
        let app = phonestra::app::elenco(&adb, 96).await?;
        let byte: usize = app.iter().map(|a| a.icona.len()).sum();
        println!("{} app in {:.1} s, icone {} KB in tutto", app.len(), inizio.elapsed().as_secs_f32(), byte / 1024);
        std::fs::create_dir_all("phonestra-prova.icone")?;
        for a in &app {
            println!("  {:<28} {}", a.nome, a.pacchetto);
            std::fs::write(format!("phonestra-prova.icone/{}.png", a.pacchetto), &a.icona)?;
        }
        Ok(())
    })
}

/// Prova del custode del tempo di spegnimento: `chiudi` chiude il canale e
/// rilegge il valore; `abbandona` esce senza chiudere (come un PC che sparisce).
fn custode(modo: String) -> Result<()> {
    use phonestra::adb::Adb;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let prima = adb.esegui("settings get system screen_off_timeout").await?;
        let c = adb
            .apri(&format!(
                "exec:trap '' HUP TERM PIPE; settings put system screen_off_timeout 1234000; cat >/dev/null; \
                 settings put system screen_off_timeout {prima}"
            ))
            .await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("prima {prima}, durante {}", adb.esegui("settings get system screen_off_timeout").await?);
        if modo == "abbandona" {
            std::process::exit(0);
        }
        c.chiudi().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("dopo la chiusura {}", adb.esegui("settings get system screen_off_timeout").await?);
        Ok(())
    })
}

/// Notifiche e stato del telefono. Per rispetto della privacy stampa solo
/// pacchetto e lunghezza dei testi, non il loro contenuto.
fn notifiche() -> Result<()> {
    use phonestra::adb::Adb;
    use phonestra::notifiche;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        println!("{:?}", notifiche::leggi_info(&adb.esegui(notifiche::COMANDO_INFO).await?));
        let inizio = std::time::Instant::now();
        let uscita = adb.esegui(notifiche::COMANDO_NOTIFICHE).await?;
        let elenco = notifiche::leggi(&uscita);
        println!("{} notifiche in {} ms ({} KB letti)", elenco.len(), inizio.elapsed().as_millis(), uscita.len() / 1024);
        for n in elenco {
            println!("  {:<40} titolo {} car., testo {} car.", n.pacchetto, n.titolo.chars().count(), n.testo.chars().count());
        }
        Ok(())
    })
}

/// Salva lo sfondo del telefono in `phonestra-prova.sfondo.png`.
fn sfondo() -> Result<()> {
    use phonestra::adb::Adb;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        match phonestra::app::sfondo(&adb, 540).await? {
            Some(png) => {
                std::fs::write("phonestra-prova.sfondo.png", &png)?;
                println!("sfondo salvato ({} KB)", png.len() / 1024);
            }
            None => println!("sfondo non disponibile"),
        }
        Ok(())
    })
}

/// Se gli appunti del telefono sono sensibili (non ne stampa il contenuto).
fn appunti() -> Result<()> {
    use phonestra::adb::Adb;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let inizio = std::time::Instant::now();
        let s = phonestra::app::appunti_sensibili(&adb).await?;
        println!("sensibili: {s:?} ({} ms)", inizio.elapsed().as_millis());
        Ok(())
    })
}

fn indirizzo_telefono() -> Result<std::net::SocketAddr> {
    let telefoni = Telefoni::carica()?;
    let Some(salvato) = telefoni.elenco.first() else {
        bail!("nessun telefono configurato: prima «phonestra-prova prepara» col cavo");
    };
    let indirizzo = rete::indirizzo_attivo(&salvato.seriale, salvato.ultimo_indirizzo)?;
    Telefoni::ricorda_indirizzo(&salvato.seriale, indirizzo)?;
    Ok(indirizzo)
}

/// Diagnosi dei tocchi: Orologio su display virtuale, poi un tocco sulla scheda
/// «Timer» col messaggio di Phonestra e uno con `input tap` di sistema; per
/// ciascuno si contano i fotogrammi che arrivano dopo (schermo che cambia).
fn tocchi() -> Result<()> {
    use phonestra::adb::Adb;
    use phonestra::sessione::{Opzioni, Pacchetto, Sessione, leggi_pacchetto};
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let Sessione { mut video, mut comandi, mut server, .. } = Sessione::avvia(&adb, &Opzioni::default()).await?;
        let contatore = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = contatore.clone();
        let mut registrazione = std::fs::File::create("phonestra-prova.h264")?;
        tokio::spawn(async move {
            use std::io::Write;
            while let Ok(p) = leggi_pacchetto(&mut video).await {
                if let Pacchetto::Dati { config, dati, .. } = p {
                    let _ = registrazione.write_all(&dati);
                    if !config {
                        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        });
        let display = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let d = display.clone();
        tokio::spawn(async move {
            while let Some(b) = server.leggi().await {
                let testo = String::from_utf8_lossy(&b).to_string();
                eprint!("[telefono] {testo}");
                if let Some(i) = testo.find("(id=") {
                    *d.lock().unwrap() = testo[i + 4..].chars().take_while(|c| c.is_ascii_digit()).collect();
                }
            }
        });
        comandi.avvia_app("com.sec.android.app.clockpackage").await?;
        let conta = |etichetta: &str| {
            let n = contatore.swap(0, std::sync::atomic::Ordering::Relaxed);
            println!("{etichetta}: {n} fotogrammi");
        };
        tokio::time::sleep(Duration::from_secs(3)).await;
        conta("avvio");
        tokio::time::sleep(Duration::from_secs(2)).await;
        conta("fermo, senza tocchi");
        // Scheda «Timer» (la quarta in basso) sul display 720×1280.
        comandi.tocco(0, 640, 1205, 720, 1280).await?;
        comandi.tocco(1, 640, 1205, 720, 1280).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        conta("dopo il tocco di Phonestra sul Timer");
        let id = display.lock().unwrap().clone();
        println!("display virtuale: {id}");
        // Scheda «Orologio mondiale» (la seconda) con input di sistema.
        println!("{}", adb.esegui(&format!("input -d {id} tap 300 1205")).await?);
        tokio::time::sleep(Duration::from_secs(2)).await;
        conta("dopo input tap di sistema sull'Orologio mondiale");
        let attivita = adb.esegui("dumpsys activity activities").await?;
        for riga in attivita.lines().filter(|r| r.contains("Display #") || r.contains("clockpackage") && r.contains("ActivityRecord")) {
            println!("  {}", riga.trim());
        }
        Ok(())
    })
}

/// Diagnosi: un comando di shell via Wi-Fi, con il livello ADB proprio.
fn shell_wifi(comando: String) -> Result<()> {
    use phonestra::adb::Adb;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        println!("{}", adb.esegui(&comando).await?);
        Ok(())
    })
}

/// Prova del livello ADB proprio: canali contemporanei e copia di un file.
fn canali() -> Result<()> {
    use phonestra::adb::{Adb, sync};
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        println!("Collegato a {indirizzo}: {}", adb.dispositivo.chars().take(60).collect::<String>());
        // Tre comandi in parallelo sullo stesso collegamento: il lento non blocca gli altri.
        let (lento, modello, versione) = tokio::join!(
            adb.esegui("sleep 2; echo lento-finito"),
            adb.esegui("getprop ro.product.model"),
            adb.esegui("getprop ro.build.version.release"),
        );
        println!("in parallelo: {} / {} / Android {}", lento?, modello?, versione?);
        let contenuto: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        sync::invia(&adb, &contenuto, "/data/local/tmp/phonestra-prova.bin", 0o644).await?;
        let dimensione = adb.esegui("stat -c %s /data/local/tmp/phonestra-prova.bin").await?;
        let impronta = adb.esegui("md5sum /data/local/tmp/phonestra-prova.bin").await?;
        adb.esegui("rm /data/local/tmp/phonestra-prova.bin").await?;
        println!("file copiato: {dimensione} byte, md5 {}", impronta.split_whitespace().next().unwrap_or("?"));
        Ok(())
    })
}

/// Prova della sessione: app su display virtuale, video salvato in
/// `phonestra-prova.<codec>` (flusso grezzo, leggibile con ffprobe/ffmpeg).
fn video(pacchetto: String, secondi: u64) -> Result<()> {
    use phonestra::adb::Adb;
    use phonestra::sessione::{Opzioni, Pacchetto, Sessione, leggi_pacchetto, nome_codec};
    use std::io::Write;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let mut s = Sessione::avvia(&adb, &Opzioni::default()).await?;
        let codec = nome_codec(s.codec);
        println!("Componente avviato su «{}», video {codec}. Avvio {pacchetto}…", s.nome_dispositivo);
        s.comandi.avvia_app(&pacchetto).await?;
        let file = format!("phonestra-prova.{codec}");
        let mut uscita = std::fs::File::create(&file)?;
        let (mut fotogrammi, mut chiave_vista, mut byte) = (0u32, 0u32, 0usize);
        let fine = tokio::time::Instant::now() + Duration::from_secs(secondi);
        while let Ok(p) = tokio::time::timeout_at(fine, leggi_pacchetto(&mut s.video)).await {
            match p? {
                Pacchetto::Dimensione { larghezza, altezza } => println!("display {larghezza}×{altezza}"),
                Pacchetto::Dati { config, chiave, dati, .. } => {
                    if !config {
                        fotogrammi += 1;
                        chiave_vista += chiave as u32;
                    }
                    byte += dati.len();
                    uscita.write_all(&dati)?;
                }
            }
        }
        println!(
            "{secondi} s: {fotogrammi} fotogrammi ({chiave_vista} chiave), {} KB → {file}",
            byte / 1024
        );
        Ok(())
    })
}

/// Banco di prova della fluidità: Chrome su un display virtuale con una
/// pagina che si anima a ogni fotogramma; conta i fotogrammi prodotti dal
/// telefono. `banco <larghezza> <altezza> <dpi> <secondi> [audio] [controllo]`:
/// «audio» aggiunge la cattura audio, «controllo» il controllo periodico
/// (blocco, notifiche, finestre) come fa Phonestra.
/// Solo la cattura dell'audio del telefono per `secondi` (poi si ferma e il
/// telefono torna a suonare da sé): per capire se è lei a disturbare qualcosa
/// sul telefono, per esempio il microfono durante una chiamata.
fn solo_audio(secondi: u64) -> Result<()> {
    gst::init()?;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = phonestra::adb::Adb::wifi(indirizzo, &chiave).await?;
        println!("cattura dell'audio accesa per {secondi} s");
        match tokio::time::timeout(Duration::from_secs(secondi), phonestra::audio::riproduci(&adb)).await {
            Ok(esito) => esito?,
            Err(_) => println!("cattura dell'audio spenta"),
        }
        anyhow::Ok(())
    })
}

fn banco(argomenti: Vec<String>) -> Result<()> {
    use phonestra::adb::Adb;
    use phonestra::sessione::{Opzioni, Pacchetto, Sessione, display_da_messaggio, leggi_pacchetto, togli_dalle_recenti};
    let numero = |i: usize, predefinito: u32| argomenti.get(i).and_then(|v| v.parse().ok()).unwrap_or(predefinito);
    let (l, a, d, secondi) = (numero(0, 1120), numero(1, 1992), numero(2, 448), numero(3, 15));
    let con = |nome: &str| argomenti.iter().any(|x| x == nome);
    let (con_audio, con_controllo) = (con("audio"), con("controllo"));
    // «video»: un filmato vero (immagine ricca di dettagli) invece dell'animazione.
    let (app, url) = if con("facebook") {
        // App solo verticale (colonna nelle finestre larghe).
        ("com.facebook.katana", "https://www.facebook.com/")
    } else if con("video") {
        // «Big Buck Bunny», canale ufficiale di Blender (licenza libera).
        ("com.google.android.youtube", "https://www.youtube.com/watch?v=aqz-KE-bpKQ")
    } else {
        ("com.android.chrome", "https://www.testufo.com/")
    };
    gst::init()?;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let audio = con_audio.then(|| {
            let adb = adb.clone();
            tokio::spawn(async move { phonestra::audio::riproduci(&adb).await })
        });
        let controllo = con_controllo.then(|| {
            let adb = adb.clone();
            tokio::spawn(async move {
                let comando = format!(
                    "dumpsys window | grep -m1 -o 'isKeyguardShowing=[a-z]*'; {}; {}",
                    phonestra::notifiche::COMANDO_NOTIFICHE,
                    phonestra::notifiche::COMANDO_FINESTRE
                );
                loop {
                    let _ = adb.esegui(&comando).await;
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            })
        });
        let opzioni = Opzioni { display: (l, a, d), ..Opzioni::default() };
        let Sessione { video: mut flusso, comandi: mut telefono, mut server, .. } = Sessione::avvia(&adb, &opzioni).await?;
        telefono.avvia_app(app).await?;
        telefono.pannello(false).await?;
        // Numero del display dal messaggio del componente.
        let mut display = None;
        let limite = tokio::time::Instant::now() + Duration::from_secs(5);
        while display.is_none() {
            let Ok(Some(blocco)) = tokio::time::timeout_at(limite, server.leggi()).await else { break };
            display = String::from_utf8_lossy(&blocco).lines().find_map(display_da_messaggio);
        }
        let display = display.ok_or_else(|| anyhow::anyhow!("display virtuale non trovato"))?;
        adb.esegui(&format!(
            "cmd window set-ignore-orientation-request -d {display} true; \
             am start --display {display} -a android.intent.action.VIEW -d {url} {app}"
        ))
        .await?;
        // «rapporto»: cosa dice Android delle finestre e delle attività sul display.
        if con("rapporto") {
            tokio::time::sleep(Duration::from_secs(6)).await;
            let r = adb.esegui("dumpsys window windows; echo '=== ATTIVITA'; dumpsys activity activities").await?;
            std::fs::write("rapporto-display.txt", format!("display {display}\n{r}"))?;
            println!("rapporto in rapporto-display.txt (display {display})");
        }
        // Si scartano i primi secondi (caricamento della pagina).
        let (mut fotogrammi, mut byte, mut pausa_max) = (0u32, 0usize, Duration::ZERO);
        let inizio_misura = tokio::time::Instant::now() + Duration::from_secs(if con("video") { 12 } else { 6 });
        let fine = inizio_misura + Duration::from_secs(u64::from(secondi));
        let mut ultimo = tokio::time::Instant::now();
        while let Ok(p) = tokio::time::timeout_at(fine, leggi_pacchetto(&mut flusso)).await {
            if let Pacchetto::Dati { config: false, dati, .. } = p? {
                let adesso = tokio::time::Instant::now();
                if adesso >= inizio_misura {
                    fotogrammi += 1;
                    byte += dati.len();
                    pausa_max = pausa_max.max(adesso - ultimo);
                }
                ultimo = adesso;
            }
        }
        println!(
            "{l}×{a}/{d}{}{}{}: {:.1} fotogrammi/s, {:.0} KB/s, pausa massima {} ms",
            if con("video") { " video" } else { "" },
            if con_audio { " +audio" } else { "" },
            if con_controllo { " +controllo" } else { "" },
            fotogrammi as f32 / secondi as f32,
            byte as f32 / 1024.0 / secondi as f32,
            pausa_max.as_millis()
        );
        togli_dalle_recenti(&adb, display).await?;
        let _ = telefono.pannello(true).await;
        if let Some(c) = controllo {
            c.abort();
        }
        if let Some(a) = audio {
            a.abort();
        }
        Ok(())
    })
}

/// Diagnosi: un comando di shell via cavo, con la chiave di Phonestra.
fn shell_usb(comando: String) -> Result<()> {
    let Some(t) = usb::telefoni().into_iter().find(|t| t.stato != Stato::DebugSpento) else {
        bail!("nessun telefono col Debug USB attivo collegato al cavo");
    };
    println!("{}", Collegamento::usb(t.vendor, t.prodotto)?.shell(&comando)?);
    Ok(())
}

fn collega() -> Result<()> {
    let telefoni = Telefoni::carica()?;
    let Some(salvato) = telefoni.elenco.first() else {
        bail!("nessun telefono configurato: prima «phonestra-prova prepara» col cavo");
    };
    println!("Cerco {} ({}) in rete…", salvato.nome, salvato.seriale);
    // Quando il debug si riavvia (cavo staccato, sblocco) la porta cambia e per
    // qualche istante in rete circola ancora quella vecchia: si ricerca e riprova.
    let mut tentativi = 0;
    let mut c = loop {
        tentativi += 1;
        let Some(trovato) = rete::cerca(Duration::from_secs(3))?.into_iter().find(|t| t.seriale == salvato.seriale) else {
            bail!("telefono non raggiungibile: acceso, sbloccato, stessa rete Wi-Fi, Debug wireless attivo?");
        };
        println!("Trovato su {}. Collegamento cifrato…", trovato.indirizzo);
        match Collegamento::wifi(trovato.indirizzo) {
            Ok(c) => break c,
            Err(e) if tentativi < 3 && format!("{e:#}").contains("Connection refused") => {
                println!("Porta non più valida, ripeto la ricerca…");
            }
            Err(e) => return Err(e),
        }
    };
    let modello = c.shell("getprop ro.product.model")?;
    let batteria = c.batteria()?;
    println!("Collegato via Wi-Fi a {modello} — batteria {batteria}%.");
    Ok(())
}
