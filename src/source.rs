use nu_json::{Map, Value};
use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::Span;
use prometheus_http_query::Client;
use reqwest::{Certificate, Identity};
use std::path::Path;

pub struct Source {
    pub name: Option<String>,
    pub url: String,
    pub identity: Option<Identity>,
    pub cacert: Option<Certificate>,
    pub span: Span,
}

impl Source {
    pub fn from_call(call: &EvaluatedCall) -> Result<Self, LabeledError> {
        let source = call.get_flag_value("source");
        let url = call.get_flag_value("url");

        if let Some(url) = url {
            if let Some(source) = source {
                return Err(LabeledError {
                    label: "Flag conflict".into(),
                    msg: "Supply only --source or --url, not both".into(),
                    span: Some(source.span()),
                });
            }

            Source::from_call_url(call, url)
        } else if let Some(source) = source {
            Source::from_config(source)
        } else {
            Err(LabeledError {
                label: "Missing flag".into(),
                msg: "Missing --source or --url flag".into(),
                span: Some(call.head),
            })
        }
    }

    fn from_call_url(
        call: &EvaluatedCall,
        url_value: nu_protocol::Value,
    ) -> Result<Self, LabeledError> {
        let nu_protocol::Value::String { val: ref url, .. } = url_value else {
            return Err(LabeledError { label: "Invalid argument type".into(), msg: "Expected --url to be a String".into(), span: Some(url_value.span()) });
        };

        let cert = call.get_flag_value("cert");
        let key = call.get_flag_value("key");

        let identity = make_identity(cert, key)?;

        let cacert = call
            .get_flag_value("cacert")
            .map(|cacert| certificate(cacert))
            .transpose()?;

        Ok(Self {
            name: None,
            url: url.clone(),
            identity,
            cacert,
            span: url_value.span(),
        })
    }

    fn from_config(source: nu_protocol::Value) -> Result<Self, LabeledError> {
        let name = source.as_string().unwrap();

        let home = std::env::var("HOME").map_err(|e| LabeledError {
            label: "Unable to find source".into(),
            msg: format!("Could not find HOME env var: {e:?}"),
            span: Some(source.span()),
        })?;

        let config_file = Path::new(&home).join(".config/nu_plugin_prometheus.hjson");

        let config: Map<String, Value> = std::fs::read(&config_file)
            .map(|config| nu_json::from_slice(&config[..]))
            .map_err(|e| LabeledError {
                label: "Unable to find source".into(),
                msg: format!("Unable to read configuration file at {config_file:?}: {e:?}"),
                span: Some(source.span()),
            })?
            .map_err(|e| LabeledError {
                label: "Unable to find source".into(),
                msg: format!("Unable to deserialize configuration file at {config_file:?}: {e:?}"),
                span: Some(source.span()),
            })?;

        let Some(sources) = config.get("sources") else {
            return Err(LabeledError {
                label: "Unable to find source".into(),
                msg: format!("Configuration file at {config_file:?} is missing a \"sources\" entry"),
                span: Some(source.span()),
            });
        };

        let Value::Object(sources) = sources else {
            return Err(LabeledError {
                label: "Unable to find source".into(),
                msg: format!("\"sources\" entry in configuration file at {config_file:?} is not a object"),
                span: Some(source.span()),
            });
        };

        let Some(chosen) = sources.get(&name) else {
            return Err(LabeledError {
                label: "Unable to find source".into(),
                msg: format!("source {name:?} in configuration file at {config_file:?} does not exist"),
                span: Some(source.span()),
            });
        };

        let Value::Object(chosen) = chosen else {
            return Err(LabeledError {
                label: "Unable to find source".into(),
                msg: format!("source {name:?} in configuration file at {config_file:?} is not an object"),
                span: Some(source.span()),
            });
        };

        let url = get_field(chosen, "url");

        let Some(url) = url else {
                return Err(LabeledError {
                    label: "Unable to find source url".into(),
                    msg: format!("source {name:?} in configuration file at {config_file:?} is missing its \"url\" field"),
                    span: Some(source.span()),
                });
            };

        let cert =
            get_field(chosen, "cert").map(|cert| nu_protocol::Value::string(cert, Span::unknown()));
        let key =
            get_field(chosen, "key").map(|key| nu_protocol::Value::string(key, Span::unknown()));

        let identity = make_identity(cert, key)?;

        let cacert = get_field(chosen, "cacert")
            .map(|cacert| nu_protocol::Value::string(cacert, Span::unknown()))
            .map(|cacert| certificate(cacert))
            .transpose()?;

        let chosen = Self {
            name: Some(name),
            url,
            identity,
            cacert,
            span: source.span(),
        };

        Ok(chosen)
    }
}

impl TryFrom<&EvaluatedCall> for Source {
    type Error = LabeledError;

    fn try_from(call: &EvaluatedCall) -> Result<Self, Self::Error> {
        Source::from_call(call)
    }
}

impl TryFrom<Source> for Client {
    type Error = LabeledError;

    fn try_from(source: Source) -> Result<Self, Self::Error> {
        let client_builder = reqwest::ClientBuilder::new();

        let client_builder = if let Some(identity) = source.identity {
            client_builder.identity(identity)
        } else {
            client_builder
        };

        let client_builder = if let Some(cacert) = source.cacert {
            client_builder.add_root_certificate(cacert)
        } else {
            client_builder
        };

        let client = client_builder.build().map_err(|e| LabeledError {
            label: "Unable to build prometheus client".to_string(),
            msg: e.to_string(),
            span: None,
        })?;

        let client = Client::from(client, &source.url).map_err(|e| LabeledError {
            label: "Unable to build prometheus client".to_string(),
            msg: e.to_string(),
            span: Some(source.span),
        })?;

        Ok(client)
    }
}

fn certificate(cacert: nu_protocol::Value) -> Result<Certificate, LabeledError> {
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

fn get_field(chosen: &Map<String, Value>, field: &str) -> Option<String> {
    let url = chosen
        .get(field)
        .and_then(|url| url.as_str())
        .map(|url| url.to_string());
    url
}

fn make_identity(
    cert: Option<nu_protocol::Value>,
    key: Option<nu_protocol::Value>,
) -> Result<Option<Identity>, LabeledError> {
    match (cert, key) {
        (None, None) => Ok(None),
        (Some(cert), Some(key)) => Ok(Some(identity(cert, key)?)),
        (None, Some(key)) => {
            return Err(missing_flag("client key", "--cert", key.span()));
        }
        (Some(cert), None) => {
            return Err(missing_flag("client cert", "--key", cert.span()));
        }
    }
}

fn identity(cert: nu_protocol::Value, key: nu_protocol::Value) -> Result<Identity, LabeledError> {
    let cert_pem = read_pem(&cert, "Client certificate")?;
    let key_pem = read_pem(&key, "Client key")?;

    Identity::from_pkcs8_pem(&cert_pem, &key_pem).map_err(|e| LabeledError {
        label: "Client certificate or key are not in PEM format".to_string(),
        msg: e.to_string(),
        span: None,
    })
}

fn missing_flag(have: &str, missing: &str, span: Span) -> LabeledError {
    LabeledError {
        label: "Missing TLS flag".into(),
        msg: format!("Have {have}, missing {missing}"),
        span: Some(span),
    }
}

fn read_pem(value: &nu_protocol::Value, kind: &str) -> Result<Vec<u8>, LabeledError> {
    let path = value.as_path()?;
    let pem = std::fs::read(path).map_err(|e| LabeledError {
        label: format!("{kind} {:?} does not exist", value.as_string().unwrap()),
        msg: e.to_string(),
        span: Some(value.span()),
    })?;

    Ok(pem)
}
