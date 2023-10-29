use nu_plugin::{EvaluatedCall, LabeledError};
use nu_protocol::{PluginSignature, Value};

pub trait SubCommand {
    fn name(&self) -> &str;

    fn signature(&self) -> PluginSignature;

    fn usage(&self) -> &str;

    fn run(
        &mut self,
        name: &str,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError>;
}
