use crate::{client::QueryBuilder, source::Source, Prometheus};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, SyntaxShape, Type, Value};

#[derive(Clone, Default)]
pub struct QueryRangeCommand;

impl SimplePluginCommand for QueryRangeCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus query range"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .usage(self.usage())
            .named(
                "start",
                SyntaxShape::DateTime,
                "Start timestamp for a range query",
                None,
            )
            .named(
                "end",
                SyntaxShape::DateTime,
                "End timestamp for a range query",
                None,
            )
            .named(
                "step",
                SyntaxShape::Duration,
                "Query resolution step width",
                None,
            )
            .named("timeout", SyntaxShape::Number, "Evaluation timeout", None)
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
            .switch("flatten", "Flatten labels into record", Some('f'))
            .input_output_type(Type::String, Type::Any)
    }

    fn usage(&self) -> &str {
        "Run a range query"
    }

    fn run(
        &self,
        _plugin: &Prometheus,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        query: &Value,
    ) -> Result<Value, LabeledError> {
        if !matches!(query, Value::String { .. }) {
            // Unreachable as we only accept String input
            return Err(
                LabeledError::new("Expected query string from pipeline").with_label(
                    format!("requires string input; got {}", query.get_type()),
                    call.head,
                ),
            );
        }

        let source = Source::from(call, engine)?;

        let mut query_builder = QueryBuilder::new(source.try_into()?);

        if let Some(timeout) = call.get_flag("timeout")? {
            query_builder.timeout(timeout);
        }

        if call.has_flag("flatten")? {
            query_builder.flatten();
        }

        let start = call.get_flag("start")?;
        let end = call.get_flag("end")?;
        let step = call.get_flag::<i64>("step")?;

        match (start, end, step) {
            (Some(start), Some(end), Some(step)) => {
                let step = step as f64 / 1_000_000_000.0;

                query_builder.range(start, end, step, query).run()
            }
            _ => {
                let mut missing = vec![];

                if call.get_flag_value("start").is_none() {
                    missing.push("--start");
                }
                if call.get_flag_value("end").is_none() {
                    missing.push("--end");
                }
                if call.get_flag_value("step").is_none() {
                    missing.push("--step");
                }
                let missing = missing.join(", ");

                Err(LabeledError::new("Missing query range arguments")
                    .with_label(format!("Missing: {missing}"), query.span()))
            }
        }
    }
}
