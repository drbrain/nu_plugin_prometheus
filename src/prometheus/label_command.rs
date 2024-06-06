use crate::Prometheus;
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, Type, Value};

#[derive(Clone, Default)]
pub struct LabelCommand;

impl SimplePluginCommand for LabelCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus label"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).input_output_type(Type::Nothing, Type::Nothing)
    }

    fn usage(&self) -> &str {
        "Prometheus label names and values"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        eprintln!("{}", engine.get_help()?);

        Ok(Value::nothing(Span::unknown()))
    }
}
