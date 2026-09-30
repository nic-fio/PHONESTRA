//! `phonestra-prova audio-componente <secondi> [aac|pcm] [--ascolta] [--uccidi]`:
//! il canale audio del componente nostro (notes/component.md, «Audio»).
//!
//! Avvia il servizio, apre il canale `audio`, riceve per `secondi` e salva
//! `phonestra-prova.aac` (ADTS) o `phonestra-prova.wav`; con `--ascolta` lo
//! riproduce anche dal vivo dalle casse del PC con la pipeline vera
//! ([`Riproduzione`]). Alla fine: riassunto (pacchetti, orari, zeri e tagli con
//! `misura_audio`) e controllo che sul telefono non resti niente (politica
//! audio, processi, jar). Con `--uccidi`, a metà tempo il servizio viene
//! ucciso con `kill -9`: la politica deve sparire lo stesso (la toglie Android
//! quando il binder del processo muore).

use std::time::{Duration, Instant};

use anyhow::{Result, bail};
use phonestra::adb::Adb;
use phonestra::audio_nostro::{self, ControlloOrari, Flusso, Formato, Pacchetto, Riproduzione};
use phonestra::componente::{self, Componente, Processo};
use phonestra::configurazione;
use phonestra::misura_audio::{self, FREQUENZA, Riga};

/// Politiche audio sul telefono: (in tutto, in loop-back), da `dumpsys audio`.
async fn politiche(adb: &Adb) -> Result<(usize, usize)> {
    Ok(audio_nostro::conta_politiche(&adb.esegui("dumpsys audio").await?))
}

pub fn audio_componente(argomenti: Vec<String>) -> Result<()> {
    let uso = "audio-componente <secondi> [aac|pcm] [--ascolta] [--uccidi]";
    let mut secondi = 10u64;
    let (mut formato, mut ascolta, mut uccidi) = (Formato::Aac, false, false);
    for a in &argomenti {
        match a.as_str() {
            "--ascolta" => ascolta = true,
            "--uccidi" => uccidi = true,
            f if Formato::da_nome(f).is_some() => formato = Formato::da_nome(f).unwrap(),
            n => secondi = n.parse().map_err(|_| anyhow::anyhow!("argomento sconosciuto: {n} ({uso})"))?,
        }
    }
    gst::init()?;
    let nome = match formato {
        Formato::Aac => "phonestra-prova.aac",
        Formato::Pcm => "phonestra-prova.wav",
    };
    let indirizzo = super::indirizzo_telefono()?;
    let chiave = configurazione::chiave()?;
    tokio::runtime::Runtime::new()?.block_on(async move {
        let adb = Adb::wifi(indirizzo, &chiave).await?;
        let politiche_prima = politiche(&adb).await?;
        let inizio = Instant::now();
        let mut c = Componente::avvia(&adb).await?;
        let pid = c.pid;
        println!("servizio avviato in {} ms (pid {})", inizio.elapsed().as_millis(), pid.map_or("?".into(), |p| p.to_string()));
        println!("  autotest.audio_policy = {}", c.ciao.valore("autotest.audio_policy").unwrap_or("?"));
        let aperto = Instant::now();
        let mut flusso = Flusso::apri(&c.apritore(), formato).await?;
        println!("canale audio aperto in {} ms", aperto.elapsed().as_millis());
        println!("[telefono] {}", flusso.inizio.testo);

        let mut riproduzione = match (ascolta, formato) {
            (true, Formato::Pcm) => Some(Riproduzione::nuova(formato, &[])?),
            _ => None,
        };
        let mut configurazione = Vec::new();
        let mut pacchetti: Vec<Vec<u8>> = Vec::new();
        let mut orari = ControlloOrari::default();
        let (mut ultima, mut deriva) = (None::<Riga>, (f64::INFINITY, f64::NEG_INFINITY));
        let (mut attesa_max, mut lettura_max) = (f64::NEG_INFINITY, 0f64);
        let (mut arrivo, mut pausa_max) = (None::<Instant>, Duration::ZERO);
        let mut primo_dato = None::<Duration>;
        let mut controllo_durante = None;
        let mut ucciso = false;
        let fine = Instant::now() + Duration::from_secs(secondi);
        let meta = Instant::now() + Duration::from_secs(secondi / 2);
        let mut chiuso_dal_telefono = false;
        while Instant::now() < fine {
            // Politiche a metà, in parallelo: la lettura non si ferma.
            if controllo_durante.is_none() && Instant::now() >= meta {
                let adb = adb.clone();
                controllo_durante = Some(tokio::spawn(async move { politiche(&adb).await }));
            }
            if uccidi && !ucciso && Instant::now() >= meta + Duration::from_secs(1) {
                let Some(pid) = pid else { bail!("pid del servizio sconosciuto") };
                println!("kill -9 {pid}");
                adb.esegui(&format!("kill -9 {pid}")).await?;
                ucciso = true;
            }
            let p = match tokio::time::timeout(Duration::from_secs(5), flusso.prossimo()).await {
                Err(_) if ucciso => break,
                Err(_) => bail!("nessun pacchetto audio da 5 s"),
                Ok(p) => p?,
            };
            let adesso = Instant::now();
            if let Some(a) = arrivo {
                pausa_max = pausa_max.max(adesso - a);
            }
            arrivo = Some(adesso);
            match p {
                None if ucciso => {
                    chiuso_dal_telefono = true;
                    break;
                }
                None => bail!("il telefono ha chiuso il canale audio"),
                Some(Pacchetto::Testo(t)) => {
                    let riga = Riga::leggi(&t);
                    println!("[telefono] {}", riga.testo);
                    if riga.tipo == "errore" {
                        bail!("il telefono ha interrotto l'audio");
                    }
                    if riga.tipo == "misura" {
                        if let Some(d) = riga.numero("deriva_ms") {
                            deriva = (deriva.0.min(d), deriva.1.max(d));
                        }
                        attesa_max = attesa_max.max(riga.numero("attesa_ms").unwrap_or(f64::NEG_INFINITY));
                        lettura_max = lettura_max.max(riga.numero("lettura_max_ms").unwrap_or(0.0));
                        ultima = Some(riga);
                    }
                }
                Some(Pacchetto::Configurazione(dati)) => {
                    println!("configurazione del codec: {dati:02x?}");
                    if ascolta && formato == Formato::Aac {
                        riproduzione = Some(Riproduzione::nuova(formato, &dati)?);
                    }
                    configurazione = dati;
                }
                Some(Pacchetto::Dati { pts, dati }) => {
                    primo_dato.get_or_insert(aperto.elapsed());
                    orari.controlla(pts, formato.campioni(dati.len()));
                    if let Some(r) = riproduzione.as_mut() {
                        let pts = r.regola(pts, &dati);
                        r.spingi(pts, dati.clone())?;
                    }
                    pacchetti.push(dati);
                }
            }
        }
        if let Some(r) = riproduzione.take() {
            let m = &r.margine;
            println!(
                "ascolto: {} pacchetti in ritardo, {} riallineamenti, margine finale {} ms",
                m.ritardi,
                m.riallineamenti,
                m.margine / 1_000_000
            );
        }
        let durante = match controllo_durante {
            Some(compito) => Some(compito.await??),
            None => None,
        };

        // Chiusura: prima il solo canale (la politica deve sparire col servizio
        // ancora vivo), poi il servizio.
        let mut dopo_canale = None;
        let mut vivo_dopo_canale = false;
        let esito = if ucciso {
            if !chiuso_dal_telefono {
                println!("ATTENZIONE: il canale audio non si è chiuso dopo kill -9");
            }
            let _ = flusso.chiudi().await;
            let e = c.attendi_uscita(Duration::from_secs(5)).await;
            c.chiudi().await?;
            e
        } else {
            flusso.chiudi().await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            dopo_canale = Some(politiche(&adb).await?);
            vivo_dopo_canale = c.processo() == Processo::Vivo;
            c.chiudi().await?
        };
        let finale = super::attendi_pulizia(&adb).await?;
        let politiche_fine = politiche(&adb).await?;

        // File.
        let mut file = Vec::new();
        match formato {
            Formato::Aac => {
                for p in &pacchetti {
                    file.extend_from_slice(&audio_nostro::intestazione_adts(p.len()));
                    file.extend_from_slice(p);
                }
            }
            Formato::Pcm => {
                let byte: usize = pacchetti.iter().map(Vec::len).sum();
                file.extend_from_slice(&misura_audio::intestazione_wav(byte as u32));
                for p in &pacchetti {
                    file.extend_from_slice(p);
                }
            }
        }
        std::fs::write(nome, &file)?;

        println!("\nriassunto: componente nostro, loopback, {}, {secondi} s{}", formato.nome(), if uccidi { ", kill -9 a metà" } else { "" });
        println!(
            "  PC: {} pacchetti, {} KB in {nome}, primo audio {} ms dopo l'apertura del canale",
            pacchetti.len(),
            file.len() / 1024,
            primo_dato.map_or("?".into(), |d| d.as_millis().to_string())
        );
        println!(
            "  orari: {} irregolari, {} campioni mancanti ({:.1} ms); pausa massima tra due pacchetti sul PC {} ms",
            orari.irregolari,
            orari.mancanti,
            orari.mancanti as f64 * 1000.0 / FREQUENZA as f64,
            pausa_max.as_millis()
        );
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
                println!("  telefono: {} sequenze di zeri esatti, {} ms, la più lunga {} ms", v("zeri"), v("zeri_ms"), v("zeri_max_ms"));
                if deriva.0.is_finite() {
                    println!(
                        "  deriva AudioTimestamp da {:.2} a {:.2} ms, attesa massima {attesa_max:.2} ms, lettura più lunga {lettura_max:.2} ms",
                        deriva.0, deriva.1
                    );
                }
            }
        }
        let analisi = {
            let configurazione = configurazione.clone();
            tokio::task::spawn_blocking(move || audio_nostro::decodifica(formato, &configurazione, &pacchetti)).await?
        };
        let (zeri, tagli) = match analisi {
            Ok(pcm) => {
                let a = misura_audio::analizza(&misura_audio::campioni(&pcm));
                let totale = |t: &[misura_audio::Tratto]| t.iter().map(|t| t.millisecondi()).sum::<f64>();
                println!(
                    "  analisi PC{}: {:.1} s, {} sequenze di zeri esatti ({:.1} ms), {} tagli netti ({:.1} ms)",
                    if formato == Formato::Aac { " (AAC decodificato)" } else { "" },
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
                (a.zeri.len(), a.tagli.len())
            }
            Err(e) => {
                println!("  analisi PC non riuscita: {e:#}");
                (usize::MAX, usize::MAX)
            }
        };
        let descrivi = |p: Option<(usize, usize)>| p.map_or("-".to_string(), |(t, l)| format!("{t} ({l} loop-back)"));
        println!(
            "  politiche audio: prima {}, durante {}, dopo la chiusura del canale {}, alla fine {}",
            descrivi(Some(politiche_prima)),
            descrivi(durante),
            descrivi(dopo_canale),
            descrivi(Some(politiche_fine))
        );
        let controllabile = durante.is_some_and(|d| d.0 > politiche_prima.0);
        if !controllabile {
            println!(
                "  ATTENZIONE: la politica non si vede in «dumpsys audio» (formato diverso?): controllo della politica non possibile, \
                 guardare la sezione «Audio policies» a mano"
            );
        }

        let mut controlli = vec![
            ("orari regolari, nessun pacchetto perso", orari.irregolari == 0),
            ("nessuna sequenza di zeri né taglio netto", zeri == 0 && tagli == 0),
        ];
        if !ucciso {
            controlli.push(("politica tolta alla chiusura del canale, servizio ancora vivo", controllabile && vivo_dopo_canale && dopo_canale == Some(politiche_prima)));
        }
        let atteso = if ucciso { Processo::Uscito(137) } else { Processo::Uscito(0) };
        controlli.extend([
            ("servizio uscito col codice atteso", esito == atteso),
            ("nessuna politica audio rimasta", controllabile && politiche_fine == politiche_prima),
            ("nessun processo del componente", finale.processi.is_empty()),
            ("nessun jar del servizio in /data/local/tmp", !finale.file.iter().any(|f| f.starts_with(componente::NOME_SERVIZIO))),
        ]);
        println!("  servizio: {esito:?}");
        for (nome, ok) in &controlli {
            println!("  {} {nome}", if *ok { "ok" } else { "NO" });
        }
        for p in &finale.processi {
            println!("    rimasto: {}", p.chars().take(110).collect::<String>());
        }
        if controlli.iter().any(|(_, ok)| !ok) {
            bail!("prova dell'audio del componente non riuscita");
        }
        println!("prova riuscita");
        Ok(())
    })
}
