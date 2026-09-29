//! Il manuale tecnico (`docs/manuale-tecnico.html`) resta allineato ai sorgenti:
//! ogni file ha la sua riga nella mappa, con le righe giuste e una descrizione.
//! I numeri li riscrive `python3 docs/aggiorna-numeri.py`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const RIMEDIO: &str = "lancia «python3 docs/aggiorna-numeri.py» e descrivi i file nuovi nella mappa";

/// Gli stessi file che conta `docs/aggiorna-numeri.py`.
fn sorgenti(radice: &Path) -> BTreeMap<String, usize> {
    fn raccogli(radice: &Path, cartella: &Path, estensione: &str, file: &mut Vec<String>) {
        let Ok(voci) = fs::read_dir(cartella) else { return };
        for voce in voci.flatten() {
            let p = voce.path();
            if p.is_dir() {
                raccogli(radice, &p, estensione, file);
            } else if p.extension().is_some_and(|e| e == estensione) {
                file.push(p.strip_prefix(radice).unwrap().to_string_lossy().into_owned());
            }
        }
    }
    let mut file = Vec::new();
    raccogli(radice, &radice.join("src"), "rs", &mut file);
    raccogli(radice, &radice.join("telefono/aiuto/src"), "java", &mut file);
    for voce in fs::read_dir(radice.join("costruzione")).unwrap().flatten() {
        let p = voce.path();
        if p.is_file() && p.extension().is_none_or(|e| e != "svg") {
            file.push(p.strip_prefix(radice).unwrap().to_string_lossy().into_owned());
        }
    }
    file.push("telefono/aiuto/costruisci.sh".into());
    file.push("docs/aggiorna-numeri.py".into());
    file.push("docs/disegna-diagrammi.py".into());
    for voce in fs::read_dir(radice.join("tests")).unwrap().flatten() {
        let p = voce.path();
        if p.extension().is_some_and(|e| e == "rs") {
            file.push(p.strip_prefix(radice).unwrap().to_string_lossy().into_owned());
        }
    }
    file.into_iter()
        .map(|f| {
            let righe = fs::read(radice.join(&f)).unwrap().iter().filter(|b| **b == b'\n').count();
            (f, righe)
        })
        .collect()
}

/// Il valore di un attributo nel testo di un'etichetta.
fn attributo<'a>(etichetta: &'a str, nome: &str) -> Option<&'a str> {
    let inizio = etichetta.find(&format!("{nome}=\""))? + nome.len() + 2;
    let fine = etichetta[inizio..].find('"')?;
    Some(&etichetta[inizio..inizio + fine])
}

#[test]
fn la_mappa_dei_file_corrisponde_ai_sorgenti() {
    let radice = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manuale = fs::read_to_string(radice.join("docs/manuale-tecnico.html")).unwrap();
    let attesi = sorgenti(radice);

    let mut nella_mappa = BTreeMap::new();
    for riga in manuale.split("<tr ").skip(1) {
        let etichetta = &riga[..riga.find('>').unwrap_or(riga.len())];
        if let (Some(file), Some(righe)) = (attributo(etichetta, "data-file"), attributo(etichetta, "data-righe")) {
            assert!(!riga.contains("DA DESCRIVERE"), "{file} senza descrizione nella mappa: {RIMEDIO}");
            nella_mappa.insert(file.to_string(), righe.parse::<usize>().unwrap());
        }
    }
    for (file, righe) in &attesi {
        match nella_mappa.get(file) {
            None => panic!("{file} manca nella mappa del manuale: {RIMEDIO}"),
            Some(n) if n != righe => panic!("{file}: {n} righe nel manuale, {righe} nel sorgente: {RIMEDIO}"),
            _ => {}
        }
    }
    for file in nella_mappa.keys() {
        assert!(attesi.contains_key(file), "{file} è nella mappa ma non esiste più: {RIMEDIO}");
    }
    let totale: usize = attesi.values().sum();
    let scritto = manuale
        .split("<tr ")
        .find(|r| r.starts_with("data-parte=\"totale\""))
        .and_then(|r| attributo(r, "data-righe"))
        .and_then(|n| n.parse::<usize>().ok());
    assert_eq!(scritto, Some(totale), "totale delle righe nel manuale: {RIMEDIO}");
}
