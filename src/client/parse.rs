use crate::Client;
use nom_openmetrics::{
    parser::{openmetrics, prometheus},
    Family, MetricDescriptor, Sample,
};
use nu_protocol::{record, LabeledError, Record, Span, Value};

#[derive(Default)]
pub enum ParseFormat {
    #[default]
    Prometheus,
    Openmetrics,
}

pub struct Parse<'a> {
    input: &'a Value,
    format: ParseFormat,
}

impl<'a> Parse<'a> {
    pub fn new(input: &'a Value) -> Self {
        Self {
            input,
            format: Default::default(),
        }
    }

    pub fn run(self) -> Result<Value, LabeledError> {
        let Self { input, format } = self;

        let (_, families) = match format {
            ParseFormat::Prometheus => prometheus(input.as_str()?).unwrap(),
            ParseFormat::Openmetrics => openmetrics(input.as_str()?).unwrap(),
        };

        let families = families
            .iter()
            .map(|family| family_to_value(family))
            .collect();

        Ok(Value::list(families, Span::unknown()))
    }

    pub fn set_format(&mut self, format: ParseFormat) {
        self.format = format;
    }
}

impl<'a> Client for Parse<'a> {}

fn family_to_value(family: &Family) -> Value {
    let descriptors = family
        .descriptors
        .iter()
        .map(|descriptor| descriptor_to_value(descriptor))
        .collect();

    let samples = family
        .samples
        .iter()
        .map(|sample| sample_to_value(sample))
        .collect();

    let record = record! {
        "descriptors" => Value::list(descriptors, Span::unknown()),
        "samples" => Value::list(samples, Span::unknown()),
    };

    Value::record(record, Span::unknown())
}

fn descriptor_to_value(descriptor: &MetricDescriptor) -> Value {
    let record = match descriptor {
        MetricDescriptor::Type { metric, r#type } => record! {
            "descriptor" => Value::string("type", Span::unknown()),
            "metric" => Value::string(*metric, Span::unknown()),
            "type" => Value::string(r#type.to_string(), Span::unknown())
        },
        MetricDescriptor::Help { metric, help } => record! {
            "descriptor" => Value::string("help", Span::unknown()),
            "metric" => Value::string(*metric, Span::unknown()),
            "help" => Value::string(help, Span::unknown())
        },
        MetricDescriptor::Unit { metric, unit } => record! {
            "descriptor" => Value::string("unit", Span::unknown()),
            "metric" => Value::string(*metric, Span::unknown()),
            "unit" => Value::string(*unit, Span::unknown())
        },
    };

    Value::record(record, Span::unknown())
}

fn sample_to_value(sample: &Sample) -> Value {
    let mut record = Record::new();

    record.insert("name", Value::string(sample.name(), Span::unknown()));
    record.insert("value", Value::float(sample.number(), Span::unknown()));

    Value::record(record, Span::unknown())
}
