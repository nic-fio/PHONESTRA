//! Controllo delle traduzioni: ogni testo `t!("…")` dei sorgenti ha la sua
//! traduzione inglese in `data/en/`, con gli stessi segnaposto `{}`, e nelle
//! tabelle non restano testi che il codice non usa più.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn sorgenti(cartella: &Path, uscita: &mut Vec<PathBuf>) {
    for voce in fs::read_dir(cartella).unwrap() {
        let p = voce.unwrap().path();
        if p.is_dir() {
            sorgenti(&p, uscita);
        } else if p.extension().is_some_and(|e| e == "rs") {
            uscita.push(p);
        }
    }
}

/// Legge la stringa Rust che comincia a `testo[0] == '"'`; restituisce il
/// contenuto senza sequenze di escape e la lunghezza consumata.
fn stringa(testo: &str) -> Option<(String, usize)> {
    let mut caratteri = testo.char_indices();
    if caratteri.next()?.1 != '"' {
        return None;
    }
    let mut valore = String::new();
    while let Some((i, c)) = caratteri.next() {
        match c {
            '"' => return Some((valore, i + 1)),
            '\\' => match caratteri.next()?.1 {
                'n' => valore.push('\n'),
                't' => valore.push('\t'),
                '\\' => valore.push('\\'),
                '"' => valore.push('"'),
                '\'' => valore.push('\''),
                // fine riga dopo «\»: si salta lo spazio iniziale della riga dopo
                '\n' => {
                    let resto = &testo[i + 2..];
                    let spazi = resto.len() - resto.trim_start().len();
                    for _ in 0..spazi {
                        caratteri.next();
                    }
                }
                altro => panic!("sequenza \\{altro} non gestita nella prova"),
            },
            c => valore.push(c),
        }
    }
    None
}

/// I testi di tutte le `t!(…)` di un file.
fn testi(sorgente: &str) -> Vec<String> {
    let mut uscita = Vec::new();
    let mut resto = sorgente;
    while let Some(i) = resto.find("t!(") {
        let prima = resto[..i].chars().last();
        resto = &resto[i + 3..];
        if prima.is_some_and(|c| c.is_alphanumeric() || c == '_') {
            continue; // p. es. `print!(`, `assert!(`
        }
        let dentro = resto.trim_start();
        if let Some((valore, _)) = stringa(dentro) {
            uscita.push(valore);
        }
    }
    uscita
}

#[test]
fn traduzioni_complete() {
    let radice = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut file = Vec::new();
    sorgenti(&radice.join("src"), &mut file);
    let mut usati: BTreeMap<String, String> = BTreeMap::new();
    for f in &file {
        if f.ends_with("lingua.rs") {
            continue;
        }
        for t in testi(&fs::read_to_string(f).unwrap()) {
            usati.entry(t).or_insert_with(|| f.strip_prefix(radice).unwrap().display().to_string());
        }
    }

    let mut tabella: BTreeMap<String, String> = BTreeMap::new();
    let mut doppi = Vec::new();
    for voce in fs::read_dir(radice.join("data/en")).unwrap() {
        let p = voce.unwrap().path();
        if p.ends_with("instructions.toml") {
            continue; // le istruzioni per marca in inglese: le controlla procedura.rs
        }
        let t: BTreeMap<String, String> = toml::from_str(&fs::read_to_string(&p).unwrap())
            .unwrap_or_else(|e| panic!("{} non valido: {e}", p.display()));
        for (k, v) in t {
            if let Some(prima) = tabella.insert(k.clone(), v.clone())
                && prima != v
            {
                doppi.push(k);
            }
        }
    }

    let mancanti: Vec<String> =
        usati.iter().filter(|(k, _)| !tabella.contains_key(*k)).map(|(k, f)| format!("{f}: {k:?}")).collect();
    let segnaposto: Vec<&String> =
        usati.keys().filter(|k| tabella.get(*k).is_some_and(|v| v.matches("{}").count() != k.matches("{}").count())).collect();
    let usati_k: BTreeSet<&String> = usati.keys().collect();
    let inutili: Vec<&String> = tabella.keys().filter(|k| !usati_k.contains(k)).collect();

    assert!(mancanti.is_empty(), "testi senza traduzione in data/en/:\n{}", mancanti.join("\n"));
    assert!(segnaposto.is_empty(), "traduzioni con un numero diverso di {{}}: {segnaposto:?}");
    assert!(inutili.is_empty(), "traduzioni che il codice non usa più: {inutili:?}");
    assert!(doppi.is_empty(), "stesso testo tradotto in due modi: {doppi:?}");
}
