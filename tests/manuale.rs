//! I manuali (`docs/Phonestra_Manuale_Tecnico.html` e
//! `docs/Phonestra_Manuale_Utente.html`) restano allineati ai sorgenti. Sono
//! generati da `docs/sorgenti/` (`python3 docs/sorgenti/build.py`); il
//! controllo li rigenera e verifica che coincidano coi file pubblicati, che la
//! mappa dei file sia completa e che i simboli, i file, le variabili
//! d'ambiente e i comandi citati esistano davvero (manuale, «Il manuale e i
//! suoi controlli» del manuale tecnico).

use std::path::Path;
use std::process::Command;

#[test]
fn i_manuali_corrispondono_ai_sorgenti() {
    let radice = Path::new(env!("CARGO_MANIFEST_DIR"));
    let uscita = Command::new("python3")
        .arg(radice.join("docs/sorgenti/build.py"))
        .arg("--controlla")
        .output()
        .expect("serve python3 (con python3-pygments) per controllare il manuale");
    assert!(
        uscita.status.success(),
        "manuali non allineati al codice:\n{}{}",
        String::from_utf8_lossy(&uscita.stdout),
        String::from_utf8_lossy(&uscita.stderr)
    );
}
