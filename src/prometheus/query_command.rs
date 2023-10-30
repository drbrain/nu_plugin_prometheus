use crate::{client::Query, source::Source, SubCommand};
use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{PluginSignature, SyntaxShape, Type, Value};

#[derive(Clone)]
pub struct QueryCommand;

impl QueryCommand {
    pub fn new() -> Self {
        Self
    }
}

impl SubCommand for QueryCommand {
    fn name(&self) -> &str {
        "prometheus query"
    }

    fn signature(&self) -> PluginSignature {
        PluginSignature::build("prometheus query")
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
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        assert_eq!("prometheus query", name);

        match input {
            Value::String { .. } => {
                let source: Source = call.try_into()?;

                let query = Query::new(source.try_into()?, input);

                query.run()
            }
            _ => Err(LabeledError {
                label: "Expected String input from pipeline".to_string(),
                msg: format!("requires string input; got {}", input.get_type()),
                span: Some(call.head),
            }),
        }
    }
}
