use nu_plugin::{EngineInterface, EvaluatedCall};
use nu_protocol::{LabeledError, Record, Span, Value};
use prometheus_http_query::Client;
use reqwest::{Certificate, Identity};

#[derive(Clone)]
pub struct Source {
    pub name: Option<String>,
    pub url: String,
    pub identity: Option<Identity>,
    pub cacert: Option<Certificate>,
    pub span: Span,
}

impl Source {
    pub fn list(engine: &EngineInterface) -> Result<Vec<Source>, LabeledError> {
        let config = engine.get_plugin_config().map_err(|e| {
            LabeledError::new("Plugin configuration not found").with_help(e.to_string())
        })?;

        let Some(config) = config else {
            return Err(LabeledError::new("Plugin configuration not found"));
        };

        let Some(sources) = config.get_data_by_key("sources") else {
            return Err(LabeledError::new("Invalid plugin configuration").with_help(r#"Missing "sources""#));
        };

        let sources = sources.as_record().map_err(|_| {
            LabeledError::new("Invalid plugin configuration")
                .with_label("must be a record", sources.span())
        })?;

        let mut result = vec![];
        for (name, source) in sources.iter() {
            let span = source.span();

            let source = source.as_record().map_err(|_| {
                LabeledError::new("Invalid plugin configuration")
                    .with_label(format!("Source {name:?} is not a record"), span)
            })?;

            let url = value_from_source(source, name, span, "url")?;

            let url = url.clone().into_string().map_err(|_| {
                LabeledError::new("Invalid plugin configuration").with_label(
                    format!(r#"Source {name:?} field "url" is not a string"#),
                    url.span(),
                )
            })?;

            let cert = value_from_source(source, name, span, "cert").ok();
            let key = value_from_source(source, name, span, "key").ok();

            let identity = if let (Some(cert), Some(key)) = (cert, key) {
                Some(identity(cert.clone(), key.clone())?)
            } else {
                None
            };

            let cacert = value_from_source(source, name, span, "cacert")
                .ok()
                .map(|cacert| certificate(cacert.clone()))
                .transpose()?;

            let source = Source {
                name: Some(name.clone()),
                url,
                cacert,
                identity,
                span,
            };

            result.push(source);
        }

        Ok(result)
    }

    pub fn from(call: &EvaluatedCall, engine: &EngineInterface) -> Result<Source, LabeledError> {
        let source = call.get_flag_value("source");

        if let Some(url) = call.get_flag_value("url") {
            if let Some(source) = source {
                return Err(LabeledError::new("Argument error")
                    .with_label("Supply only --source or --url, not both", source.span()));
            }

            Source::from_call_url(call, url)
        } else {
            let sources = Source::list(engine)?;

            if let Some(source) = source {
                let source_name = source.clone().into_string()?;

                sources
                    .iter()
                    .find(|source| source.name == Some(source_name.clone()))
                    .cloned()
                    .ok_or_else(|| {
                        LabeledError::new("Matching source not found")
                            .with_label("this source is not configured", source.span())
                    })
            } else {
                Err(
                    LabeledError::new("Prometheus server not specified").with_help(
                        "You must configure an unnamed default source or provide --source or --url",
                    ),
                )
            }
        }
    }

    fn from_call_url(call: &EvaluatedCall, url_value: Value) -> Result<Self, LabeledError> {
        let Value::String { val: ref url, .. } = url_value else {
            return Err(LabeledError::new("Invalid argument type")
                .with_label("Expected --url to be a String", url_value.span()));
        };

        let cert = call.get_flag_value("cert");
        let key = call.get_flag_value("key");

        let identity = make_identity(cert, key)?;

        let cacert = call.get_flag_value("cacert").map(certificate).transpose()?;

        Ok(Self {
            name: None,
            url: url.clone(),
            identity,
            cacert,
            span: url_value.span(),
        })
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

        let client = client_builder.build().map_err(|e| {
            LabeledError::new("Unable to build prometheus client").with_help(e.to_string())
        })?;

        let client = Client::from(client, &source.url).map_err(|e| {
            LabeledError::new("Unable to build prometheus client").with_help(e.to_string())
        })?;

        Ok(client)
    }
}

fn certificate(cacert: Value) -> Result<Certificate, LabeledError> {
    let cacert_pem = read_pem(&cacert, "CA certificate")?;

    let cacert = Certificate::from_pem(&cacert_pem).map_err(|e| {
        LabeledError::new(e.to_string()).with_label(
            format!(
                "CA certificate {} is not in PEM format",
                cacert.clone().into_string().unwrap()
            ),
            cacert.span(),
        )
    })?;

    Ok(cacert)
}

fn value_from_source<'a>(
    source: &'a Record,
    source_name: &str,
    source_span: Span,
    name: &str,
) -> Result<&'a Value, LabeledError> {
    source.get(name).ok_or_else(|| {
        LabeledError::new("Invalid plugin configuration").with_label(
            format!("Source {source_name:?} missing {name} field"),
            source_span,
        )
    })
}

fn identity(cert: Value, key: Value) -> Result<Identity, LabeledError> {
    let cert_pem = read_pem(&cert, "Client certificate")?;
    let key_pem = read_pem(&key, "Client key")?;

    Identity::from_pkcs8_pem(&cert_pem, &key_pem).map_err(|e| {
        LabeledError::new("Client certificate or key are not in PEM format")
            .with_help(e.to_string())
    })
}

fn make_identity(
    cert: Option<Value>,
    key: Option<Value>,
) -> Result<Option<Identity>, LabeledError> {
    match (cert, key) {
        (None, None) => Ok(None),
        (Some(cert), Some(key)) => Ok(Some(identity(cert, key)?)),
        (None, Some(key)) => Err(missing_entry("client key", "cert", key.span())),
        (Some(cert), None) => Err(missing_entry("client cert", "key", cert.span())),
    }
}

fn missing_entry(have: &str, missing: &str, span: Span) -> LabeledError {
    LabeledError::new("Missing TLS item")
        .with_label(format!("Have {have}, missing {missing}"), span)
}

fn read_pem(value: &Value, kind: &str) -> Result<Vec<u8>, LabeledError> {
    let path = value.to_path()?;
    let pem = std::fs::read(path).map_err(|e| {
        LabeledError::new(format!(
            "{kind} {:?} does not exist",
            value.clone().into_string().unwrap()
        ))
        .with_label(e.to_string(), value.span())
    })?;

    Ok(pem)
}

#[cfg(test)]
mod test {
    use nu_protocol::{Span, Value};
    use prometheus_http_query::Client;
    use rstest::rstest;
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn cert_path() -> PathBuf {
        Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test/fixtures/cert.pem")
    }

    fn cert_value() -> Value {
        Value::string(cert_path().to_string_lossy(), Span::unknown())
    }

    fn key_path() -> PathBuf {
        Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test/fixtures/key.pem")
    }

    fn key_value() -> Value {
        Value::string(key_path().to_string_lossy(), Span::unknown())
    }

    #[test]
    fn certificate() {
        assert!(super::certificate(cert_value()).is_ok());
    }

    #[test]
    fn identity() {
        assert!(super::identity(cert_value(), key_value()).is_ok());
    }

    #[rstest]
    #[case(None, None, Ok(None))]
    #[case(None, Some(key_value()), Err(("client key", "cert")))]
    #[case(Some(cert_value()), None, Err(("client cert", "key")))]
    #[case(Some(cert_value()), Some(key_value()), Ok(Some(())))]
    fn make_identity(
        #[case] cert: Option<Value>,
        #[case] key: Option<Value>,
        #[case] expected: Result<Option<()>, (&str, &str)>,
    ) {
        let result = super::make_identity(cert, key);

        match expected {
            Ok(Some(_)) => {
                assert!(matches!(result, Ok(Some(_))));
            }
            Ok(None) => {
                assert!(matches!(result, Ok(None)));
            }
            Err((have, missing)) => {
                let expected = super::missing_entry(have, missing, Span::unknown());

                assert_eq!(expected, result.unwrap_err());
            }
        }
    }

    #[test]
    fn missing_entry() {
        let err = super::missing_entry("first", "second", Span::test_data());

        assert_eq!("Missing TLS item", err.msg);

        let label = err.labels.first().unwrap();
        assert_eq!("Have first, missing second", label.text);
    }

    #[test]
    fn read_pem() {
        let key_path = key_path();
        let expected = fs::read(&key_path).unwrap();

        let pem = super::read_pem(
            &Value::string(key_path.to_string_lossy(), Span::unknown()),
            "key",
        )
        .unwrap();

        assert_eq!(expected, pem);
    }

    #[test]
    fn source_from_client() {
        let url = "https://prometheus.example/";

        let source = super::Source {
            name: Some("test".into()),
            url: url.into(),
            identity: None,
            cacert: None,
            span: Span::unknown(),
        };

        let result = TryInto::<Client>::try_into(source);

        let Ok(client) = result else {
            unreachable!("Client not created");
        };

        assert_eq!(url, client.base_url().as_str());
    }
}
