use crate::{Prometheus, source::Source};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{IntoPipelineData, LabeledError, PipelineData, Signature, Type};

#[derive(Clone, Default)]
pub struct SourcesCommand;

impl PluginCommand for SourcesCommand {
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
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        // validate only, format is different from config
        Source::list(engine)?;

        let sources = engine
            .get_plugin_config()?
            .ok_or_else(|| {
                // unreachable, Sources::list already validated
                LabeledError::new("prometheus plugin config missing")
            })?
            .get_data_by_key("sources")
            .ok_or_else(|| {
                LabeledError::new("No sources defined")
                    .with_help("Use `config nu` to define prometheus sources")
                    .with_url("https://github.com/drbrain/nu_plugin_prometheus#sources")
            })?
            .into_pipeline_data();

        Ok(sources)
    }
}
