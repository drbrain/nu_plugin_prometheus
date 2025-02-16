use crate::{client::Scrape, Prometheus};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Type, Value};

#[derive(Clone, Default)]
pub struct ScrapeCommand;

impl SimplePluginCommand for ScrapeCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus scrape"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .description(self.description())
            .input_output_type(Type::String, Type::table())
    }

    fn description(&self) -> &str {
        "Scrape a prometheus target"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        _call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let target = input.as_str()?;

        Scrape::new(target.into()).run()
    }
}
