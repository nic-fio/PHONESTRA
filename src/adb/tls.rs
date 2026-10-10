// Copyright (c) 2026 Nicola Fiorillo
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Cifratura del Debug wireless (Android 11+): dopo `STLS` si passa a TLS e il
//! telefono riconosce il PC dalla chiave pubblica del certificato client, che
//! dev'essere quella autorizzata con «Consenti sempre».

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use rcgen::{CertificateParams, KeyPair, PKCS_RSA_SHA256};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};

use crate::t;

/// Il certificato del telefono è autofirmato: l'identità è garantita
/// dall'abbinamento/autorizzazione, non da un'autorità. Si controlla solo che
/// la firma dell'handshake sia valida.
#[derive(Debug)]
struct CertificatoTelefono(Arc<CryptoProvider>);

impl ServerCertVerifier for CertificatoTelefono {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        messaggio: &[u8],
        certificato: &CertificateDer<'_>,
        firma: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(messaggio, certificato, firma, &self.0.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        messaggio: &[u8],
        certificato: &CertificateDer<'_>,
        firma: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(messaggio, certificato, firma, &self.0.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

pub fn configurazione_client(chiave: &Path) -> Result<Arc<ClientConfig>> {
    let pem = std::fs::read_to_string(chiave).with_context(|| t!("chiave {} illeggibile", chiave.display()))?;
    let coppia = KeyPair::from_pkcs8_pem_and_sign_algo(&pem, &PKCS_RSA_SHA256)?;
    let certificato = CertificateParams::default().self_signed(&coppia)?;
    let privata = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(coppia.serialize_der()));
    let fornitore = Arc::new(rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(fornitore.clone())
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(CertificatoTelefono(fornitore)))
        .with_client_auth_cert(vec![certificato.der().clone()], privata)?;
    Ok(Arc::new(config))
}
