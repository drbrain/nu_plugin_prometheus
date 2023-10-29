mod query;

use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::Value;
use prometheus_http_query::Client;
pub use query::Query;
use reqwest::{Certificate, Identity};

pub fn build(call: &EvaluatedCall, source: Value) -> Result<Client, LabeledError> {
    let cert = call.get_flag_value("cert");
    let key = call.get_flag_value("key");

    let client_builder = match (cert, key) {
        (None, None) => Ok(reqwest::Client::builder()),
        (None, Some(key)) => Err(missing_flag(key, "Client key")),
        (Some(cert), None) => Err(missing_flag(cert, "Client cert")),
        (Some(cert), Some(key)) => {
            let builder = reqwest::ClientBuilder::new().identity(identity(cert, key)?);

            Ok(builder)
        }
    }?;

    let client_builder = if let Some(cacert) = call.get_flag_value("cacert") {
        client_builder.add_root_certificate(certificate(cacert)?)
    } else {
        client_builder
    };

    let client = client_builder.build().map_err(|e| LabeledError {
        label: "Unable to build prometheus client".to_string(),
        msg: e.to_string(),
        span: None,
    })?;

    let client = Client::from(client, &source.as_string().unwrap()).map_err(|e| LabeledError {
        label: "Unable to build prometheus client".to_string(),
        msg: e.to_string(),
        span: Some(source.span()),
    })?;

    Ok(client)
}

fn certificate(cacert: Value) -> Result<Certificate, LabeledError> {
    let cacert_pem = read_pem(&cacert, "CA certificate")?;

    let cacert = Certificate::from_pem(&cacert_pem).map_err(|e| LabeledError {
        label: format!(
            "CA certificate {} is not in PEM format",
            cacert.as_string().unwrap()
        ),
        msg: e.to_string(),
        span: Some(cacert.span()),
    })?;

    Ok(cacert)
}

fn identity(cert: Value, key: Value) -> Result<Identity, LabeledError> {
    let cert_pem = read_pem(&cert, "Client certificate")?;
    let key_pem = read_pem(&key, "Client key")?;

    Identity::from_pkcs8_pem(&cert_pem, &key_pem).map_err(|e| LabeledError {
        label: "Client certificate or key are not in PEM format".to_string(),
        msg: e.to_string(),
        span: None,
    })
}

fn missing_flag(value: Value, kind: &str) -> LabeledError {
    LabeledError {
        label: format!("{kind} provided without certificate"),
        msg: format!("{}", value.as_string().unwrap()),
        span: Some(value.span()),
    }
}

fn read_pem(value: &Value, kind: &str) -> Result<Vec<u8>, LabeledError> {
    let path = value.as_path()?;
    let pem = std::fs::read(path).map_err(|e| LabeledError {
        label: format!("{kind} {:?} does not exist", value.as_string().unwrap()),
        msg: e.to_string(),
        span: Some(value.span()),
    })?;

    Ok(pem)
}
