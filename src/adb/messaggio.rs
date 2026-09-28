//! Messaggi del protocollo ADB: intestazione di 24 byte little-endian
//! (comando, arg0, arg1, lunghezza, somma dei byte, comando ^ 0xffffffff)
//! seguita dai dati.

use anyhow::{Result, bail};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const CNXN: u32 = 0x4e58_4e43;
pub const OPEN: u32 = 0x4e45_504f;
pub const OKAY: u32 = 0x5941_4b4f;
pub const CLSE: u32 = 0x4553_4c43;
pub const WRTE: u32 = 0x4554_5257;
pub const STLS: u32 = 0x534c_5453;

/// Versione del protocollo annunciata: dalla 0x01000001 la somma dei byte è
/// facoltativa, ma la calcoliamo comunque per i telefoni più vecchi.
pub const VERSIONE: u32 = 0x0100_0001;
pub const VERSIONE_STLS: u32 = 0x0100_0000;
/// Dimensione massima dei dati che dichiariamo di accettare.
pub const MAX_DATI: u32 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Messaggio {
    pub comando: u32,
    pub arg0: u32,
    pub arg1: u32,
    pub dati: Vec<u8>,
}

pub fn nome(comando: u32) -> String {
    String::from_utf8_lossy(&comando.to_le_bytes()).to_string()
}

impl Messaggio {
    pub fn new(comando: u32, arg0: u32, arg1: u32, dati: impl Into<Vec<u8>>) -> Self {
        Self { comando, arg0, arg1, dati: dati.into() }
    }

    pub fn in_byte(&self) -> Vec<u8> {
        let somma = self.dati.iter().map(|&b| b as u32).fold(0u32, u32::wrapping_add);
        let mut v = Vec::with_capacity(24 + self.dati.len());
        for campo in [self.comando, self.arg0, self.arg1, self.dati.len() as u32, somma, self.comando ^ 0xffff_ffff] {
            v.extend_from_slice(&campo.to_le_bytes());
        }
        v.extend_from_slice(&self.dati);
        v
    }

    pub async fn scrivi<W: AsyncWrite + Unpin>(&self, w: &mut W) -> Result<()> {
        w.write_all(&self.in_byte()).await?;
        w.flush().await?;
        Ok(())
    }

    pub async fn leggi<R: AsyncRead + Unpin>(r: &mut R) -> Result<Self> {
        let mut intestazione = [0u8; 24];
        r.read_exact(&mut intestazione).await?;
        let campo = |i: usize| u32::from_le_bytes(intestazione[i * 4..i * 4 + 4].try_into().unwrap());
        let (comando, arg0, arg1, lunghezza, magico) = (campo(0), campo(1), campo(2), campo(3), campo(5));
        if magico != comando ^ 0xffff_ffff {
            bail!("messaggio ADB non valido (comando {:08x})", comando);
        }
        if lunghezza > 16 * 1024 * 1024 {
            bail!("messaggio ADB troppo grande ({lunghezza} byte)");
        }
        let mut dati = vec![0u8; lunghezza as usize];
        r.read_exact(&mut dati).await?;
        Ok(Self { comando, arg0, arg1, dati })
    }
}

#[cfg(test)]
mod prove {
    use super::*;

    #[tokio::test]
    async fn andata_e_ritorno() {
        let m = Messaggio::new(WRTE, 7, 9, b"ciao".to_vec());
        let byte = m.in_byte();
        assert_eq!(&byte[0..4], b"WRTE");
        assert_eq!(u32::from_le_bytes(byte[16..20].try_into().unwrap()), b"ciao".iter().map(|&b| b as u32).sum::<u32>());
        let letto = Messaggio::leggi(&mut byte.as_slice()).await.unwrap();
        assert_eq!(letto, m);
    }
}
