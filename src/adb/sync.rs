//! Protocollo `sync:` di ADB, per copiare file sul telefono (es. il componente
//! di Phonestra in `/data/local/tmp`).

use anyhow::{Result, bail};

use super::Adb;

const BLOCCO: usize = 64 * 1024;

fn richiesta(id: &[u8; 4], lunghezza: u32) -> Vec<u8> {
    let mut v = id.to_vec();
    v.extend_from_slice(&lunghezza.to_le_bytes());
    v
}

/// Copia `contenuto` sul telefono in `percorso` con i permessi `modo`.
pub async fn invia(adb: &Adb, contenuto: &[u8], percorso: &str, modo: u32) -> Result<()> {
    invia_a_blocchi(adb, contenuto, percorso, modo, |_| true).await
}

/// Come [`invia`], a blocchi: dopo ogni blocco `avanzamento` riceve i byte
/// già mandati; se risponde `false` la copia si interrompe con un errore.
pub async fn invia_a_blocchi(
    adb: &Adb,
    contenuto: &[u8],
    percorso: &str,
    modo: u32,
    mut avanzamento: impl FnMut(usize) -> bool,
) -> Result<()> {
    let mut c = adb.apri("sync:").await?;
    let intestazione = format!("{percorso},{modo}");
    let mut v = richiesta(b"SEND", intestazione.len() as u32);
    v.extend_from_slice(intestazione.as_bytes());
    c.scrivi(&v).await?;
    let mut mandati = 0;
    for blocco in contenuto.chunks(BLOCCO) {
        let mut v = richiesta(b"DATA", blocco.len() as u32);
        v.extend_from_slice(blocco);
        c.scrivi(&v).await?;
        mandati += blocco.len();
        if !avanzamento(mandati) {
            bail!("copia di {percorso} annullata");
        }
    }
    let adesso = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as u32;
    c.scrivi(&richiesta(b"DONE", adesso)).await?;
    let risposta = c.leggi_esatti(8).await?;
    match &risposta[0..4] {
        b"OKAY" => {}
        b"FAIL" => {
            let n = u32::from_le_bytes(risposta[4..8].try_into()?) as usize;
            let motivo = c.leggi_esatti(n).await?;
            bail!("copia di {percorso} rifiutata: {}", String::from_utf8_lossy(&motivo));
        }
        altro => bail!("risposta sync inattesa: {:?}", String::from_utf8_lossy(altro)),
    }
    c.scrivi(&richiesta(b"QUIT", 0)).await?;
    Ok(())
}
