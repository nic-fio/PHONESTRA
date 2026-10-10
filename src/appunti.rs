// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Appunti tra PC e telefono, solo testo (SPECIFICATION §9).
//!
//! - Telefono → PC: il componente nostro avvisa di ogni copia
//!   (`APPUNTI_CAMBIATI`, già senza testo se sensibile e senza doppioni). Le
//!   copie sensibili (password) non passano, nel dubbio nemmeno.
//! - PC → telefono: solo con Ctrl+V nella finestra di un'app (vedi
//!   [`crate::finestra`]); i testi dei gestori di password non passano.
//! - Quello che Phonestra stesso mette negli appunti del telefono (incolla,
//!   lettere accentate) non torna indietro al PC.

use anyhow::{Context, Result, bail};

use crate::collegamento::Collegamento;
use crate::componente::Condiviso;
use crate::input_nostro::{Appunti, tipo};

/// Oltre questa lunghezza (byte) il testo non passa: meglio il trasferimento file.
pub const MASSIMO: usize = 200_000;

/// Formato con cui KeePassXC, KDE e simili segnano negli appunti una password.
pub const SEGNO_PASSWORD: &str = "x-kde-passwordManagerHint";

/// Ascolta le copie fatte sul telefono finché il collegamento resta aperto,
/// dal componente nostro del collegamento (anche quando riparte).
pub async fn ascolta(collegamento: &Collegamento) -> Result<()> {
    let mut componente = collegamento.componente();
    loop {
        let attuale = componente.borrow_and_update().clone();
        if let Some(servizio) = attuale
            && let Err(e) = ascolta_nostro(&servizio, collegamento).await
        {
            crate::diagnosi(&format!("appunti: {e:#}"));
        }
        // Servizio caduto o non ancora partito: si aspetta il prossimo.
        if componente.changed().await.is_err() {
            return Ok(());
        }
    }
}

/// Le copie dal componente nostro, finché il servizio resta vivo.
async fn ascolta_nostro(servizio: &Condiviso, collegamento: &Collegamento) -> Result<()> {
    let mut avvisi = servizio.iscrivi(tipo::APPUNTI_CAMBIATI)?;
    servizio.domanda(tipo::APPUNTI_ASCOLTA, vec![1]).await.context("ascolto degli appunti sul telefono")?;
    crate::diagnosi("appunti: in ascolto delle copie sul telefono (componente)");
    while let Some(m) = avvisi.recv().await {
        let Some(avviso) = Appunti::da_avviso(&m) else { continue };
        match avviso? {
            Appunti::Testo(testo) => {
                crate::diagnosi(&format!("appunti: copia dal telefono, {} byte", testo.len()));
                if testo.len() <= MASSIMO && !collegamento.e_un_rimbalzo(&testo) {
                    collegamento.appunti_dal_telefono(testo);
                }
            }
            Appunti::Sensibili => eprintln!("[appunti] copia sensibile sul telefono: non passa al PC"),
            Appunti::Sconosciuti => eprintln!("[appunti] sensibilità sconosciuta: la copia non passa al PC"),
            Appunti::Vuoti => {}
        }
    }
    bail!("componente del telefono chiuso")
}
