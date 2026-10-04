//! Collegamento al telefono (cavo o Wi-Fi TLS) con la chiave di Phonestra,
//! e le impostazioni che Phonestra applica da solo (SPECIFICATION §5.2).

use std::net::SocketAddr;

use adb_client::ADBDeviceExt;
use adb_client::tcp::ADBTcpDevice;
use adb_client::usb::ADBUSBDevice;
use anyhow::{Context, Result, bail};

use crate::{configurazione, t};

pub struct Collegamento {
    dispositivo: Box<dyn ADBDeviceExt>,
}

impl Collegamento {
    /// Via cavo. La prima volta il telefono chiede «Consentire il debug USB?».
    ///
    /// Subito dopo l'autorizzazione il primo comando può fallire («got AUTH in
    /// response instead of OKAY», visto il 26 set 2026): si riprova da capo.
    pub fn usb(vendor: u16, prodotto: u16) -> Result<Self> {
        let chiave = configurazione::chiave()?;
        let mut ultimo_errore = None;
        for _ in 0..3 {
            let esito = ADBUSBDevice::new_with_custom_private_key(vendor, prodotto, chiave.clone())
                .context(t!("collegamento USB non riuscito (debug non autorizzato o accesso negato?)"))
                .and_then(|d| {
                    let mut c = Self { dispositivo: Box::new(d) };
                    c.shell("echo ok")?;
                    Ok(c)
                });
            match esito {
                Ok(c) => return Ok(c),
                Err(e) => ultimo_errore = Some(e),
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Err(ultimo_errore.expect("almeno un tentativo"))
    }

    /// Via Wi-Fi, all'indirizzo trovato con la ricerca mDNS.
    pub fn wifi(indirizzo: SocketAddr) -> Result<Self> {
        let chiave = configurazione::chiave()?;
        let d = ADBTcpDevice::new_with_custom_private_key(indirizzo, chiave)
            .with_context(|| t!("collegamento Wi-Fi a {} non riuscito", indirizzo))?;
        Ok(Self { dispositivo: Box::new(d) })
    }

    /// Esegue un comando di shell e restituisce l'uscita standard.
    pub fn shell(&mut self, comando: &str) -> Result<String> {
        let mut uscita = Vec::new();
        let mut errori = Vec::new();
        self.dispositivo
            .shell_command(&comando, Some(&mut uscita), Some(&mut errori))
            .with_context(|| t!("comando «{}» non riuscito", comando))?;
        if uscita.is_empty() && !errori.is_empty() {
            bail!("{}", String::from_utf8_lossy(&errori).trim());
        }
        Ok(String::from_utf8_lossy(&uscita).trim().to_string())
    }

    /// Livello della batteria in percentuale.
    pub fn batteria(&mut self) -> Result<u8> {
        let testo = self.shell("dumpsys battery")?;
        testo
            .lines()
            .find_map(|r| r.trim().strip_prefix("level:").and_then(|v| v.trim().parse().ok()))
            .context("livello della batteria non trovato")
    }

    fn proprieta(&mut self, nome: &str) -> Result<String> {
        self.shell(&format!("getprop {nome}"))
    }

    /// Legge i dati per la configurazione del PC.
    pub fn descrivi(&mut self) -> Result<configurazione::Telefono> {
        let modello = self.proprieta("ro.product.model")?;
        let nome = match self.shell("settings get global device_name")? {
            n if n.is_empty() || n == "null" => modello.clone(),
            n => n,
        };
        let (modello_commerciale, tablet) = leggi_modello(&self.shell(COMANDO_MODELLO)?);
        Ok(configurazione::Telefono {
            seriale: self.proprieta("ro.serialno")?,
            nome,
            modello,
            nome_scelto: None,
            modello_commerciale,
            tablet,
            android: self.proprieta("ro.build.version.release")?,
            ultimo_indirizzo: None,
            spegnimento_originale: None,
            volume_originale: None,
            preferiti: Vec::new(),
        })
    }

    /// Autorizzazione senza scadenza e Debug wireless acceso.
    ///
    /// La prima volta su una rete Android chiede sul telefono «Consentire il
    /// debug wireless su questa rete?» e rimette `adb_wifi_enabled` a 0 finché
    /// l'utente non accetta: restituisce `false` in quel caso.
    pub fn prepara_wifi(&mut self) -> Result<bool> {
        self.shell("settings put global adb_allowed_connection_time 0")?;
        self.shell("settings put global adb_wifi_enabled 1")?;
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if self.shell("settings get global adb_wifi_enabled")? == "1" {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// Comando che legge il nome commerciale del modello e se è un tablet: il nome
/// sta in proprietà diverse secondo la marca; Samsung lo tiene nel nome di
/// fabbrica del dispositivo (`default_device_name`, «Galaxy S23+»). Ultima
/// riga: `ro.build.characteristics`. Risposta da leggere con [`leggi_modello`].
pub const COMANDO_MODELLO: &str = "getprop ro.product.marketname; getprop ro.product.vendor.marketname; \
     getprop ro.product.odm.marketname; getprop ro.config.marketing_name; \
     settings get global default_device_name; getprop ro.build.characteristics";

/// Nome commerciale (se c'è) e «è un tablet» dalla risposta di [`COMANDO_MODELLO`].
pub fn leggi_modello(risposta: &str) -> (Option<String>, bool) {
    let righe: Vec<&str> = risposta.lines().map(str::trim).collect();
    let (caratteristiche, nomi) = righe.split_last().unwrap_or((&"", &[]));
    let nome = nomi.iter().find(|n| !n.is_empty() && **n != "null").map(|n| n.to_string());
    (nome, caratteristiche.split(',').any(|c| c == "tablet"))
}


#[cfg(test)]
mod prove {
    use super::*;

    #[test]
    fn lettura_del_modello() {
        assert_eq!(leggi_modello("\n\n\n\nGalaxy S23+\nphone"), (Some("Galaxy S23+".into()), false));
        assert_eq!(leggi_modello("Redmi Pad SE\n\n\n\nnull\ntablet"), (Some("Redmi Pad SE".into()), true));
        assert_eq!(leggi_modello("\n\n\n\nnull\nnosdcard,tablet"), (None, true));
    }
}
