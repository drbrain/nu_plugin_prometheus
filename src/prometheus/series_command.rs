use crate::{
    Prometheus, Source,
    client::{SelectorParser, Series},
};
use chrono::{DateTime, FixedOffset};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, SyntaxShape, Type, Value};
use prometheus_http_query::Client;

#[derive(Clone, Default)]
pub struct SeriesCommand;

impl PluginCommand for SeriesCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus series"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .description(self.description())
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
            .named("start", SyntaxShape::DateTime, "Start timestamp", None)
            .named("end", SyntaxShape::DateTime, "End timestamp", None)
            .input_output_types(vec![
                (Type::String, Type::List(Box::new(Type::String))),
                (
                    Type::List(Box::new(Type::String)),
                    Type::List(Box::new(Type::String)),
                ),
            ])
    }

    fn description(&self) -> &str {
        "Query for series"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let call_span = call.head;

        let selectors = input.into_value(call_span)?;
        let selectors_span = selectors.span();

        let client: Client = Source::from(call, engine)?.try_into()?;

        let mut builder = match selectors {
            Value::String { .. } => client.series(vec![SelectorParser::parse(&selectors)?]),
            Value::List { vals: values, .. } => {
                let mut selectors = vec![];

                for selector in &values {
                    selectors.push(SelectorParser::parse(selector)?);
                }

                client.series(selectors)
            }
            _ => {
                return Err(LabeledError::new("Invalid input type")
                    .with_label("must be Nothing, String or list of Strings", selectors_span));
            }
        }
        .map_err(|e| LabeledError::new("Series query error").with_help(e.to_string()))?;

        if let Some(start) = call.get_flag::<DateTime<FixedOffset>>("start")? {
            builder = builder.start(start.timestamp());
        }

        if let Some(end) = call.get_flag::<DateTime<FixedOffset>>("end")? {
            builder = builder.end(end.timestamp());
        }

        Series::new(builder, selectors_span).run(engine.signals(), call_span)
    }
}
