// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Protocollo `sync:` di ADB, per copiare file sul telefono (es. il componente
//! di Phonestra in `/data/local/tmp`), leggere le cartelle del telefono
//! (`LIS2`) e copiare file dal telefono al PC (`RECV`).

use anyhow::{Result, bail};

use super::Adb;
use crate::t;

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
            bail!(t!("copia di {} annullata", percorso));
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
            bail!(t!("copia di {} rifiutata: {}", percorso, String::from_utf8_lossy(&motivo)));
        }
        altro => bail!(t!("risposta sync inattesa: {}", format!("{:?}", String::from_utf8_lossy(altro)))),
    }
    c.scrivi(&richiesta(b"QUIT", 0)).await?;
    Ok(())
}

/// Una voce di una cartella del telefono.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voce {
    pub nome: String,
    pub cartella: bool,
    pub dimensione: u64,
    /// Ultima modifica, in secondi dal 1970.
    pub modificato: i64,
}

/// Byte di una voce `DNT2` dopo l'identificativo: errore, dispositivo, inode,
/// modo, collegamenti, uid, gid, dimensione, tre orari, lunghezza del nome
/// (`file_sync_protocol.h`, strutture senza riempitivi).
const DENT2: usize = 72;

/// Modo, dimensione, ultima modifica e lunghezza del nome da una voce `DNT2`.
fn leggi_dent2(b: &[u8]) -> (u32, u64, i64, usize) {
    let u32_a = |i: usize| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
    let u64_a = |i: usize| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
    (u32_a(20), u64_a(36), u64_a(52) as i64, u32_a(68) as usize)
}

/// Byte di una risposta `STA2` dopo l'identificativo: come `DNT2` senza la
/// lunghezza del nome.
const STAT2: usize = 68;

/// Le voci della cartella `percorso` del telefono, senza «.» e «..»;
/// `None` se la cartella non esiste o non si può leggere. Prima `STA2`:
/// nella memoria condivisa (FUSE) l'elenco non contiene «.» e «..», e una
/// cartella vuota darebbe la stessa risposta di una che non esiste. Poi
/// `LIS2` (Android 11+), con dimensioni a 64 bit: con `LIST` i video oltre
/// 4 GiB avrebbero misure sbagliate.
pub async fn elenca(adb: &Adb, percorso: &str) -> Result<Option<Vec<Voce>>> {
    let mut c = adb.apri("sync:").await?;
    if !cartella_su(&mut c, percorso).await? {
        c.scrivi(&richiesta(b"QUIT", 0)).await?;
        return Ok(None);
    }
    let mut v = richiesta(b"LIS2", percorso.len() as u32);
    v.extend_from_slice(percorso.as_bytes());
    c.scrivi(&v).await?;
    let mut voci = Vec::new();
    loop {
        let id = c.leggi_esatti(4).await?;
        let corpo = c.leggi_esatti(DENT2).await?;
        match &id[..] {
            b"DNT2" => {
                let (modo, dimensione, modificato, lunghezza) = leggi_dent2(&corpo);
                let nome = String::from_utf8_lossy(&c.leggi_esatti(lunghezza).await?).into_owned();
                if nome == "." || nome == ".." {
                    continue;
                }
                // 0o170000: tipo del file; 0o040000: cartella.
                voci.push(Voce { nome, cartella: modo & 0o170000 == 0o040000, dimensione, modificato });
            }
            b"DONE" => break,
            altro => bail!(t!("risposta sync inattesa: {}", format!("{:?}", String::from_utf8_lossy(altro)))),
        }
    }
    c.scrivi(&richiesta(b"QUIT", 0)).await?;
    Ok(Some(voci))
}

/// `percorso` esiste sul telefono ed è una cartella leggibile.
pub async fn e_cartella(adb: &Adb, percorso: &str) -> Result<bool> {
    let mut c = adb.apri("sync:").await?;
    let si = cartella_su(&mut c, percorso).await?;
    c.scrivi(&richiesta(b"QUIT", 0)).await?;
    Ok(si)
}

/// `STA2` sul canale `c` già aperto: `percorso` è una cartella.
async fn cartella_su(c: &mut super::Canale, percorso: &str) -> Result<bool> {
    let mut v = richiesta(b"STA2", percorso.len() as u32);
    v.extend_from_slice(percorso.as_bytes());
    c.scrivi(&v).await?;
    let id = c.leggi_esatti(4).await?;
    if &id[..] != b"STA2" {
        bail!(t!("risposta sync inattesa: {}", format!("{:?}", String::from_utf8_lossy(&id))));
    }
    let stat = c.leggi_esatti(STAT2).await?;
    let errore = u32::from_le_bytes(stat[0..4].try_into()?);
    let modo = u32::from_le_bytes(stat[20..24].try_into()?);
    Ok(errore == 0 && modo & 0o170000 == 0o040000)
}

/// Copia il file `percorso` del telefono in `destinazione`, a blocchi: dopo
/// ogni blocco `avanzamento` riceve i byte già arrivati; se risponde `false`
/// la copia si interrompe con un errore (il canale si chiude: adbd smette di
/// mandare). Restituisce i byte copiati.
pub async fn ricevi(
    adb: &Adb,
    percorso: &str,
    destinazione: &mut impl std::io::Write,
    mut avanzamento: impl FnMut(u64) -> bool,
) -> Result<u64> {
    let mut c = adb.apri("sync:").await?;
    let mut v = richiesta(b"RECV", percorso.len() as u32);
    v.extend_from_slice(percorso.as_bytes());
    c.scrivi(&v).await?;
    let mut ricevuti = 0u64;
    loop {
        let testa = c.leggi_esatti(8).await?;
        let n = u32::from_le_bytes(testa[4..8].try_into()?) as usize;
        match &testa[0..4] {
            b"DATA" => {
                let dati = c.leggi_esatti(n).await?;
                destinazione.write_all(&dati)?;
                ricevuti += n as u64;
                if !avanzamento(ricevuti) {
                    let _ = c.chiudi().await;
                    bail!(t!("copia di {} annullata", percorso));
                }
            }
            b"DONE" => break,
            b"FAIL" => {
                let motivo = c.leggi_esatti(n).await?;
                bail!(t!("copia di {} rifiutata: {}", percorso, String::from_utf8_lossy(&motivo)));
            }
            altro => bail!(t!("risposta sync inattesa: {}", format!("{:?}", String::from_utf8_lossy(altro)))),
        }
    }
    c.scrivi(&richiesta(b"QUIT", 0)).await?;
    Ok(ricevuti)
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn voce_dnt2() {
        let mut b = vec![0u8; DENT2];
        b[20..24].copy_from_slice(&0o100644u32.to_le_bytes());
        b[36..44].copy_from_slice(&5_000_000_000u64.to_le_bytes());
        b[52..60].copy_from_slice(&1_790_000_000i64.to_le_bytes());
        b[68..72].copy_from_slice(&7u32.to_le_bytes());
        assert_eq!(leggi_dent2(&b), (0o100644, 5_000_000_000, 1_790_000_000, 7));
    }
}
