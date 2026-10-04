//! Stato dei telefoni collegati col cavo, letto da `/sys/bus/usb/devices`
//! senza aprire il dispositivo (quindi senza permessi speciali).
//!
//! Interfacce che contano (verificate sul Galaxy S23+, notes/connection-tests.md):
//! `06/01/01` MTP/PTP, `ff/42/01` ADB.

use std::fs;
use std::path::Path;

use crate::t;

const INTERFACCIA_MTP: (u8, u8, u8) = (0x06, 0x01, 0x01);
const INTERFACCIA_ADB: (u8, u8, u8) = (0xff, 0x42, 0x01);

/// Codici USB dei produttori di telefoni Android: servono a riconoscere un
/// telefono in «Solo ricarica» col Debug USB spento, che non offre né file né
/// debug (è com'esce dalla fabbrica: visto con un Galaxy S26, 27 set 2026).
const PRODUTTORI_ANDROID: &[u16] = &[
    0x04e8, // Samsung
    0x18d1, // Google
    0x2717, // Xiaomi
    0x22b8, // Motorola
    0x22d9, // Oppo, Realme
    0x2a70, // OnePlus
    0x12d1, // Huawei
    0x339b, // Honor
    0x0fce, // Sony
    0x1004, // LG
    0x2e04, // Nokia (HMD)
    0x0b05, // Asus
    0x17ef, // Lenovo
    0x19d2, // ZTE
    0x2d95, // Vivo
    0x2b0e, // Nothing
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stato {
    /// Solo trasferimento file: il Debug USB è spento.
    DebugSpento,
    /// Interfaccia ADB presente (se autorizzato si scopre solo collegandosi).
    DebugAttivo,
    /// Interfaccia ADB presente ma nessuna MTP/PTP: in «Solo ricarica» il
    /// permesso di systemd (uaccess) non c'è e l'accesso può essere negato.
    DebugAttivoSoloRicarica,
    /// Telefono Android in «Solo ricarica» col Debug USB spento: né file né
    /// debug, se ne conosce solo il produttore.
    SoloRicarica,
}

#[derive(Debug, Clone)]
pub struct DispositivoUsb {
    pub porta: String,
    pub vendor: u16,
    pub prodotto: u16,
    pub produttore: String,
    pub nome: String,
    pub seriale: String,
    pub stato: Stato,
}

/// Tutti i dispositivi USB collegati (esclusi hub e controller), come
/// «nome (produttore) vvvv:pppp»: per la diagnosi quando il telefono non si
/// vede (la procedura li mostra, chi prova manda una foto).
pub fn tutti() -> Vec<String> {
    let mut elenco = Vec::new();
    let Ok(voci) = fs::read_dir("/sys/bus/usb/devices") else { return elenco };
    for voce in voci.flatten() {
        let dir = voce.path();
        let porta = voce.file_name().to_string_lossy().to_string();
        if porta.contains(':') || porta.starts_with("usb") || esadecimale(&dir.join("bDeviceClass")) == Some(0x09) {
            continue;
        }
        let (Some(vendor), Some(prodotto)) = (esadecimale(&dir.join("idVendor")), esadecimale(&dir.join("idProduct"))) else {
            continue;
        };
        let (nome, produttore) = (leggi(&dir.join("product")), leggi(&dir.join("manufacturer")));
        let nome = match (nome.is_empty(), produttore.is_empty()) {
            (true, true) => t!("senza nome").to_string(),
            (false, true) => nome,
            (true, false) => produttore,
            (false, false) => format!("{nome} ({produttore})"),
        };
        let telefono = if PRODUTTORI_ANDROID.contains(&vendor) { format!(" {}", t!("← telefono")) } else { String::new() };
        elenco.push(format!("{nome} {vendor:04x}:{prodotto:04x}{telefono}"));
    }
    elenco.sort();
    elenco
}

fn leggi(percorso: &Path) -> String {
    fs::read_to_string(percorso).map(|s| s.trim().to_string()).unwrap_or_default()
}

fn esadecimale(percorso: &Path) -> Option<u16> {
    u16::from_str_radix(&leggi(percorso), 16).ok()
}

/// Elenca i dispositivi USB che espongono MTP o ADB: i telefoni Android.
pub fn telefoni() -> Vec<DispositivoUsb> {
    let mut trovati = Vec::new();
    let Ok(voci) = fs::read_dir("/sys/bus/usb/devices") else { return trovati };
    for voce in voci.flatten() {
        let dir = voce.path();
        let porta = voce.file_name().to_string_lossy().to_string();
        // Solo i dispositivi, non le loro interfacce («1-2:1.0») né i controller.
        if porta.contains(':') || porta.starts_with("usb") {
            continue;
        }
        let (Some(vendor), Some(prodotto)) = (esadecimale(&dir.join("idVendor")), esadecimale(&dir.join("idProduct"))) else {
            continue;
        };
        let mut mtp = false;
        let mut adb = false;
        if let Ok(interfacce) = fs::read_dir(&dir) {
            for i in interfacce.flatten() {
                let p = i.path();
                if !i.file_name().to_string_lossy().starts_with(&format!("{porta}:")) {
                    continue;
                }
                let classe = (
                    esadecimale(&p.join("bInterfaceClass")).unwrap_or(0) as u8,
                    esadecimale(&p.join("bInterfaceSubClass")).unwrap_or(0) as u8,
                    esadecimale(&p.join("bInterfaceProtocol")).unwrap_or(0) as u8,
                );
                mtp |= classe == INTERFACCIA_MTP;
                adb |= classe == INTERFACCIA_ADB;
            }
        }
        let stato = match (mtp, adb) {
            (_, false) if mtp => Stato::DebugSpento,
            (true, true) => Stato::DebugAttivo,
            (false, true) => Stato::DebugAttivoSoloRicarica,
            _ if PRODUTTORI_ANDROID.contains(&vendor) => Stato::SoloRicarica,
            _ => continue,
        };
        trovati.push(DispositivoUsb {
            vendor,
            prodotto,
            produttore: leggi(&dir.join("manufacturer")),
            nome: leggi(&dir.join("product")),
            seriale: leggi(&dir.join("serial")),
            stato,
            porta,
        });
    }
    trovati.sort_by(|a, b| a.porta.cmp(&b.porta));
    trovati
}
