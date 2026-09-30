//! Il manuale tecnico (`docs/manuale-tecnico.html`) resta allineato ai sorgenti.
//! È generato da `docs/sorgenti/` (`python3 docs/sorgenti/build.py`); il
//! controllo lo rigenera e verifica che coincida col file pubblicato, che la
//! mappa dei file sia completa e che i simboli, i file, le variabili
//! d'ambiente e i comandi citati esistano davvero (manuale, «Il manuale e i
//! suoi controlli»).

use std::path::Path;
use std::process::Command;

#[test]
fn il_manuale_corrisponde_ai_sorgenti() {
    let radice = Path::new(env!("CARGO_MANIFEST_DIR"));
    let uscita = Command::new("python3")
        .arg(radice.join("docs/sorgenti/build.py"))
        .arg("--controlla")
        .output()
        .expect("serve python3 (con python3-pygments) per controllare il manuale");
    assert!(
        uscita.status.success(),
        "manuale tecnico non allineato al codice:\n{}{}",
        String::from_utf8_lossy(&uscita.stdout),
        String::from_utf8_lossy(&uscita.stderr)
    );
}
