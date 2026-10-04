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
        Ok(configurazione::Telefono {
            seriale: self.proprieta("ro.serialno")?,
            nome,
            modello,
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
