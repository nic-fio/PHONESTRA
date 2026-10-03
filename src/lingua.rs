//! Lingua dell'interfaccia: italiano o inglese (SPECIFICATION §15.1).
//!
//! Nel codice i testi dell'interfaccia restano in italiano dentro [`t!`]: il
//! testo italiano è la chiave, la traduzione inglese sta nelle tabelle di
//! `data/en/` (un file per modulo, incluse nell'eseguibile). I segnaposto `{}`
//! si riempiono nell'ordine, dopo la traduzione, così l'inglese può spostarli.
//! I messaggi del registro (`eprintln!`) e di `phonestra-prova` restano in
//! italiano: sono per chi sviluppa.
//!
//! La lingua si sceglie all'avvio: quella delle preferenze se c'è, altrimenti
//! quella del sistema (`LC_ALL`, `LC_MESSAGES`, `LANG`): italiano se comincia
//! con `it`, inglese in tutti gli altri casi.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;

/// Le tabelle inglesi, un file per modulo.
const TABELLE: &[(&str, &str)] = &[
    ("app", include_str!("../data/en/app.toml")),
    ("appunti", include_str!("../data/en/appunti.toml")),
    ("avvisi", include_str!("../data/en/avvisi.toml")),
    ("azioni", include_str!("../data/en/azioni.toml")),
    ("cassetto", include_str!("../data/en/cassetto.toml")),
    ("collegamento", include_str!("../data/en/collegamento.toml")),
    ("componente", include_str!("../data/en/componente.toml")),
    ("finestra", include_str!("../data/en/finestra.toml")),
    ("notifiche", include_str!("../data/en/notifiche.toml")),
    ("phonestra", include_str!("../data/en/phonestra.toml")),
    ("prepara", include_str!("../data/en/prepara.toml")),
    ("procedura", include_str!("../data/en/procedura.toml")),
    ("ricevi", include_str!("../data/en/ricevi.toml")),
    ("varie", include_str!("../data/en/varie.toml")),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lingua {
    Italiano,
    Inglese,
}

impl Lingua {
    /// Il codice salvato nelle preferenze.
    pub fn codice(self) -> &'static str {
        match self {
            Lingua::Italiano => "it",
            Lingua::Inglese => "en",
        }
    }

    pub fn da_codice(codice: &str) -> Option<Self> {
        match codice {
            "it" => Some(Lingua::Italiano),
            "en" => Some(Lingua::Inglese),
            _ => None,
        }
    }
}

/// La lingua del sistema, dalle variabili della localizzazione: la prima non
/// vuota tra `LC_ALL`, `LC_MESSAGES` e `LANG`.
pub fn di_sistema() -> Lingua {
    let valore = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find(|v| !v.is_empty())
        .unwrap_or_default();
    da_locale(&valore)
}

/// `it_IT.UTF-8`, `it` → italiano; tutto il resto (compresi `C` e `POSIX`) → inglese.
pub fn da_locale(valore: &str) -> Lingua {
    if valore.starts_with("it") { Lingua::Italiano } else { Lingua::Inglese }
}

/// La lingua in uso: si decide una volta, al primo testo mostrato.
pub fn attuale() -> Lingua {
    static ATTUALE: OnceLock<Lingua> = OnceLock::new();
    *ATTUALE.get_or_init(|| {
        crate::configurazione::Preferenze::attuali()
            .lingua
            .as_deref()
            .and_then(Lingua::da_codice)
            .unwrap_or_else(di_sistema)
    })
}

/// Tutte le tabelle in una sola: testo italiano → testo inglese.
pub fn tabella() -> &'static HashMap<String, String> {
    static TABELLA: OnceLock<HashMap<String, String>> = OnceLock::new();
    TABELLA.get_or_init(|| {
        let mut tutte = HashMap::new();
        for (nome, testo) in TABELLE {
            match toml::from_str::<HashMap<String, String>>(testo) {
                Ok(t) => tutte.extend(t),
                Err(e) => eprintln!("[lingua] data/en/{nome}.toml non valido: {e}"),
            }
        }
        tutte
    })
}

/// Il testo nella lingua in uso. Un testo senza traduzione resta in italiano
/// (`cargo test` controlla che non ne manchino).
pub fn testo(italiano: &'static str) -> &'static str {
    match attuale() {
        Lingua::Italiano => italiano,
        Lingua::Inglese => tabella().get(italiano).map(String::as_str).unwrap_or(italiano),
    }
}

/// Riempie i `{}` del modello, nell'ordine, con gli argomenti.
pub fn componi(modello: &str, argomenti: &[&dyn Display]) -> String {
    let mut uscita = String::with_capacity(modello.len() + 16);
    let mut argomenti = argomenti.iter();
    let mut resto = modello;
    while let Some(i) = resto.find("{}") {
        uscita.push_str(&resto[..i]);
        match argomenti.next() {
            Some(a) => uscita.push_str(&a.to_string()),
            None => uscita.push_str("{}"),
        }
        resto = &resto[i + 2..];
    }
    uscita.push_str(resto);
    uscita
}

/// Un testo dell'interfaccia: `t!("Notifiche")` dà `&'static str`,
/// `t!("Collegamento a {}…", nome)` dà `String`.
#[macro_export]
macro_rules! t {
    ($testo:literal) => {
        $crate::lingua::testo($testo)
    };
    ($testo:literal, $($argomento:expr),+ $(,)?) => {
        $crate::lingua::componi($crate::lingua::testo($testo), &[$(&$argomento as &dyn std::fmt::Display),+])
    };
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn locale() {
        assert_eq!(da_locale("it_IT.UTF-8"), Lingua::Italiano);
        assert_eq!(da_locale("it_CH"), Lingua::Italiano);
        assert_eq!(da_locale("en_US.UTF-8"), Lingua::Inglese);
        assert_eq!(da_locale("de_DE.UTF-8"), Lingua::Inglese);
        assert_eq!(da_locale("C"), Lingua::Inglese);
        assert_eq!(da_locale(""), Lingua::Inglese);
    }

    #[test]
    fn segnaposto() {
        assert_eq!(componi("{} di {}", &[&1, &"due"]), "1 di due");
        assert_eq!(componi("niente", &[]), "niente");
        assert_eq!(componi("{} e {}", &[&"solo"]), "solo e {}");
    }
}
