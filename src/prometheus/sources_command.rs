use crate::{source::Source, sub_command::SubCommand};
use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{PluginSignature, Type, Value};

#[derive(Clone, Default)]
pub struct SourcesCommand;

impl SubCommand for SourcesCommand {
    fn name(&self) -> &str {
        "prometheus sources"
    }

    fn signature(&self) -> nu_protocol::PluginSignature {
        PluginSignature::build(self.name())
            .usage(self.usage())
            .input_output_type(Type::Nothing, Type::Table(vec![]))
    }

    fn usage(&self) -> &str {
        "List configured prometheus sources"
    }

    fn run(
        &mut self,
        name: &str,
        _call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        assert_eq!("prometheus sources", name);

        Source::list()
    }
}
