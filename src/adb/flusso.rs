//! Controllo di flusso di ADB, parti pure (senza rete): banner del CNXN,
//! `OPEN` con la finestra, `OKAY` con i byte confermati e saldo di invio del
//! *delayed ack* (`notes/adb.md`).
//!
//! Dal sorgente di adbd (`adb.cpp`, `sockets.cpp`, `docs/dev/delayed_ack.md`):
//! - il *delayed ack* vale solo se **tutte e due** le parti annunciano
//!   `delayed_ack` nelle `features=` del CNXN;
//! - allora `arg1` dell'`OPEN` è quanti byte chi apre accetta senza conferma
//!   (dev'essere > 0; senza *delayed ack* dev'essere 0, altrimenti adbd chiude);
//! - ogni `OKAY` porta 4 byte (int32 LE, anche negativo) da aggiungere al saldo
//!   di chi manda; il primo, in risposta all'`OPEN`, è il saldo iniziale
//!   (adbd concede 32 MiB);
//! - chi manda toglie dal saldo i byte di ogni `WRTE` e manda finché il saldo
//!   è positivo (quindi può sforare di un blocco: adbd fa lo stesso).

use anyhow::{Result, bail};

use super::messaggio::{Messaggio, OKAY, OPEN};

/// Nome della funzione nel banner del CNXN.
pub const DELAYED_ACK: &str = "delayed_ack";

/// Banner del nostro CNXN. `abb_exec` non serve ancora: si annuncia solo ciò
/// che il client sa usare.
pub fn banner_host(delayed_ack: bool) -> Vec<u8> {
    let mut funzioni = vec!["shell_v2", "cmd", "stat_v2"];
    if delayed_ack {
        funzioni.push(DELAYED_ACK);
    }
    format!("host::features={}\0", funzioni.join(",")).into_bytes()
}

/// Funzioni annunciate in un banner CNXN
/// (`device::ro.product.name=…;…;features=a,b,c`).
pub fn funzioni(banner: &str) -> Vec<&str> {
    let dopo_tipo = banner.split_once("::").map_or(banner, |(_, resto)| resto);
    dopo_tipo
        .trim_end_matches('\0')
        .split(';')
        .filter_map(|voce| voce.strip_prefix("features="))
        .flat_map(|elenco| elenco.split(','))
        .filter(|f| !f.is_empty())
        .collect()
}

/// Il telefono annuncia il *delayed ack*?
pub fn offre_delayed_ack(banner: &str) -> bool {
    funzioni(banner).contains(&DELAYED_ACK)
}

/// `OPEN(locale, finestra, "servizio\0")`: `finestra` è `None` senza
/// *delayed ack* (arg1 = 0), altrimenti i byte che accettiamo senza conferma.
pub fn apertura(locale: u32, finestra: Option<u32>, servizio: &str) -> Messaggio {
    let mut destinazione = servizio.as_bytes().to_vec();
    destinazione.push(0);
    Messaggio::new(OPEN, locale, finestra.map_or(0, |f| f.max(1)), destinazione)
}

/// `OKAY(locale, remoto)`, con i 4 byte dei byte confermati se il *delayed
/// ack* è attivo (adbd rifiuta un `OKAY` vuoto in quel caso, e viceversa).
pub fn conferma(locale: u32, remoto: u32, byte: Option<u32>) -> Messaggio {
    let dati = byte.map(|n| n.to_le_bytes().to_vec()).unwrap_or_default();
    Messaggio::new(OKAY, locale, remoto, dati)
}

/// Byte confermati da un `OKAY` del telefono: `None` se è vuoto (senza
/// *delayed ack*), errore se il contenuto non è né vuoto né di 4 byte.
pub fn byte_confermati(m: &Messaggio) -> Result<Option<i32>> {
    match m.dati.len() {
        0 => Ok(None),
        4 => Ok(Some(i32::from_le_bytes(m.dati[..4].try_into()?))),
        n => bail!("OKAY con {n} byte di contenuto (attesi 0 o 4)"),
    }
}

/// Saldo di invio di un canale col *delayed ack*: quanti byte si possono
/// ancora mandare senza conferma. Parte dal minimo tra quanto concede il
/// telefono e la nostra finestra di invio (adbd concede 32 MiB: senza un
/// limite nostro un invio grande riempirebbe la connessione davanti ai tocchi
/// degli altri canali).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Saldo(i64);

impl Saldo {
    pub fn nuovo(concesso: i32, limite: u32) -> Self {
        Self(i64::from(concesso).min(i64::from(limite)))
    }

    /// Si può mandare il prossimo blocco (anche se sfora il saldo)?
    pub fn puo_mandare(&self) -> bool {
        self.0 > 0
    }

    pub fn mandati(&mut self, byte: usize) {
        self.0 -= byte as i64;
    }

    /// Byte confermati da un `OKAY` (possono essere 0 o negativi).
    pub fn confermati(&mut self, byte: i32) {
        self.0 += i64::from(byte);
    }

    pub fn valore(&self) -> i64 {
        self.0
    }
}

/// Una dimensione in byte scritta a mano: `65536`, `64k`, `1m` (k e m in
/// multipli di 1024).
pub fn dimensione(testo: &str) -> Option<u32> {
    let testo = testo.trim().to_ascii_lowercase();
    let (numero, per) = match testo.strip_suffix(['k', 'm']) {
        Some(n) if testo.ends_with('k') => (n, 1024),
        Some(n) => (n, 1024 * 1024),
        None => (testo.as_str(), 1),
    };
    numero.trim().parse::<u32>().ok()?.checked_mul(per)
}

#[cfg(test)]
mod prove {
    use super::*;

    const BANNER_S23: &str = "device::ro.product.name=x;ro.product.model=y;ro.product.device=z;\
        features=sendrecv_v2_brotli,remount_shell,sendrecv_v2,abb_exec,fixed_push_mkdir,\
        fixed_push_symlink_timestamp,abb,shell_v2,cmd,ls_v2,apex,stat_v2,delayed_ack";

    #[test]
    fn banner_nostro() {
        assert_eq!(banner_host(false), b"host::features=shell_v2,cmd,stat_v2\0");
        assert_eq!(banner_host(true), b"host::features=shell_v2,cmd,stat_v2,delayed_ack\0");
        let testo = String::from_utf8(banner_host(true)).unwrap();
        assert!(offre_delayed_ack(&testo));
    }

    #[test]
    fn funzioni_del_telefono() {
        assert!(offre_delayed_ack(BANNER_S23));
        assert!(funzioni(BANNER_S23).contains(&"abb_exec"));
        let vecchio = "device::ro.product.name=x;features=shell_v2,cmd\0";
        assert!(!offre_delayed_ack(vecchio));
        assert_eq!(funzioni(vecchio), ["shell_v2", "cmd"]);
        assert!(funzioni("device::").is_empty());
        // «delayed_ack» dentro un altro nome non conta.
        assert!(!offre_delayed_ack("device::features=delayed_ack_v2"));
    }

    #[test]
    fn apertura_con_e_senza_finestra() {
        let m = apertura(5, None, "exec:ls");
        assert_eq!((m.comando, m.arg0, m.arg1), (OPEN, 5, 0));
        assert_eq!(m.dati, b"exec:ls\0");
        assert_eq!(apertura(5, Some(262_144), "x").arg1, 262_144);
        // Col delayed ack arg1 = 0 farebbe chiudere il canale ad adbd.
        assert_eq!(apertura(5, Some(0), "x").arg1, 1);
    }

    #[test]
    fn conferme_andata_e_ritorno() {
        let vuota = conferma(3, 9, None);
        assert_eq!((vuota.comando, vuota.arg0, vuota.arg1), (OKAY, 3, 9));
        assert!(vuota.dati.is_empty());
        assert_eq!(byte_confermati(&vuota).unwrap(), None);
        let piena = conferma(3, 9, Some(65_536));
        assert_eq!(piena.dati, [0, 0, 1, 0]);
        assert_eq!(byte_confermati(&piena).unwrap(), Some(65_536));
        // adbd può mandare valori negativi.
        let negativa = Messaggio::new(OKAY, 1, 1, (-10i32).to_le_bytes().to_vec());
        assert_eq!(byte_confermati(&negativa).unwrap(), Some(-10));
        assert!(byte_confermati(&Messaggio::new(OKAY, 1, 1, vec![1, 2])).is_err());
    }

    #[test]
    fn dimensioni_scritte_a_mano() {
        assert_eq!(dimensione("65536"), Some(65_536));
        assert_eq!(dimensione("64k"), Some(65_536));
        assert_eq!(dimensione("64K"), Some(65_536));
        assert_eq!(dimensione("1m"), Some(1_048_576));
        assert_eq!(dimensione("5000m"), None);
        assert_eq!(dimensione("tanto"), None);
        assert_eq!(dimensione(""), None);
    }

    #[test]
    fn contabilita_del_saldo() {
        // adbd concede 32 MiB, noi ci limitiamo a 256 KiB.
        let mut s = Saldo::nuovo(32 * 1024 * 1024, 256 * 1024);
        assert_eq!(s.valore(), 256 * 1024);
        for _ in 0..4 {
            assert!(s.puo_mandare());
            s.mandati(64 * 1024);
        }
        assert!(!s.puo_mandare());
        s.confermati(0);
        assert!(!s.puo_mandare());
        s.confermati(64 * 1024);
        assert!(s.puo_mandare());
        // Si può sforare di un blocco, come adbd.
        s.mandati(100_000);
        assert!(s.valore() < 0 && !s.puo_mandare());
        s.confermati(-5);
        s.confermati(100_005);
        assert_eq!(s.valore(), 64 * 1024);
        // Concessione più piccola della nostra finestra.
        assert_eq!(Saldo::nuovo(1000, 256 * 1024).valore(), 1000);
        assert!(!Saldo::nuovo(0, 10).puo_mandare());
    }
}
