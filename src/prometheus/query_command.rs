use crate::{client::Query, source::Source, Prometheus};
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
            .input_output_type(Type::String, Type::Any)
    }

    fn usage(&self) -> &str {
        "Run a prometheus query"
    }

    fn run(
        &self,
        _plugin: &Prometheus,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        if !matches!(input, Value::String { .. }) {
            // Unreachable as we only accept String input
            return Err(
                LabeledError::new("Expected query string from pipeline").with_label(
                    format!("requires string input; got {}", input.get_type()),
                    call.head,
                ),
            );
        }

        let source = Source::from(call, engine)?;

        let query = Query::new(source.try_into()?, input);

        query.run()
    }
}
