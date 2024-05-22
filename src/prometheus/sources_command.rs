use crate::{source::Source, Prometheus};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Type, Value};

#[derive(Clone, Default)]
pub struct SourcesCommand;

impl SimplePluginCommand for SourcesCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus sources"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .usage(self.usage())
            .input_output_type(Type::Nothing, Type::table())
    }

    fn usage(&self) -> &str {
        "List configured prometheus sources"
    }

    fn run(
        &self,
        _plugin: &Prometheus,
        _engine: &EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        Source::list()
    }
}
