use nu_protocol::{
    IntoInterruptiblePipelineData, LabeledError, PipelineData, Signals, Span, Value, record,
};
use prometheus_http_query::MetricMetadataQueryBuilder;

use crate::{client::labeled_error, signals::run_with_signal};

use super::Client;

pub struct MetricMetadata {
    builder: MetricMetadataQueryBuilder,
    span: Span,
}

impl MetricMetadata {
    pub fn new(builder: MetricMetadataQueryBuilder, span: Span) -> Self {
        Self { builder, span }
    }

    pub fn run(self, signals: &Signals, call_span: Span) -> Result<PipelineData, LabeledError> {
        let Self {
            ref builder,
            span: metric_span,
        } = self;

        self.runtime()?.block_on(async {
            let metric_metadata = run_with_signal(signals, call_span, builder.clone().get())
                .await?
                .map_err(|error| labeled_error(error, metric_span))?;

            let pipeline = metric_metadata
                .into_iter()
                .map(move |(metric, metadata)| {
                    let metadata: Vec<_> = metadata
                        .iter()
                        .map(|item| {
                            let item = record! {
                                "type" => Value::string(item.metric_type().to_string(), call_span),
                                "help" => Value::string(item.help().to_string(), call_span),
                                "unit" => Value::string(item.unit().to_string(), call_span),
                            };

                            Value::record(item, call_span)
                        })
                        .collect();

                    Value::record(
                        record! {
                            "name" => Value::string(metric, call_span),
                            "metadata" => Value::list(metadata, call_span),
                        },
                        call_span,
                    )
                })
                .into_pipeline_data(call_span, signals.clone());

            Ok(pipeline)
        })
    }
}

impl Client for MetricMetadata {}
