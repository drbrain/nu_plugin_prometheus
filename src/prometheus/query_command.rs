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
            .named(
                "cert",
                SyntaxShape::Filepath,
                "Prometheus client certificate",
                Some('c'),
            )
            .named(
                "key",
                SyntaxShape::Filepath,
                "Prometheus client key",
                Some('k'),
            )
            .named(
                "cacert",
                SyntaxShape::Filepath,
                "Prometheus CA certificate",
                Some('C'),
            )
            .input_output_type(Type::String, Type::Any)
    }

    fn usage(&self) -> &str {
        "Run a prometheus query"
    }

    fn run(
        &self,
        _plugin: &Prometheus,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        match input {
            Value::String { .. } => {
                let source: Source = call.try_into()?;

                let query = Query::new(source.try_into()?, input);

                query.run()
            }
            _ => Err(
                LabeledError::new("Expected String input from pipeline").with_label(
                    format!("requires string input; got {}", input.get_type()),
                    call.head,
                ),
            ),
        }
    }
}
