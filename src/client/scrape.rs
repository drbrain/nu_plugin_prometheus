use nom_openmetrics::{Family, Sample};
use nu_protocol::{LabeledError, Record, Span, Value};

use crate::client::Client;

pub struct Scrape {
    target: String,
}

impl Scrape {
    pub fn new(target: String) -> Self {
        Self { target }
    }

    pub fn run(self) -> Result<Value, LabeledError> {
        let Self { ref target, .. } = self;

        self.runtime()?.block_on(async {
            let body = reqwest::get(target).await.unwrap().bytes().await.unwrap();

            let body = String::from_utf8(body.to_vec()).unwrap();

            let (_, families) = nom_openmetrics::parser::prometheus(&body).unwrap();

            let families = families
                .iter()
                .map(|family| family_to_value(family))
                .collect();

            Ok(Value::list(families, Span::unknown()))
        })
    }
}

impl Client for Scrape {}

fn family_to_value(family: &Family) -> Value {
    let mut record = Record::new();

    let samples = family
        .samples
        .iter()
        .map(|sample| sample_to_value(sample))
        .collect();

    record.insert("samples", Value::list(samples, Span::unknown()));

    Value::record(record, Span::unknown())
}

fn sample_to_value(sample: &Sample) -> Value {
    let mut record = Record::new();

    record.insert("name", Value::string(sample.name(), Span::unknown()));
    record.insert("value", Value::float(sample.number(), Span::unknown()));

    Value::record(record, Span::unknown())
}
