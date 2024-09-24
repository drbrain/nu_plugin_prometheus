use crate::{client::QueryBuilder, source::Source, Prometheus};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, SyntaxShape, Type, Value};

#[derive(Clone, Default)]
pub struct QueryCommand;

impl SimplePluginCommand for QueryCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus query"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .description(self.description())
            .named(
                "at",
                SyntaxShape::DateTime,
                "Evaluation timestamp for an instant query",
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

    fn description(&self) -> &str {
        "Run an instant query"
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

        let at = call.get_flag("at")?;

        query_builder.instant(at, query).run()
    }
}
