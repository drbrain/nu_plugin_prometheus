use nu_protocol::{record, LabeledError, Record, Span, Value};
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
                .query(self.query.clone().into_string().unwrap())
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
        use prometheus_http_query::Error;

        match error {
            Error::Client(e) => LabeledError::new("Prometheus client error")
                .with_label(e.to_string(), self.query.span()),
            Error::EmptySeriesSelector => {
                LabeledError::new("Empty series selector").with_label("", self.query.span())
            }
            // This error should be impossible to reach because it should occur when building the client
            Error::ParseUrl(e) => LabeledError::new("Invalid URL").with_help(e.to_string()),
            Error::Prometheus(e) => {
                LabeledError::new("Prometheus error").with_label(e.to_string(), self.query.span())
            }
            e => LabeledError::new("Other error").with_label(e.to_string(), self.query.span()),
        }
    }
}

fn matrix_to_value(matrix: &[RangeVector]) -> Value {
    let records = matrix
        .iter()
        .map(|rv| {
            let metric = rv.metric();
            let values = rv.samples().iter().map(scalar_to_value).collect();

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
        .map_err(|e| LabeledError::new("Tokio runtime build error").with_help(e.to_string()))
}

#[cfg(test)]
mod test {
    use prometheus_http_query::response::{InstantVector, RangeVector, Sample};

    #[test]
    fn matrix_to_value() {
        let data = r#"[
         {
            "metric" : {
               "__name__" : "up",
               "job" : "prometheus",
               "instance" : "localhost:9090"
            },
            "values" : [
               [ 1435781430.781, "1" ],
               [ 1435781445.781, "1" ],
               [ 1435781460.781, "1" ]
            ]
         },
         {
            "metric" : {
               "__name__" : "up",
               "job" : "node",
               "instance" : "localhost:9091"
            },
            "values" : [
               [ 1435781430.781, "0" ],
               [ 1435781445.781, "0" ],
               [ 1435781460.781, "1" ]
            ]
         }
      ]"#
        .as_bytes();
        let matrix: Vec<RangeVector> = serde_json::from_slice(data).unwrap();

        let result = super::matrix_to_value(&matrix);

        let record = result
            .clone()
            .into_list()
            .unwrap()
            .first()
            .unwrap()
            .clone()
            .into_record()
            .unwrap();

        assert_eq!("up", record.get("name").unwrap().as_str().unwrap());

        let labels = record.get("labels").unwrap().as_record().unwrap();

        assert_eq!("prometheus", labels.get("job").unwrap().as_str().unwrap());

        let values = record.get("values").unwrap().as_list().unwrap();

        assert_eq!(3, values.len());
    }

    #[test]
    fn scalar_to_value() {
        let data = r#"[1716956024.754,"1"]"#.as_bytes();
        let scalar: Sample = serde_json::from_slice(data).unwrap();

        let result = super::scalar_to_value(&scalar).into_record().unwrap();

        assert_eq!(1.0, result.get("value").unwrap().as_f64().unwrap());
        assert_eq!(
            1716956024,
            result.get("timestamp").unwrap().as_f64().unwrap() as u64
        );
    }

    #[test]
    fn vector_to_value() {
        let data = r#"[{"metric":{"__name__":"up","instance":"target.example","job":"job name"},"value":[1716956024.754,"1"]}]"#.as_bytes();
        let vector: Vec<InstantVector> = serde_json::from_slice(data).unwrap();

        let result = super::vector_to_value(&vector).into_list().unwrap();

        let record = result.first().unwrap().as_record().unwrap();

        assert_eq!("up", record.get("name").unwrap().as_str().unwrap());

        let labels = record.get("labels").unwrap().as_record().unwrap();

        assert_eq!("job name", labels.get("job").unwrap().as_str().unwrap());

        let value = record
            .get("value")
            .unwrap()
            .as_record()
            .unwrap()
            .get("value")
            .unwrap()
            .as_f64()
            .unwrap();

        assert_eq!(1.0, value);
    }
}
