//! Dati di Phonestra sul PC: tutto sta in `$XDG_CONFIG_HOME/Phonestra`
//! (di norma `~/.config/Phonestra`), niente altrove (SPECIFICHE §4).

use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;

use anyhow::{Context, Result};
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use serde::{Deserialize, Serialize};

/// Dimensione della chiave RSA usata da ADB.
const BIT_CHIAVE: usize = 2048;

pub fn cartella() -> Result<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(std::env::var_os("HOME").context("HOME non impostata")?).join(".config"),
    };
    let dir = cartella_in(&base);
    fs::create_dir_all(&dir).with_context(|| format!("impossibile creare {}", dir.display()))?;
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    Ok(dir)
}

/// `base/Phonestra`.
pub fn cartella_in(base: &std::path::Path) -> PathBuf {
    base.join("Phonestra")
}

/// Percorso della chiave privata ADB di Phonestra; la crea (permessi 600) se manca.
///
/// È una chiave diversa da quella di `adb` (`~/.android/adbkey`): il telefono
/// chiede di autorizzare Phonestra come un computer nuovo.
pub fn chiave() -> Result<PathBuf> {
    let percorso = cartella()?.join("adbkey");
    if percorso.exists() {
        return Ok(percorso);
    }
    let chiave = RsaPrivateKey::new(&mut rand::thread_rng(), BIT_CHIAVE)?;
    let pem = chiave.to_pkcs8_pem(LineEnding::LF)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&percorso)?;
    file.write_all(pem.as_bytes())?;
    Ok(percorso)
}

/// Un telefono configurato.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telefono {
    /// Numero di serie: è anche nel nome mDNS `adb-<seriale>-<suffisso>`.
    pub seriale: String,
    /// Nome mostrato all'utente (dal telefono, modificabile).
    pub nome: String,
    pub modello: String,
    pub android: String,
    /// Ultimo indirizzo Wi-Fi che ha funzionato: si prova per primo, prima
    /// della ricerca mDNS (che a volte non risponde al primo colpo).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ultimo_indirizzo: Option<std::net::SocketAddr>,
    /// Tempo di spegnimento dello schermo dell'utente (ms), salvato prima che
    /// una sessione lo allunghi: se la sessione cade senza ripristinarlo, lo si
    /// rimette al collegamento successivo (SPECIFICHE §5.9).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spegnimento_originale: Option<u64>,
    /// Volume multimediale dell'utente, salvato prima che la sessione lo porti
    /// al massimo: se la sessione cade senza ripristinarlo, lo si rimette al
    /// collegamento successivo (SPECIFICHE §10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_originale: Option<u32>,
    /// App preferite (pacchetti) in cima al drawer, nell'ordine scelto.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preferiti: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Telefoni {
    #[serde(default, rename = "telefono")]
    pub elenco: Vec<Telefono>,
}

impl Telefoni {
    fn percorso() -> Result<PathBuf> {
        Ok(cartella()?.join("telefoni.toml"))
    }

    pub fn carica() -> Result<Self> {
        let percorso = Self::percorso()?;
        match fs::read_to_string(&percorso) {
            Ok(testo) => Ok(toml::from_str(&testo)
                .with_context(|| format!("{} non valido", percorso.display()))?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn salva(&self) -> Result<()> {
        fs::write(Self::percorso()?, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Ricorda l'indirizzo che ha appena funzionato per il telefono `seriale`.
    pub fn ricorda_indirizzo(seriale: &str, indirizzo: std::net::SocketAddr) -> Result<()> {
        let mut telefoni = Self::carica()?;
        if let Some(t) = telefoni.elenco.iter_mut().find(|t| t.seriale == seriale)
            && t.ultimo_indirizzo != Some(indirizzo)
        {
            t.ultimo_indirizzo = Some(indirizzo);
            telefoni.salva()?;
        }
        Ok(())
    }

    /// Salva (o cancella, con `None`) il tempo di spegnimento originale.
    pub fn ricorda_spegnimento(seriale: &str, valore: Option<u64>) -> Result<()> {
        let mut telefoni = Self::carica()?;
        if let Some(t) = telefoni.elenco.iter_mut().find(|t| t.seriale == seriale) {
            t.spegnimento_originale = valore;
            telefoni.salva()?;
        }
        Ok(())
    }

    /// Salva (o cancella, con `None`) il volume multimediale originale.
    pub fn ricorda_volume(seriale: &str, valore: Option<u32>) -> Result<()> {
        let mut telefoni = Self::carica()?;
        if let Some(t) = telefoni.elenco.iter_mut().find(|t| t.seriale == seriale) {
            t.volume_originale = valore;
            telefoni.salva()?;
        }
        Ok(())
    }

    /// Salva le app preferite del telefono `seriale`.
    pub fn imposta_preferiti(seriale: &str, preferiti: &[String]) -> Result<()> {
        let mut telefoni = Self::carica()?;
        if let Some(t) = telefoni.elenco.iter_mut().find(|t| t.seriale == seriale) {
            t.preferiti = preferiti.to_vec();
            telefoni.salva()?;
        }
        Ok(())
    }

    /// Cambia il nome mostrato del telefono `seriale`.
    pub fn rinomina(seriale: &str, nome: &str) -> Result<()> {
        let mut telefoni = Self::carica()?;
        if let Some(t) = telefoni.elenco.iter_mut().find(|t| t.seriale == seriale) {
            t.nome = nome.to_string();
            telefoni.salva()?;
        }
        Ok(())
    }

    /// Mette il telefono `seriale` in cima: è quello che si usa all'avvio
    /// («l'ultimo usato»).
    pub fn metti_primo(seriale: &str) -> Result<()> {
        let mut telefoni = Self::carica()?;
        if let Some(i) = telefoni.elenco.iter().position(|t| t.seriale == seriale) {
            let t = telefoni.elenco.remove(i);
            telefoni.elenco.insert(0, t);
            telefoni.salva()?;
        }
        Ok(())
    }

    /// Toglie il telefono `seriale` dalla configurazione.
    pub fn dimentica(seriale: &str) -> Result<()> {
        let mut telefoni = Self::carica()?;
        telefoni.elenco.retain(|t| t.seriale != seriale);
        telefoni.salva()
    }

    /// Aggiunge il telefono o aggiorna quello con lo stesso numero di serie.
    pub fn registra(&mut self, telefono: Telefono) {
        match self.elenco.iter_mut().find(|t| t.seriale == telefono.seriale) {
            // Rifare la configurazione non deve perdere i preferiti.
            Some(esistente) => {
                let preferiti = std::mem::take(&mut esistente.preferiti);
                *esistente = telefono;
                if esistente.preferiti.is_empty() {
                    esistente.preferiti = preferiti;
                }
            }
            None => self.elenco.push(telefono),
        }
    }
}

/// Preferenze dell'utente (`preferenze.toml`), valide per tutti i telefoni.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferenze {
    /// Esc fa «Indietro» nelle finestre delle app (SPECIFICHE §7.5).
    pub esc_indietro: bool,
    /// Le notifiche nuove del telefono diventano notifiche del sistema (§8).
    pub avvisi: bool,
    /// Negli avvisi solo il nome dell'app, senza mittente e testo.
    pub solo_nome_app: bool,
    /// App che non devono avvisare (pacchetti).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub app_silenziate: Vec<String>,
    /// Cartella del telefono, dentro `/sdcard`, dove arrivano i file inviati.
    pub cartella_file: String,
    /// Cartella del PC dove arrivano i file ricevuti dal telefono; se manca,
    /// la cartella Scaricati del sistema ([`Preferenze::cartella_ricevuti`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cartella_ricevuti: Option<PathBuf>,
}

impl Default for Preferenze {
    fn default() -> Self {
        Self { esc_indietro: true, avvisi: true, solo_nome_app: false, app_silenziate: Vec::new(), cartella_file: "Download".into(), cartella_ricevuti: None }
    }
}

fn preferenze_attuali() -> &'static std::sync::RwLock<Preferenze> {
    static ATTUALI: std::sync::OnceLock<std::sync::RwLock<Preferenze>> = std::sync::OnceLock::new();
    ATTUALI.get_or_init(|| {
        let lette = cartella()
            .ok()
            .and_then(|c| fs::read_to_string(c.join("preferenze.toml")).ok())
            .and_then(|t| toml::from_str(&t).map_err(|e| eprintln!("[configurazione] preferenze.toml non valido: {e}")).ok());
        std::sync::RwLock::new(lette.unwrap_or_default())
    })
}

impl Preferenze {
    /// Le preferenze in uso (lette dal file la prima volta).
    pub fn attuali() -> Self {
        preferenze_attuali().read().unwrap().clone()
    }

    /// Dove salvare i file ricevuti dal telefono: la cartella scelta nelle
    /// Preferenze, altrimenti Scaricati (`XDG_DOWNLOAD_DIR`, «Downloads» su un
    /// PC in inglese), altrimenti la cartella personale.
    pub fn cartella_ricevuti(&self) -> PathBuf {
        self.cartella_ricevuti
            .clone()
            .or_else(|| gtk::glib::user_special_dir(gtk::glib::UserDirectory::Downloads))
            .unwrap_or_else(gtk::glib::home_dir)
    }

    /// Cambia le preferenze e le salva.
    pub fn cambia(modifica: impl FnOnce(&mut Self)) -> Result<()> {
        let nuove = {
            let mut p = preferenze_attuali().write().unwrap();
            modifica(&mut p);
            p.clone()
        };
        fs::write(cartella()?.join("preferenze.toml"), toml::to_string_pretty(&nuove)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn preferenze_mancanti_prendono_il_valore_predefinito() {
        let p: Preferenze = toml::from_str("solo_nome_app = true").unwrap();
        assert!(p.solo_nome_app && p.esc_indietro && p.avvisi);
        assert_eq!(p.cartella_file, "Download");
        assert_eq!(p.cartella_ricevuti, None);
    }

}
