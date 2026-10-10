// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Il client contro un finto adbd in memoria: si controllano i messaggi che
//! escono e la contabilità del saldo, con e senza *delayed ack*.

use std::time::Duration;

use tokio::io::DuplexStream;

use super::messaggio::{self, CLSE, MAX_DATI, Messaggio, OKAY, OPEN, WRTE};
use super::{Adb, OpzioniCanale, Trasporto, flusso};

const BLOCCO: usize = 4096;

fn collega(delayed_ack: bool, finestra: u32) -> (Adb, DuplexStream) {
    let (nostro, telefono) = tokio::io::duplex(1024 * 1024);
    let (lettore, scrittore) = tokio::io::split(nostro);
    let trasporto = Trasporto { delayed_ack, max_payload: BLOCCO as u32, finestra };
    (Adb::avvia(lettore, scrittore, BLOCCO, delayed_ack, trasporto, "device::".into()), telefono)
}

async fn ricevi(t: &mut DuplexStream) -> Messaggio {
    tokio::time::timeout(Duration::from_secs(2), Messaggio::leggi(t)).await.expect("niente dal PC").unwrap()
}

/// Nessun messaggio dal PC per un po'.
async fn silenzio(t: &mut DuplexStream) {
    let esito = tokio::time::timeout(Duration::from_millis(150), Messaggio::leggi(t)).await;
    assert!(esito.is_err(), "il PC ha mandato {:?}", esito.unwrap().map(|m| messaggio::nome(m.comando)));
}

async fn manda(t: &mut DuplexStream, m: Messaggio) {
    m.scrivi(t).await.unwrap();
}

#[tokio::test]
async fn delayed_ack_saldo_e_conferme() {
    let (adb, mut tel) = collega(true, 3 * BLOCCO as u32);
    let apertura = tokio::spawn({
        let adb = adb.clone();
        async move { adb.apri("shell,v2,raw:cat").await.unwrap() }
    });
    let open = ricevi(&mut tel).await;
    assert_eq!(open.comando, OPEN);
    assert_eq!(open.arg1, 3 * BLOCCO as u32, "arg1 dell'OPEN = finestra");
    assert_eq!(open.dati, b"shell,v2,raw:cat\0");
    let locale = open.arg0;
    // adbd concede 32 MiB: il PC si limita alla sua finestra.
    manda(&mut tel, flusso::conferma(77, locale, Some(32 * 1024 * 1024))).await;
    let mut canale = apertura.await.unwrap();

    // 5 blocchi da scrivere, saldo per 3: ne partono 3 senza attese.
    let scrittura = tokio::spawn(async move {
        canale.scrivi(&vec![7u8; 5 * BLOCCO]).await.unwrap();
        canale
    });
    for _ in 0..3 {
        let m = ricevi(&mut tel).await;
        assert_eq!((m.comando, m.arg0, m.arg1, m.dati.len()), (WRTE, locale, 77, BLOCCO));
    }
    silenzio(&mut tel).await;
    // Un OKAY da 0 byte non sblocca niente, uno da un blocco ne fa partire uno.
    manda(&mut tel, flusso::conferma(77, locale, Some(0))).await;
    silenzio(&mut tel).await;
    manda(&mut tel, flusso::conferma(77, locale, Some(BLOCCO as u32))).await;
    assert_eq!(ricevi(&mut tel).await.comando, WRTE);
    silenzio(&mut tel).await;
    manda(&mut tel, flusso::conferma(77, locale, Some(2 * BLOCCO as u32))).await;
    assert_eq!(ricevi(&mut tel).await.comando, WRTE);
    let mut canale = scrittura.await.unwrap();

    // Dati dal telefono: ogni WRTE confermato subito, con i 4 byte.
    manda(&mut tel, Messaggio::new(WRTE, 77, locale, vec![1u8; 1000])).await;
    let okay = ricevi(&mut tel).await;
    assert_eq!((okay.comando, okay.arg0, okay.arg1), (OKAY, locale, 77));
    assert_eq!(flusso::byte_confermati(&okay).unwrap(), Some(1000));
    assert_eq!(canale.leggi().await.unwrap().len(), 1000);
    silenzio(&mut tel).await;
}

#[tokio::test]
async fn delayed_ack_conferma_alla_lettura() {
    let (adb, mut tel) = collega(true, 64 * 1024);
    let opzioni = OpzioniCanale { finestra: Some(8192), conferma_alla_lettura: true };
    let apertura = tokio::spawn({
        let adb = adb.clone();
        async move { adb.apri_con("localabstract:video", opzioni).await.unwrap() }
    });
    let open = ricevi(&mut tel).await;
    assert_eq!(open.arg1, 8192, "la finestra del canale vince su quella del trasporto");
    let locale = open.arg0;
    manda(&mut tel, flusso::conferma(5, locale, Some(32 * 1024 * 1024))).await;
    let mut canale = apertura.await.unwrap();
    manda(&mut tel, Messaggio::new(WRTE, 5, locale, vec![1u8; 3000])).await;
    manda(&mut tel, Messaggio::new(WRTE, 5, locale, vec![2u8; 500])).await;
    // Finché nessuno legge, nessuna conferma.
    silenzio(&mut tel).await;
    // Letti a pezzi: si conferma il blocco intero quando esce dalla coda.
    assert_eq!(canale.leggi_esatti(10).await.unwrap(), vec![1u8; 10]);
    assert_eq!(flusso::byte_confermati(&ricevi(&mut tel).await).unwrap(), Some(3000));
    assert_eq!(canale.leggi().await.unwrap().len(), 2990);
    silenzio(&mut tel).await;
    assert_eq!(canale.leggi().await.unwrap(), vec![2u8; 500]);
    assert_eq!(flusso::byte_confermati(&ricevi(&mut tel).await).unwrap(), Some(500));
}

#[tokio::test]
async fn senza_delayed_ack_come_prima() {
    let (adb, mut tel) = collega(false, 1024 * 1024);
    // La conferma alla lettura senza delayed ack non si usa.
    let opzioni = OpzioniCanale { finestra: Some(8192), conferma_alla_lettura: true };
    let apertura = tokio::spawn({
        let adb = adb.clone();
        async move { adb.apri_con("exec:ls", opzioni).await.unwrap() }
    });
    let open = ricevi(&mut tel).await;
    assert_eq!(open.arg1, 0, "senza delayed ack arg1 dev'essere 0");
    let locale = open.arg0;
    manda(&mut tel, flusso::conferma(9, locale, None)).await;
    let mut canale = apertura.await.unwrap();
    let scrittura = tokio::spawn(async move {
        canale.scrivi(&vec![0u8; 2 * BLOCCO + 10]).await.unwrap();
        canale
    });
    // Un WRTE alla volta, ognuno dopo l'OKAY del precedente.
    for lunghezza in [BLOCCO, BLOCCO, 10] {
        let m = ricevi(&mut tel).await;
        assert_eq!((m.comando, m.dati.len()), (WRTE, lunghezza));
        silenzio(&mut tel).await;
        manda(&mut tel, flusso::conferma(9, locale, None)).await;
    }
    let mut canale = scrittura.await.unwrap();
    // In arrivo: OKAY vuoto subito, anche senza lettura.
    manda(&mut tel, Messaggio::new(WRTE, 9, locale, b"ciao".to_vec())).await;
    let okay = ricevi(&mut tel).await;
    assert_eq!((okay.comando, okay.arg0, okay.arg1), (OKAY, locale, 9));
    assert!(okay.dati.is_empty());
    assert_eq!(canale.leggi().await.unwrap(), b"ciao");
    // Chiusura dal telefono.
    manda(&mut tel, Messaggio::new(CLSE, 9, locale, Vec::new())).await;
    assert!(canale.leggi().await.is_none());
}

#[tokio::test]
async fn apertura_rifiutata() {
    let (adb, mut tel) = collega(true, 1024);
    let apertura = tokio::spawn({
        let adb = adb.clone();
        async move { adb.apri("inesistente:").await.map(|_| ()) }
    });
    let open = ricevi(&mut tel).await;
    manda(&mut tel, Messaggio::new(CLSE, 0, open.arg0, Vec::new())).await;
    assert!(apertura.await.unwrap().is_err());
    assert!(adb.registro.lock().unwrap().aperti.is_empty());
}

#[test]
fn trasporto_predefinito_e_limiti() {
    // Delayed ack spento finché non funziona sul telefono; blocchi da 64 KiB.
    let t = Trasporto::default();
    assert!(!t.delayed_ack);
    assert_eq!((t.max_payload, t.finestra), (64 * 1024, 256 * 1024));
    let strano = Trasporto { delayed_ack: false, max_payload: 10, finestra: 0 }.normalizzato();
    assert_eq!((strano.max_payload, strano.finestra), (4096, 1));
    let grande = Trasporto { delayed_ack: true, max_payload: u32::MAX, finestra: u32::MAX }.normalizzato();
    assert_eq!((grande.max_payload, grande.finestra), (MAX_DATI, i32::MAX as u32));
}
