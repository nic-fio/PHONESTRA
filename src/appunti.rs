//! Appunti tra PC e telefono, solo testo (SPECIFICHE §9).
//!
//! - Telefono → PC: una sessione solo-comandi del componente
//!   (`clipboard_autosync`) segnala ogni copia; quelle segnate come sensibili
//!   (password) non passano.
//! - PC → telefono: solo con Ctrl+V nella finestra di un'app (vedi
//!   [`crate::finestra`]); i testi dei gestori di password non passano.
//! - Quello che Phonestra stesso mette negli appunti del telefono (incolla,
//!   lettere accentate) non torna indietro al PC.

use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::adb::{Adb, Canale};
use crate::app;
use crate::collegamento::Collegamento;
use crate::sessione::{apri_canale, lancia};

/// Oltre questa lunghezza (byte) il testo non passa: meglio il trasferimento file.
pub const MASSIMO: usize = 200_000;

/// Formato con cui KeePassXC, KDE e simili segnano negli appunti una password.
pub const SEGNO_PASSWORD: &str = "x-kde-passwordManagerHint";

/// Ascolta le copie fatte sul telefono finché il collegamento resta aperto.
pub async fn ascolta(adb: &Adb, collegamento: &Collegamento) -> Result<()> {
    let (scid, mut server) = lancia(adb, "video=false audio=false control=true clipboard_autosync=true").await?;
    // Messaggi del componente (errori di lettura degli appunti…).
    let chiudi_server = server.chiusore();
    tokio::spawn(async move {
        while let Some(blocco) = server.leggi().await {
            for riga in String::from_utf8_lossy(&blocco).lines() {
                eprintln!("[telefono/appunti] {riga}");
            }
        }
    });
    let esito = async {
        let mut canale = apri_canale(adb, scid).await?;
        // Byte fittizio e nome del dispositivo (64 byte); niente codec.
        tokio::time::timeout(Duration::from_secs(10), canale.leggi_esatti(1 + 64))
            .await
            .context("il componente degli appunti non risponde")??;
        crate::sessione::diagnosi("appunti: in ascolto delle copie sul telefono");
        loop {
            if let Some(testo) = messaggio(&mut canale).await? {
                crate::sessione::diagnosi(&format!("appunti: copia dal telefono, {} byte", testo.len()));
                if testo.len() > MASSIMO || collegamento.e_un_rimbalzo(&testo) {
                    continue;
                }
                match app::appunti_sensibili(adb).await {
                    Ok(Some(false)) => collegamento.appunti_dal_telefono(testo),
                    Ok(Some(true)) => eprintln!("[appunti] copia sensibile sul telefono: non passa al PC"),
                    // Nel dubbio non passa: potrebbe essere una password.
                    Ok(None) | Err(_) => eprintln!("[appunti] sensibilità sconosciuta: la copia non passa al PC"),
                }
            }
        }
    }
    .await;
    let _ = chiudi_server.chiudi().await;
    esito
}

/// Il prossimo messaggio del telefono: il testo se sono appunti, `None` per
/// gli altri (conferme, tastiera emulata).
async fn messaggio(canale: &mut Canale) -> Result<Option<String>> {
    let tipo = canale.leggi_esatti(1).await?[0];
    let u32_be = |b: &[u8]| u32::from_be_bytes(b[0..4].try_into().unwrap());
    match tipo {
        0 => {
            let lunghezza = u32_be(&canale.leggi_esatti(4).await?) as usize;
            let testo = canale.leggi_esatti(lunghezza).await?;
            Ok(Some(String::from_utf8_lossy(&testo).into_owned()))
        }
        1 => {
            canale.leggi_esatti(8).await?;
            Ok(None)
        }
        2 => {
            let intestazione = canale.leggi_esatti(4).await?;
            let lunghezza = u16::from_be_bytes([intestazione[2], intestazione[3]]) as usize;
            canale.leggi_esatti(lunghezza).await?;
            Ok(None)
        }
        altro => bail!("messaggio sconosciuto dal componente: {altro}"),
    }
}
