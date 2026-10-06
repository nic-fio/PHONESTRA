//! `phonestra-prova`: la tappa 1 da riga di comando, senza `adb`.
//!
//!   phonestra-prova usb        stato del telefono collegato col cavo
//!   phonestra-prova cerca      telefoni col Debug wireless in rete
//!   phonestra-prova prepara    via cavo: Wi-Fi acceso, telefono salvato
//!   phonestra-prova collega    via Wi-Fi, senza indirizzi, al telefono salvato
//!   phonestra-prova video-prova <prova> [opzioni]   misure del video (aiutante)
//!   phonestra-prova servizio [secondi] [--sparisci]  scheletro del componente nostro
//!   phonestra-prova input-componente <prova> [opzioni]  modulo input del componente
//!   phonestra-prova audio-componente <secondi> [aac|pcm] [--ascolta] [--uccidi]  audio del componente
//!   phonestra-prova video-componente app|schermo [opzioni]  video col componente nostro
//!   phonestra-prova file <cartella> | ricevi <percorso> <destinazione>  file del telefono

#[path = "prova/audio_componente.rs"]
mod audio_componente;

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
        "banner" => banner(),
        "shell" => shell_wifi(std::env::args().skip(2).collect::<Vec<_>>().join(" ")),
        "app" => app(),
        "audio-nostro" => audio_nostro(std::env::args().skip(2).collect()),
        "codificatori" => codificatori(),
        "notifiche" => notifiche(),
        "sfondo" => sfondo(),
        "file" => file(std::env::args().skip(2).collect()),
        "custode" => custode(std::env::args().nth(2).unwrap_or_default()),
        "procedura" => procedura(),
        "video-prova" => video_prova(std::env::args().skip(2).collect()),
        "throughput" => throughput(std::env::args().skip(2).collect()),
        "servizio" => servizio(std::env::args().skip(2).collect()),
        "input-componente" => input_componente(std::env::args().skip(2).collect()),
        "audio-componente" => audio_componente::audio_componente(std::env::args().skip(2).collect()),
        "video-componente" => video_componente(std::env::args().skip(2).collect()),
        _ => {
            eprintln!("uso: phonestra-prova usb | cerca | abbina <codice> [ip:porta] | prepara | procedura | collega | shell-usb <comando>");
            eprintln!("     phonestra-prova video-prova schermo|chiave|istanze|protetto|task|permessi|codificatori [opzioni]");
            eprintln!("     phonestra-prova servizio [secondi] [--sparisci]");
            eprintln!("     phonestra-prova {}", phonestra::prova_input::USO);
            eprintln!("     phonestra-prova audio-componente <secondi> [aac|pcm] [--ascolta] [--uccidi]");
            eprintln!("     phonestra-prova file <cartella> | file ricevi <percorso> <destinazione> | file miniature <percorsi>");
            eprintln!("     phonestra-prova video-componente app|schermo [--app P] [--secondi S] [--codec h264|h265] [--senza-pannello]");
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

/// Associazione col codice a 6 cifre (SPECIFICATION §5.2): trova la schermata del
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

/// Strumento di misura dell'audio nostro (notes/study/audio.md, prove A2,
/// A4, A5, A6): `audio-nostro <secondi> [submix|loopback|render] [pcm|aac]
/// [senza-priorita] [voce]` (predefiniti submix e pcm). Salva
/// `phonestra-prova.wav` (PCM) o `phonestra-prova.aac` (ADTS, leggibile con
/// ffprobe/ffmpeg), stampa le misure del telefono man mano e alla fine un
/// riassunto; il PCM è analizzato anche sul PC (zeri esatti, tagli netti).
fn audio_nostro(argomenti: Vec<String>) -> Result<()> {
    use phonestra::adb::Adb;
    use phonestra::misura_audio::{self, FREQUENZA, Riga};
    use phonestra::video_nostro::{Pacchetto, leggi_pacchetto};
    let secondi: u64 = argomenti.first().and_then(|s| s.parse().ok()).unwrap_or(10);
    let (mut sorgente, mut formato, mut priorita, mut voce) = ("submix", "pcm", "si", "no");
    for a in argomenti.iter().skip(1) {
        match a.as_str() {
            "submix" | "loopback" | "render" => sorgente = a.as_str(),
            "pcm" | "aac" => formato = a.as_str(),
            "senza-priorita" => priorita = "no",
            "voce" => voce = "si",
            _ => bail!("argomento sconosciuto: {a} (audio-nostro <secondi> [submix|loopback|render] [pcm|aac] [senza-priorita] [voce])"),
        }
    }
    let pcm = formato == "pcm";
    let comando = format!("audio sorgente={sorgente} formato={formato} priorita={priorita} voce={voce}");
    let nome = if pcm { "phonestra-prova.wav" } else { "phonestra-prova.aac" };
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        println!("telefono: {comando}");
        let mut canale = phonestra::app::aiutante_continuo(&adb, &comando).await?;
        let mut file = Vec::new();
        let (mut pacchetti, mut irregolari, mut mancanti) = (0u32, 0u32, 0u64);
        // PCM: campioni ricevuti, da cui l'orario atteso del prossimo pacchetto.
        let mut ricevuti = 0u64;
        // Orario del primo pacchetto: dal 6 ott gli orari sono sull'orologio
        // monotono del telefono, non partono da 0.
        let mut origine = None::<u64>;
        let mut precedente = None::<u64>;
        let mut ultima = None::<Riga>;
        let (mut deriva_min, mut deriva_max) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut attesa_max, mut lettura_max) = (f64::NEG_INFINITY, 0f64);
        let fine = std::time::Instant::now() + Duration::from_secs(secondi);
        while std::time::Instant::now() < fine {
            let p = match tokio::time::timeout(Duration::from_secs(5), leggi_pacchetto(&mut canale)).await {
                Err(_) => bail!("nessun pacchetto audio da 5 s"),
                Ok(p) => p?,
            };
            match p {
                // Bit 61: una riga di testo del telefono (misura, avviso, errore).
                Pacchetto::Dati { chiave: true, dati, .. } => {
                    let riga = Riga::leggi(&String::from_utf8_lossy(&dati));
                    println!("[telefono] {}", riga.testo);
                    if riga.tipo == "errore" {
                        bail!("il telefono ha interrotto la cattura");
                    }
                    if riga.tipo == "misura" {
                        if let Some(d) = riga.numero("deriva_ms") {
                            deriva_min = deriva_min.min(d);
                            deriva_max = deriva_max.max(d);
                        }
                        attesa_max = attesa_max.max(riga.numero("attesa_ms").unwrap_or(f64::NEG_INFINITY));
                        lettura_max = lettura_max.max(riga.numero("lettura_max_ms").unwrap_or(0.0));
                        ultima = Some(riga);
                    }
                }
                Pacchetto::Dati { config: true, dati, .. } => println!("configurazione AAC: {dati:02x?}"),
                Pacchetto::Dati { pts, dati, .. } if pcm => {
                    pacchetti += 1;
                    let pts = pts - *origine.get_or_insert(pts);
                    // Orario = origine + campioni × 10⁶ / 48000 arrotondato per difetto: un
                    // salto vuol dire pacchetti persi per strada (coda piena).
                    let atteso = ricevuti * 1_000_000 / FREQUENZA as u64;
                    if pts != atteso {
                        irregolari += 1;
                        let campione = (pts * 6).div_ceil(125);
                        mancanti += campione.saturating_sub(ricevuti);
                        ricevuti = campione;
                    }
                    ricevuti += dati.len() as u64 / 4;
                    file.extend_from_slice(&dati);
                }
                Pacchetto::Dati { pts, dati, .. } => {
                    pacchetti += 1;
                    if let Some(p) = precedente
                        && pts.saturating_sub(p).abs_diff(21_333) > 1
                    {
                        irregolari += 1;
                    }
                    precedente = Some(pts);
                    // Intestazione ADTS: AAC-LC, 48 kHz, stereo.
                    let n = dati.len() + 7;
                    file.extend_from_slice(&[
                        0xff,
                        0xf1,
                        0x4c,
                        0x80 | ((n >> 11) & 0x03) as u8,
                        ((n >> 3) & 0xff) as u8,
                        (((n & 0x07) << 5) | 0x1f) as u8,
                        0xfc,
                    ]);
                    file.extend_from_slice(&dati);
                }
                Pacchetto::Dimensione { .. } => {}
            }
        }
        canale.chiudi().await.ok();
        if pcm {
            let mut wav = misura_audio::intestazione_wav(file.len() as u32).to_vec();
            wav.extend_from_slice(&file);
            std::fs::write(nome, &wav)?;
        } else {
            std::fs::write(nome, &file)?;
        }

        println!("\nriassunto: {sorgente}, {formato}, priorità {priorita}, {secondi} s");
        println!("  PC: {pacchetti} pacchetti, {irregolari} con orario irregolare, {} KB in {nome}", file.len() / 1024);
        if mancanti > 0 {
            println!(
                "  mancano {mancanti} campioni ({:.1} ms) persi per strada: l'analisi li salta, le giunture possono sembrare tagli",
                mancanti as f64 * 1000.0 / FREQUENZA as f64
            );
        }
        match &ultima {
            None => println!("  telefono: nessuna misura ricevuta"),
            Some(m) => {
                let v = |k: &str| m.valore(k).unwrap_or("?").to_string();
                println!(
                    "  telefono: {} s letti, {} letture ({} brevi), {} pacchetti persi in coda, nice {}",
                    v("t"),
                    v("letture"),
                    v("brevi"),
                    v("persi"),
                    v("nice")
                );
                println!(
                    "  telefono: {} sequenze di zeri esatti, {} ms in tutto, la più lunga {} ms",
                    v("zeri"),
                    v("zeri_ms"),
                    v("zeri_max_ms")
                );
                if deriva_min.is_finite() {
                    println!(
                        "  deriva AudioTimestamp: da {deriva_min:.2} a {deriva_max:.2} ms (ultima {}), attesa massima {attesa_max:.2} ms",
                        v("deriva_ms")
                    );
                } else {
                    println!("  deriva AudioTimestamp: non disponibile (getTimestamp senza risultato)");
                }
                println!("  lettura più lunga: {lettura_max:.2} ms");
            }
        }
        if pcm {
            let a = misura_audio::analizza(&misura_audio::campioni(&file));
            let totale = |t: &[misura_audio::Tratto]| t.iter().map(|t| t.millisecondi()).sum::<f64>();
            println!(
                "  analisi PC: {:.1} s, {} sequenze di zeri esatti ({:.1} ms), {} tagli netti ({:.1} ms)",
                a.campioni as f64 / FREQUENZA as f64,
                a.zeri.len(),
                totale(&a.zeri),
                a.tagli.len(),
                totale(&a.tagli)
            );
            for (nome, tratti) in [("zeri", &a.zeri), ("taglio", &a.tagli)] {
                for t in tratti.iter().take(20) {
                    println!("    {nome} a {:.3} s: {:.1} ms", t.secondi(), t.millisecondi());
                }
                if tratti.len() > 20 {
                    println!("    … altri {}", tratti.len() - 20);
                }
            }
        }
        Ok(())
    })
}

/// I codificatori audio e video del telefono (prova A1 dell'audio, 14 del video).
fn codificatori() -> Result<()> {
    use phonestra::adb::Adb;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let uscita = phonestra::app::codificatori(&adb).await?;
        for riga in uscita.lines() {
            match riga.split('\t').collect::<Vec<_>>()[..] {
                [nome, tipo, realizzazione, origine, alias, dettagli] => {
                    println!("{nome:<36} {tipo:<22} {realizzazione:<8} {origine:<9} {alias}\n    {dettagli}")
                }
                _ => println!("{riga}"),
            }
        }
        Ok(())
    })
}

/// Strumento di misura del video (notes/study/video.md, «Prove da fare sul
/// telefono»): avvia `video-prova <prova> [opzioni]` nell'aiutante e ne stampa
/// l'uscita man mano. I PNG salvati sul telefono (righe `png: …`) vengono
/// copiati in `phonestra-prova.<nome>.png` e cancellati dal telefono.
fn video_prova(argomenti: Vec<String>) -> Result<()> {
    use phonestra::adb::Adb;
    use base64::Engine;
    use phonestra::azioni::virgolette;
    if argomenti.is_empty() {
        bail!("uso: phonestra-prova video-prova schermo|chiave|istanze|protetto|task|permessi|codificatori [opzioni]");
    }
    let comando = std::iter::once("video-prova".to_string()).chain(argomenti.iter().map(|a| virgolette(a))).collect::<Vec<_>>().join(" ");
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let mut canale = phonestra::app::aiutante_testo(&adb, &comando).await?;
        let (mut resto, mut png) = (Vec::new(), Vec::new());
        while let Some(blocco) = canale.leggi().await {
            resto.extend_from_slice(&blocco);
            while let Some(fine) = resto.iter().position(|&b| b == b'\n') {
                let riga: Vec<u8> = resto.drain(..=fine).collect();
                let riga = String::from_utf8_lossy(&riga).trim_end().to_string();
                println!("{riga}");
                if let Some(p) = phonestra::app::png_della_prova(&riga) {
                    png.push(p.to_string());
                }
            }
        }
        if !resto.is_empty() {
            println!("{}", String::from_utf8_lossy(&resto));
        }
        for percorso in png {
            let testo = adb.esegui(&format!("base64 -w 0 {percorso}; rm -f {percorso}")).await?;
            let nome = percorso.rsplit('/').next().unwrap_or("immagine.png");
            match base64::engine::general_purpose::STANDARD.decode(testo.trim()) {
                Ok(dati) => {
                    std::fs::write(format!("phonestra-prova.{nome}"), &dati)?;
                    println!("→ phonestra-prova.{nome} ({} KB)", dati.len() / 1024);
                }
                Err(e) => println!("{percorso} non copiato: {e}"),
            }
        }
        Ok(())
    })
}

/// Scheletro del componente nostro (notes/component.md): avvia il servizio,
/// stampa il CIAO con l'autotest, affida al custode la prova innocua (un file
/// in /data/local/tmp), tiene il battito per `secondi` e chiude in ordine.
/// Con `--sparisci` smette di mandare il battito senza chiudere niente (PC
/// sparito): il servizio deve uscire da solo dopo ~5 s e il custode ripulire;
/// il controllo si fa con un secondo collegamento.
fn servizio(argomenti: Vec<String>) -> Result<()> {
    use phonestra::adb::Adb;
    use phonestra::componente::{self, Componente, Processo};
    let (mut secondi, mut sparisci) = (10u64, false);
    for a in &argomenti {
        match a.as_str() {
            "--sparisci" => sparisci = true,
            n => {
                secondi = n.parse().map_err(|_| anyhow::anyhow!("argomento sconosciuto: {n} (servizio [secondi] [--sparisci])"))?
            }
        }
    }
    let descrivi = |p: Processo| match p {
        Processo::Vivo => "ancora vivo".to_string(),
        Processo::Uscito(c) => format!("uscito con codice {c} ({})", componente::descrivi_uscita(c)),
        Processo::Chiuso => "canale chiuso senza codice d'uscita".to_string(),
    };
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let inizio = std::time::Instant::now();
        let mut c = Componente::avvia(&adb).await?;
        println!("servizio avviato, CIAO ricevuto in {} ms (pid {})", inizio.elapsed().as_millis(), c.pid.map_or("?".into(), |p| p.to_string()));
        for (k, v) in c.ciao.voci.iter().filter(|(k, _)| !k.starts_with("autotest.")) {
            println!("  {k} = {v}");
        }
        println!("autotest:");
        for (k, v) in c.ciao.autotest() {
            println!("  {k:<20} {v}");
        }
        let file = c.prova_custode().await?;
        println!("prova del custode: il servizio ha creato {file} e ha chiesto al custode di toglierlo");
        let r = componente::residui(&adb).await?;
        println!("processi del componente (PID, RSS in KB, comando):");
        for p in &r.processi {
            println!("  {}", p.chars().take(110).collect::<String>());
        }
        if !r.file.iter().any(|f| file.ends_with(f.as_str())) {
            println!("ATTENZIONE: il file di prova non c'è");
        }
        let esito;
        let finale;
        if sparisci {
            tokio::time::sleep(Duration::from_secs(2)).await;
            c.sospendi_battito();
            let da = std::time::Instant::now();
            println!("battito sospeso (il PC «sparisce» senza chiudere): il servizio deve uscire dopo ~{} s", componente::LIMITE_SILENZIO.as_secs());
            esito = c.attendi_uscita(Duration::from_secs(15)).await;
            println!("servizio: {} dopo {:.1} s", descrivi(esito), da.elapsed().as_secs_f32());
            // Controllo da un secondo collegamento: il primo è quello «sparito».
            let adb2 = Adb::wifi(indirizzo, &chiave).await?;
            finale = attendi_pulizia(&adb2).await?;
            drop(c);
        } else {
            let fine = std::time::Instant::now() + Duration::from_secs(secondi);
            while std::time::Instant::now() < fine && c.processo() == Processo::Vivo {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            let b = c.battito();
            println!(
                "battito: {} messaggi ricevuti dal servizio, {} battiti mandati, silenzio massimo {} ms",
                b.ricevuti,
                b.mandati,
                b.pausa_massima.as_millis()
            );
            esito = c.chiudi().await?;
            println!("chiusura: servizio {}", descrivi(esito));
            finale = attendi_pulizia(&adb).await?;
        }
        let atteso = if sparisci { Processo::Uscito(3) } else { Processo::Uscito(0) };
        let controlli = [
            ("servizio uscito col codice atteso", esito == atteso),
            ("nessun processo del componente", finale.processi.is_empty()),
            ("custode eseguito (file di prova tolto)", !finale.file.iter().any(|f| file.ends_with(f.as_str()))),
            ("nessun jar del servizio in /data/local/tmp", !finale.file.iter().any(|f| f.starts_with(componente::NOME_SERVIZIO))),
        ];
        for (nome, ok) in controlli {
            println!("  {} {nome}", if ok { "ok" } else { "NO" });
        }
        for p in &finale.processi {
            println!("    rimasto: {}", p.chars().take(110).collect::<String>());
        }
        for f in &finale.file {
            println!("    rimasto: /data/local/tmp/{f}");
        }
        if !finale.altri_file.is_empty() {
            println!("  altri file phonestra-* (non del componente): {}", finale.altri_file.join(" "));
        }
        if controlli.iter().any(|(_, ok)| !ok) {
            bail!("prova del servizio non riuscita");
        }
        println!("prova riuscita");
        Ok(())
    })
}

/// Prove del modulo input del componente (`prova_input`).
fn input_componente(argomenti: Vec<String>) -> Result<()> {
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = phonestra::adb::Adb::wifi(indirizzo, &chiave).await?;
        phonestra::prova_input::esegui(&adb, &argomenti).await
    })
}

/// Video col componente nostro (`phonestra::video_nostro::prova`).
fn video_componente(argomenti: Vec<String>) -> Result<()> {
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = phonestra::adb::Adb::wifi(indirizzo, &chiave).await?;
        phonestra::video_nostro::prova::esegui(&adb, &argomenti).await
    })
}

/// Aspetta (al massimo 5 s) che servizio e custode abbiano finito e ripulito.
async fn attendi_pulizia(adb: &phonestra::adb::Adb) -> Result<phonestra::componente::Residui> {
    let limite = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let r = phonestra::componente::residui(adb).await?;
        if r.pulito() || std::time::Instant::now() >= limite {
            return Ok(r);
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
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

/// Elenca una cartella del telefono (nomi, misure, date) o, con `ricevi`,
/// copia un file del telefono sul PC misurando la velocità.
fn file(argomenti: Vec<String>) -> Result<()> {
    use phonestra::adb::{Adb, sync};
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        match argomenti.iter().map(String::as_str).collect::<Vec<_>>()[..] {
            ["ricevi", percorso, destinazione] => {
                let inizio = std::time::Instant::now();
                let mut f = std::fs::File::create(destinazione)?;
                let n = sync::ricevi(&adb, percorso, &mut f, |_| true).await?;
                let s = inizio.elapsed().as_secs_f64();
                println!("{n} byte in {s:.2} s ({:.1} MB/s)", n as f64 / 1e6 / s.max(0.001));
            }
            ["miniature", ref percorsi @ ..] => {
                let percorsi: Vec<String> = percorsi.iter().map(|p| p.to_string()).collect();
                let inizio = std::time::Instant::now();
                let v = phonestra::app::miniature(&adb, 192, &percorsi).await?;
                for (p, m) in percorsi.iter().zip(&v) {
                    println!("{p}: {}", m.as_ref().map_or("nessuna miniatura".to_string(), |j| format!("{} byte", j.len())));
                }
                println!("{} miniature in {} ms", v.len(), inizio.elapsed().as_millis());
            }
            [cartella] => {
                let inizio = std::time::Instant::now();
                match sync::elenca(&adb, cartella).await? {
                    None => println!("{cartella}: non esiste o non si può leggere"),
                    Some(voci) => {
                        for v in &voci {
                            println!("{:>12} {:>11} {}{}", v.dimensione, v.modificato, v.nome, if v.cartella { "/" } else { "" });
                        }
                        println!("{} voci in {} ms", voci.len(), inizio.elapsed().as_millis());
                    }
                }
            }
            _ => bail!("uso: phonestra-prova file <cartella> | file ricevi <percorso> <destinazione>"),
        }
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
/// Il banner CNXN del telefono: funzioni ADB offerte (`features=`).
fn banner() -> Result<()> {
    use phonestra::adb::Adb;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        println!("{}", adb.dispositivo);
        Ok(())
    })
}

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

/// Misura del trasporto ADB (`notes/adb.md`): MB/s di un canale carico,
/// pause tra i blocchi e, con `--latenza`, piccoli messaggi su un secondo canale.
///   phonestra-prova throughput [MB] [--senza-delayed-ack] [--payload N] [--finestra N]
///                              [--latenza] [--exec] [--alla-lettura]
fn throughput(argomenti: Vec<String>) -> Result<()> {
    use phonestra::adb::misura;
    let opzioni = misura::Opzioni::da_argomenti(&argomenti)?;
    let indirizzo = indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(misura::esegui(indirizzo, &chiave, &opzioni))
}
