//! Le app del telefono per il drawer (SPECIFICATION §7.2), lette dall'aiutante di
//! Phonestra (`android/helper`): copiato in `/data/local/tmp`, eseguito con i
//! permessi della shell e cancellato subito dopo. Non è un'app installata.

use anyhow::{Context, Result, bail};
use base64::Engine;

use crate::adb::{Adb, sync};
use crate::t;

pub(crate) const AIUTO: &[u8] = include_bytes!("../android/phonestra-helper.jar");
const PERCORSO_AIUTO: &str = "/data/local/tmp/phonestra-aiuto.jar";

/// Un'app con un'icona nel launcher del telefono.
#[derive(Debug, Clone)]
pub struct App {
    pub pacchetto: String,
    /// Attività del launcher: un pacchetto può averne più d'una.
    pub attivita: String,
    pub nome: String,
    /// Icona PNG quadrata, già con la forma usata dal telefono.
    pub icona: Vec<u8>,
}

/// Elenco delle app del launcher in ordine alfabetico, con le icone di
/// `lato` pixel.
pub async fn elenco(adb: &Adb, lato: u32) -> Result<Vec<App>> {
    let uscita = aiutante(adb, &format!("app {lato}")).await?;
    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
        for riga in uscita.lines().filter(|r| r.split('\t').count() < 4) {
            eprintln!("[aiutante] {riga}");
        }
    }
    // L'aiutante chiude l'elenco con «fine»: se manca, è stato interrotto.
    if !uscita.lines().any(|r| r.starts_with("fine\t")) {
        bail!("{}", t!("elenco delle app interrotto ({} lette): lo rileggo al ricollegamento", leggi(&uscita).len()));
    }
    let mut app = leggi(&uscita);
    if app.is_empty() {
        bail!("{}", t!("l'aiutante non ha restituito app: {}", uscita.lines().take(15).collect::<Vec<_>>().join(" / ")));
    }
    app.sort_by_key(|a| a.nome.to_lowercase());
    Ok(app)
}

/// Copia l'aiutante (ogni volta con un nome suo: più richieste possono
/// arrivare insieme), lo esegue con `argomenti` e lo cancella.
async fn aiutante(adb: &Adb, argomenti: &str) -> Result<String> {
    let percorso = format!("{PERCORSO_AIUTO}.{:08x}", rand::random::<u32>());
    sync::invia(adb, AIUTO, &percorso, 0o644).await.context(t!("copia dell'aiutante sul telefono"))?;
    adb.esegui(&format!("CLASSPATH={percorso} app_process / phonestra.Aiuto {argomenti} 2>&1; rm -f {percorso}")).await
}

/// Avvia l'aiutante con `argomenti` e restituisce il canale della sua uscita,
/// per i comandi che mandano dati finché il PC li legge (l'audio, per esempio
/// `audio sorgente=render formato=pcm`). Chiudendo
/// il canale l'aiutante termina e si cancella.
pub async fn aiutante_continuo(adb: &Adb, argomenti: &str) -> Result<crate::adb::Canale> {
    avvia_continuo(adb, argomenti, "2>/dev/null").await
}

/// Come [`aiutante_continuo`], ma con gli errori mescolati all'uscita: per le
/// prove che stampano righe di testo man mano (`video-prova`).
pub async fn aiutante_testo(adb: &Adb, argomenti: &str) -> Result<crate::adb::Canale> {
    avvia_continuo(adb, argomenti, "2>&1").await
}

async fn avvia_continuo(adb: &Adb, argomenti: &str, errori: &str) -> Result<crate::adb::Canale> {
    let percorso = format!("{PERCORSO_AIUTO}.{:08x}", rand::random::<u32>());
    sync::invia(adb, AIUTO, &percorso, 0o644).await.context("copia dell'aiutante sul telefono")?;
    adb.apri(&format!("exec:CLASSPATH={percorso} app_process / phonestra.Aiuto {argomenti} {errori}; rm -f {percorso}"))
        .await
        .context("avvio dell'aiutante")
}

/// I codificatori audio e video del telefono, una riga ciascuno (prova A1
/// dell'audio, 14 del video): `nome \t tipo \t hardware|software \t
/// fornitore|android \t alias \t dettagli`.
pub async fn codificatori(adb: &Adb) -> Result<String> {
    aiutante(adb, "codificatori").await
}

/// Miniature JPEG (lato massimo `lato` pixel) dei file del telefono in
/// `percorsi`, nello stesso ordine; `None` per quelli che Android non sa
/// rimpicciolire (non foto, non video, file rovinati).
pub async fn miniature(adb: &Adb, lato: u32, percorsi: &[String]) -> Result<Vec<Option<Vec<u8>>>> {
    let b64 = base64::engine::general_purpose::STANDARD;
    let argomenti: Vec<String> = percorsi.iter().map(|p| b64.encode(p)).collect();
    let uscita = aiutante(adb, &format!("miniature {lato} {}", argomenti.join(" "))).await?;
    if std::env::var_os("PHONESTRA_DEBUG").is_some() {
        for riga in uscita.lines().filter(|r| !r.contains('\t')) {
            eprintln!("[miniature] {riga}");
        }
    }
    Ok(leggi_miniature(&uscita, percorsi.len()))
}

/// Righe `indice \t JPEG base64` (o `-`); le altre si scartano.
fn leggi_miniature(uscita: &str, quante: usize) -> Vec<Option<Vec<u8>>> {
    let mut v = vec![None; quante];
    for riga in uscita.lines() {
        let Some((i, dati)) = riga.split_once('\t') else { continue };
        if let (Ok(i), Ok(jpeg)) = (i.parse::<usize>(), base64::engine::general_purpose::STANDARD.decode(dati.trim()))
            && i < quante
            && jpeg.starts_with(&[0xff, 0xd8])
        {
            v[i] = Some(jpeg);
        }
    }
    v
}

/// Percorso di un PNG salvato da `video-prova` (riga `png: /data/local/tmp/….png`),
/// se è sicuro da usare in un comando di shell.
pub fn png_della_prova(riga: &str) -> Option<&str> {
    let percorso = riga.trim().strip_prefix("png: ")?;
    let nome = percorso.strip_prefix("/data/local/tmp/")?;
    let valido = nome.ends_with(".png") && nome.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c));
    valido.then_some(percorso)
}

/// Lo sfondo della schermata Home del telefono, largo `larghezza` pixel (PNG);
/// `None` se il telefono non lo concede.
pub async fn sfondo(adb: &Adb, larghezza: u32) -> Result<Option<Vec<u8>>> {
    let uscita = aiutante(adb, &format!("sfondo {larghezza}")).await?;
    let riga = uscita.lines().last().unwrap_or("").trim();
    match base64::engine::general_purpose::STANDARD.decode(riga) {
        Ok(png) if png.starts_with(b"\x89PNG") => Ok(Some(png)),
        _ => {
            eprintln!("[sfondo] non disponibile: {}", uscita.lines().take(4).collect::<Vec<_>>().join(" / "));
            Ok(None)
        }
    }
}

/// Righe `pacchetto \t attività \t nome \t icona base64`; le altre (errori
/// dell'aiutante) si scartano.
fn leggi(uscita: &str) -> Vec<App> {
    uscita
        .lines()
        .filter_map(|riga| {
            let mut campi = riga.split('\t');
            let (pacchetto, attivita, nome, icona) = (campi.next()?, campi.next()?, campi.next()?, campi.next()?);
            let icona = base64::engine::general_purpose::STANDARD.decode(icona.trim()).ok()?;
            Some(App { pacchetto: pacchetto.into(), attivita: attivita.into(), nome: nome.into(), icona })
        })
        .collect()
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn righe_valide_e_scarti() {
        let uscita = "com.a\tcom.a.Main\tUno\tiVBORw0K\nerrore qualsiasi\ncom.b\tcom.b.Main\tDue\tnon-base64!\nfine\t2\n";
        let app = leggi(uscita);
        assert_eq!(app.len(), 1);
        assert_eq!((app[0].pacchetto.as_str(), app[0].nome.as_str()), ("com.a", "Uno"));
        assert_eq!(&app[0].icona[1..4], b"PNG");
    }

    #[test]
    fn righe_delle_miniature() {
        let v = leggi_miniature("0\t/9j/\n1\t-\n2: errore\n7\t/9j/\n", 3);
        assert_eq!(v, vec![Some(vec![0xff, 0xd8, 0xff]), None, None]);
    }

    #[test]
    fn png_delle_prove() {
        assert_eq!(png_della_prova("png: /data/local/tmp/phonestra-schermo-12.png"), Some("/data/local/tmp/phonestra-schermo-12.png"));
        assert_eq!(png_della_prova("png: /data/local/tmp/a b.png"), None);
        assert_eq!(png_della_prova("png: /data/local/tmp/x.png; rm -rf /"), None);
        assert_eq!(png_della_prova("png: /sdcard/x.png"), None);
        assert_eq!(png_della_prova("immagine: 3 % di pixel neri"), None);
    }
}
