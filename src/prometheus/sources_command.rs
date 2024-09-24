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
            .description(self.description())
            .input_output_type(Type::Nothing, Type::table())
    }

    fn description(&self) -> &str {
        "List configured sources"
    }

    fn run(
        &self,
        _plugin: &Prometheus,
        engine: &EngineInterface,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        // validate
        Source::list(engine)?;

        Ok(engine
            .get_plugin_config()
            .expect("already validated by Source::list()")
            .unwrap()
            .get_data_by_key("sources")
            .unwrap())
    }
}
