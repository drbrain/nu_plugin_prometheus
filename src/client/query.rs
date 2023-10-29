use nu_plugin::LabeledError;
use nu_protocol::{record, Record, Span, Value};
use prometheus_http_query::{
    response::{Data, InstantVector, RangeVector, Sample},
    Client,
};

pub struct Query<'a> {
    client: Client,
    query: &'a Value,
}

impl<'a> Query<'a> {
    pub fn new(client: Client, query: &'a Value) -> Self {
        Self { client, query }
    }

    pub fn run(&self) -> Result<Value, LabeledError> {
        runtime()?.block_on(async {
            let response = self
                .client
                .query(self.query.as_string().unwrap())
                .get()
                .await
                .map_err(|error| self.into_labeled_error(error))?;

            let value = match response.data() {
                Data::Vector(v) => vector_to_value(v),
                Data::Matrix(m) => matrix_to_value(m),
                Data::Scalar(s) => scalar_to_value(s),
            };

            Ok(value)
        })
    }

    fn into_labeled_error(&self, error: prometheus_http_query::Error) -> LabeledError {
        match error {
            prometheus_http_query::Error::Client(e) => LabeledError {
                label: "Prometheus client error".to_string(),
                msg: e.to_string(),
                span: Some(self.query.span()),
            },
            prometheus_http_query::Error::ApiError(e) => LabeledError {
                label: "Prometheus query error".to_string(),
                msg: e.to_string(),
                span: Some(self.query.span()),
            },
            prometheus_http_query::Error::EmptySeriesSelector => LabeledError {
                label: "Empty series selector".to_string(),
                msg: "".to_string(),
                span: Some(self.query.span()),
            },
            // This error should be impossible to reach because it should occur when building the client
            prometheus_http_query::Error::UrlParse(e) => LabeledError {
                label: "Invalid URL".to_string(),
                msg: e.to_string(),
                span: None,
            },
            prometheus_http_query::Error::ResponseParse(e) => LabeledError {
                label: "Prometheus response parse error".to_string(),
                msg: e.to_string(),
                span: None,
            },
            prometheus_http_query::Error::MissingField(e) => LabeledError {
                label: "Prometheus missing response field error".to_string(),
                msg: e.to_string(),
                span: None,
            },
            e => LabeledError {
                label: "Other prometheus query error".to_string(),
                msg: e.to_string(),
                span: Some(self.query.span()),
            },
        }
    }
}

fn matrix_to_value(matrix: &[RangeVector]) -> Value {
    let records = matrix
        .iter()
        .map(|rv| {
            let metric = rv.metric();
            let values = rv
                .samples()
                .iter()
                .map(|sample| scalar_to_value(sample))
                .collect();

            let name = metric
                .get("__name__")
                .cloned()
                .unwrap_or("[UNKNOWN]".to_string());

            let mut labels = Record::new();
            for (name, label) in metric {
                if name == "__name__" {
                    continue;
                }
                labels.push(name, Value::string(label, Span::unknown()));
            }

            Value::record(
                record! {
                    "name" => Value::string(name, Span::unknown()),
                    "labels" => Value::record(labels, Span::unknown()),
                    "values" => Value::list(values, Span::unknown()),
                },
                Span::unknown(),
            )
        })
        .collect();

    Value::list(records, Span::unknown())
}

fn scalar_to_value(scalar: &Sample) -> Value {
    Value::record(
        record! {
            "value" => Value::float(scalar.value(), Span::unknown()),
            "timestamp" => Value::float(scalar.timestamp(), Span::unknown())
        },
        Span::unknown(),
    )
}

fn vector_to_value(vector: &[InstantVector]) -> Value {
    let records = vector
        .iter()
        .map(|iv| {
            let metric = iv.metric();
            let value = scalar_to_value(iv.sample());

            let name = metric
                .get("__name__")
                .cloned()
                .unwrap_or("[UNKNOWN]".to_string());

            let mut labels = Record::new();
            for (name, label) in metric {
                if name == "__name__" {
                    continue;
                }
                labels.push(name, Value::string(label, Span::unknown()));
            }

            Value::record(
                record! {
                    "name" => Value::string(name, Span::unknown()),
                    "labels" => Value::record(labels, Span::unknown()),
                    "value" => value,
                },
                Span::unknown(),
            )
        })
        .collect();

    Value::list(records, Span::unknown())
}

fn runtime() -> Result<tokio::runtime::Runtime, LabeledError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| LabeledError {
            label: "Tokio runtime build error".into(),
            msg: e.to_string(),
            span: None,
        })
}
