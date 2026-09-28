//! Le app del telefono per il drawer (SPECIFICHE §7.2), lette dall'aiutante di
//! Phonestra (`telefono/aiuto`): copiato in `/data/local/tmp`, eseguito con i
//! permessi della shell e cancellato subito dopo. Non è un'app installata.

use anyhow::{Context, Result, bail};
use base64::Engine;

use crate::adb::{Adb, sync};

const AIUTO: &[u8] = include_bytes!("../telefono/phonestra-aiuto.jar");
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
    let mut app = leggi(&uscita);
    if app.is_empty() {
        bail!("l'aiutante non ha restituito app: {}", uscita.lines().take(15).collect::<Vec<_>>().join(" / "));
    }
    app.sort_by_key(|a| a.nome.to_lowercase());
    Ok(app)
}

/// Copia l'aiutante (ogni volta con un nome suo: più richieste possono
/// arrivare insieme), lo esegue con `argomenti` e lo cancella.
async fn aiutante(adb: &Adb, argomenti: &str) -> Result<String> {
    let percorso = format!("{PERCORSO_AIUTO}.{:08x}", rand::random::<u32>());
    sync::invia(adb, AIUTO, &percorso, 0o644).await.context("copia dell'aiutante sul telefono")?;
    adb.esegui(&format!("CLASSPATH={percorso} app_process / phonestra.Aiuto {argomenti} 2>&1; rm -f {percorso}")).await
}

/// Avvia l'aiutante con `argomenti` e restituisce il canale della sua uscita,
/// per i comandi che mandano dati finché il PC li legge (l'audio). Chiudendo
/// il canale l'aiutante termina e si cancella.
pub async fn aiutante_continuo(adb: &Adb, argomenti: &str) -> Result<crate::adb::Canale> {
    let percorso = format!("{PERCORSO_AIUTO}.{:08x}", rand::random::<u32>());
    sync::invia(adb, AIUTO, &percorso, 0o644).await.context("copia dell'aiutante sul telefono")?;
    adb.apri(&format!("exec:CLASSPATH={percorso} app_process / phonestra.Aiuto {argomenti} 2>/dev/null; rm -f {percorso}"))
        .await
        .context("avvio dell'aiutante")
}

/// Se gli appunti del telefono sono segnati come sensibili (password): non
/// devono arrivare al PC (SPECIFICHE §9). `None` se l'aiutante non risponde.
pub async fn appunti_sensibili(adb: &Adb) -> Result<Option<bool>> {
    let uscita = aiutante(adb, "appunti-sensibili").await?;
    Ok(match uscita.lines().last().map(str::trim) {
        Some("sensibile") => Some(true),
        Some("normale" | "vuoto") => Some(false),
        _ => {
            eprintln!("[appunti] risposta inattesa dell'aiutante: {}", uscita.lines().take(3).collect::<Vec<_>>().join(" / "));
            None
        }
    })
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
        let uscita = "com.a\tcom.a.Main\tUno\tiVBORw0K\nerrore qualsiasi\ncom.b\tcom.b.Main\tDue\tnon-base64!\n";
        let app = leggi(uscita);
        assert_eq!(app.len(), 1);
        assert_eq!((app[0].pacchetto.as_str(), app[0].nome.as_str()), ("com.a", "Uno"));
        assert_eq!(&app[0].icona[1..4], b"PNG");
    }
}
