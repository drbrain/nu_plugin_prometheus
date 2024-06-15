use crate::{client::MetricMetadata, Prometheus, Source};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, SyntaxShape, Type, Value};
use prometheus_http_query::Client;

#[derive(Clone, Default)]
pub struct MetricMetadataCommand;

impl SimplePluginCommand for MetricMetadataCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus metric metadata"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .usage(self.usage())
            .named(
                "source",
                SyntaxShape::String,
                "Prometheus source to query",
                Some('s'),
            )
            .named(
                "url",
                SyntaxShape::String,
                "Prometheus source url to query",
                Some('u'),
            )
            .named(
                "limit",
                SyntaxShape::Int,
                "Maximum number of metrics to retrieve",
                None,
            )
            .named(
                "limit-per-metric",
                SyntaxShape::Int,
                "Maximum number of metadata to retrieve per metric",
                None,
            )
            .input_output_types(vec![
                (Type::Nothing, Type::record()),
                (Type::String, Type::record()),
            ])
    }

    fn usage(&self) -> &str {
        "Retrieve metric metadata"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        metric: &Value,
    ) -> Result<Value, LabeledError> {
        let span = metric.span();

        let client: Client = Source::from(call, engine)?.try_into()?;

        let mut builder = client.metric_metadata();

        if let Value::String { val: metric, .. } = metric {
            builder = builder.metric(metric);
        };

        if let Some(limit) = call.get_flag::<i64>("limit")? {
            let limit = limit.try_into().map_err(|_| {
                let span = call.get_flag_value("limit").unwrap().span();

                LabeledError::new("Limit too large").with_label("does not fit in i32", span)
            })?;

            builder = builder.limit(limit);
        }

        if let Some(limit_per_metric) = call.get_flag::<i64>("limit-per-metric")? {
            let limit_per_metric = limit_per_metric.try_into().map_err(|_| {
                let span = call.get_flag_value("limit-per-metric").unwrap().span();

                LabeledError::new("Limit per metric too large")
                    .with_label("does not fit in i32", span)
            })?;

            builder = builder.limit_per_metric(limit_per_metric);
        }

        MetricMetadata::new(builder, span).run()
    }
}
