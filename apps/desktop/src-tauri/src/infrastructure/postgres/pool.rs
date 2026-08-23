use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};

use crate::domain::connection::{ConnectionProfile, SslMode};
use crate::error::AppError;

pub fn build_pool(profile: &ConnectionProfile) -> Result<Pool, AppError> {
    let mut cfg = Config::new();
    cfg.host = Some(profile.host.clone());
    cfg.port = Some(profile.port);
    cfg.dbname = Some(profile.database.clone());
    cfg.user = Some(profile.user.clone());
    cfg.password = Some(profile.password.clone());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = match profile.ssl_mode {
        SslMode::Disable | SslMode::Prefer => cfg
            .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
            .map_err(|e| AppError::new(format!("Failed to create connection pool: {e}")))?,
        SslMode::Require | SslMode::VerifyCa | SslMode::VerifyFull => {
            let tls = build_tls_connector(profile.ssl_mode)?;
            cfg.create_pool(Some(Runtime::Tokio1), tls)
                .map_err(|e| AppError::new(format!("Failed to create connection pool: {e}")))?
        }
    };

    Ok(pool)
}

fn build_tls_connector(
    ssl_mode: SslMode,
) -> Result<tokio_postgres_rustls::MakeRustlsConnect, AppError> {
    let mut roots = rustls::RootCertStore::empty();
    let native = rustls_native_certs::load_native_certs();
    for err in &native.errors {
        log::warn!("Failed to load a native root certificate: {err}");
    }
    for cert in native.certs {
        let _ = roots.add(cert);
    }

    let builder = rustls::ClientConfig::builder().with_root_certificates(roots);

    let tls_config = if ssl_mode == SslMode::VerifyCa || ssl_mode == SslMode::VerifyFull {
        builder.with_no_client_auth()
    } else {
        rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(std::sync::Arc::new(NoCertVerification))
            .with_no_client_auth()
    };

    Ok(tokio_postgres_rustls::MakeRustlsConnect::new(tls_config))
}

#[derive(Debug)]
struct NoCertVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
