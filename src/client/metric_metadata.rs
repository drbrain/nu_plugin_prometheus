use nu_protocol::{record, LabeledError, Span, Value};
use prometheus_http_query::MetricMetadataQueryBuilder;

use super::Client;

pub struct MetricMetadata {
    builder: MetricMetadataQueryBuilder,
    span: Span,
}

impl MetricMetadata {
    pub fn new(builder: MetricMetadataQueryBuilder, span: Span) -> Self {
        Self { builder, span }
    }

    pub fn run(self) -> Result<Value, LabeledError> {
        let Self { ref builder, span } = self;

        self.runtime()?.block_on(async {
            let metric_metadata = builder
                .clone()
                .get()
                .await
                .map_err(|error| self.labeled_error(error, span))?;

            let record = metric_metadata
                .iter()
                .fold(record!(), |mut record, (metric, metadata)| {
                    let metadata: Vec<_> =
                    metadata.iter().map(|item| {
                        let item = record! {
                            "type" => Value::string(item.metric_type().to_string(), Span::unknown()),
                            "help" => Value::string(item.help().to_string(), Span::unknown()),
                            "unit" => Value::string(item.unit().to_string(), Span::unknown()),
                        };

                        Value::record(item, Span::unknown())
                    }).collect();

                    record.insert(metric, Value::list(metadata, Span::unknown()));

                    record
                });

            Ok(Value::record(record, Span::unknown()))
        })
    }
}

impl Client for MetricMetadata {}
