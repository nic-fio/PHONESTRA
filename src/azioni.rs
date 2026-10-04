//! Azioni sul telefono chieste dal drawer (SPECIFICATION §11 e seguenti):
//! disinstallare, installare, inviare file. Solo comandi della
//! shell di Android: niente app di supporto sul telefono.

use std::collections::HashSet;

use anyhow::{Context, Result, bail};

use crate::adb::{Adb, sync};
use crate::t;

/// Una stringa tra virgolette singole per la shell del telefono.
pub fn virgolette(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Pacchetti installati dall'utente: solo questi si possono disinstallare.
pub async fn app_utente(adb: &Adb) -> Result<HashSet<String>> {
    let uscita = adb.esegui("pm list packages -3").await?;
    Ok(uscita.lines().filter_map(|r| r.trim().strip_prefix("package:")).map(str::to_string).collect())
}

pub async fn disinstalla(adb: &Adb, pacchetto: &str) -> Result<()> {
    let uscita = adb.esegui(&format!("pm uninstall {}", virgolette(pacchetto))).await?;
    if !uscita.contains("Success") {
        bail!("{}", uscita.trim());
    }
    Ok(())
}

/// Nome libero in `esistenti`: `foto.jpg`, poi `foto (1).jpg`, `foto (2).jpg`…
pub fn nome_libero(nome: &str, esistenti: &HashSet<String>) -> String {
    if !esistenti.contains(nome) {
        return nome.to_string();
    }
    let (base, estensione) = match nome.rfind('.') {
        Some(i) if i > 0 => (&nome[..i], &nome[i..]),
        _ => (nome, ""),
    };
    (1..).map(|n| format!("{base} ({n}){estensione}")).find(|c| !esistenti.contains(c)).unwrap()
}

/// Cartelle del telefono (dentro `/sdcard`) dove si possono mandare i file,
/// col nome italiano. Il nome da mostrare, nella lingua in uso, è
/// [`nome_cartella`].
pub const CARTELLE: [(&str, &str); 6] =
    [("Download", "Download"), ("Documents", "Documenti"), ("Pictures", "Immagini"), ("DCIM", "Fotocamera"), ("Music", "Musica"), ("Movies", "Video")];

/// Il nome da mostrare di una cartella di [`CARTELLE`], nella lingua in uso;
/// le altre restano col loro nome.
pub fn nome_cartella(cartella: &str) -> &str {
    match cartella {
        "Download" => t!("Download"),
        "Documents" => t!("Documenti"),
        "Pictures" => t!("Immagini"),
        "DCIM" => t!("Fotocamera"),
        "Music" => t!("Musica"),
        "Movies" => t!("Video"),
        altra => altra,
    }
}

/// Copia un file nella `cartella` del telefono (dentro `/sdcard`) senza
/// sovrascrivere e lo segnala ad Android (compare in Galleria e File).
/// Restituisce il nome usato.
pub async fn invia_file(adb: &Adb, cartella: &str, nome: &str, dati: &[u8], avanzamento: impl FnMut(usize) -> bool) -> Result<String> {
    let cartella = format!("/sdcard/{cartella}");
    let elenco = adb.esegui(&format!("mkdir -p {0}; ls -1 {0}", virgolette(&cartella))).await.unwrap_or_default();
    let esistenti: HashSet<String> = elenco.lines().map(str::to_string).collect();
    let nome = nome_libero(nome, &esistenti);
    let percorso = format!("{cartella}/{nome}");
    sync::invia_a_blocchi(adb, dati, &percorso, 0o644, avanzamento).await?;
    let _ = adb
        .esegui(&format!(
            "am broadcast -a android.intent.action.MEDIA_SCANNER_SCAN_FILE -d {} >/dev/null",
            virgolette(&format!("file://{percorso}"))
        ))
        .await;
    Ok(nome)
}

/// Installa (o aggiorna) un `.apk`. Android via ADB non chiede conferma:
/// la conferma la chiede Phonestra prima (SPECIFICATION §11).
pub async fn installa(adb: &Adb, dati: &[u8], avanzamento: impl FnMut(usize) -> bool) -> Result<()> {
    let percorso = "/data/local/tmp/phonestra-installa.apk";
    sync::invia_a_blocchi(adb, dati, percorso, 0o644, avanzamento).await.context(t!("copia sul telefono"))?;
    let uscita = adb.esegui(&format!("pm install -r {percorso}; rm -f {percorso}")).await?;
    if uscita.contains("Success") {
        return Ok(());
    }
    bail!("{}", spiega_installazione(&uscita));
}

/// Il motivo di un'installazione non riuscita, in parole semplici.
pub fn spiega_installazione(uscita: &str) -> String {
    let casi = [
        ("INSTALL_FAILED_UPDATE_INCOMPATIBLE", t!("sul telefono c'è già quest'app firmata da qualcun altro: disinstallala prima")),
        ("INSTALL_FAILED_VERSION_DOWNGRADE", t!("sul telefono c'è già una versione più recente")),
        ("INSTALL_FAILED_DEPRECATED_SDK_VERSION", t!("l'app è per una versione di Android troppo vecchia e Android la rifiuta")),
        ("INSTALL_FAILED_OLDER_SDK", t!("l'app richiede una versione di Android più recente di quella del telefono")),
        ("INSTALL_FAILED_INSUFFICIENT_STORAGE", t!("sul telefono non c'è abbastanza spazio")),
        ("INSTALL_FAILED_USER_RESTRICTED", t!("il telefono non permette di installare dal PC (sugli Xiaomi: attiva «Installa tramite USB» nelle Opzioni sviluppatore)")),
        ("INSTALL_FAILED_VERIFICATION_FAILURE", t!("Play Protect ha bloccato l'app")),
        ("INSTALL_PARSE_FAILED", t!("il file non è un'app Android valida")),
        ("INSTALL_FAILED_INVALID_APK", t!("il file non è un'app Android valida")),
        ("INSTALL_FAILED_NO_MATCHING_ABIS", t!("l'app non è fatta per il processore di questo telefono")),
    ];
    for (codice, testo) in casi {
        if uscita.contains(codice) {
            return testo.to_string();
        }
    }
    let riga = uscita.lines().find(|r| r.contains("Failure")).unwrap_or(uscita).trim();
    t!("installazione rifiutata dal telefono ({})", riga)
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn virgolette_singole() {
        assert_eq!(virgolette("foto di Anna.jpg"), "'foto di Anna.jpg'");
        assert_eq!(virgolette("l'estate.jpg"), r"'l'\''estate.jpg'");
    }

    #[test]
    fn nomi_liberi() {
        let esistenti: HashSet<String> = ["foto.jpg", "foto (1).jpg", "LEGGIMI"].iter().map(|s| s.to_string()).collect();
        assert_eq!(nome_libero("nuovo.pdf", &esistenti), "nuovo.pdf");
        assert_eq!(nome_libero("foto.jpg", &esistenti), "foto (2).jpg");
        assert_eq!(nome_libero("LEGGIMI", &esistenti), "LEGGIMI (1)");
    }

    #[test]
    fn motivi_di_installazione() {
        assert_eq!(spiega_installazione("Failure [INSTALL_FAILED_VERSION_DOWNGRADE: ...]"), t!("sul telefono c'è già una versione più recente"));
        assert!(spiega_installazione("Failure [STRANO]").contains("STRANO"));
    }
}
