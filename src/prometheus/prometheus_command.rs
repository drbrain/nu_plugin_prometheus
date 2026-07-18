use crate::Prometheus;
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, Type};

#[derive(Clone, Default)]
pub struct PrometheusCommand;

impl PluginCommand for PrometheusCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name()).input_output_type(Type::Nothing, Type::Nothing)
    }

    fn description(&self) -> &str {
        "Prometheus plugin"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        _call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        eprintln!("{}", engine.get_help()?);

        Ok(PipelineData::Empty)
    }
}
