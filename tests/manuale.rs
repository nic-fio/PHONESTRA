//! I manuali (`docs/Technical Manual.html` e `docs/User Manual.html`, in
//! inglese) restano allineati ai sorgenti. Sono
//! generati da `docs/sorgenti/` (`python3 docs/sorgenti/build.py`); il
//! controllo li rigenera e verifica che coincidano coi file pubblicati, che la
//! mappa dei file sia completa e che i simboli, i file, le variabili
//! d'ambiente e i comandi citati esistano davvero, e che il testo non sia
//! rimasto in italiano (manuale tecnico, «The manuals and their checks»).

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
