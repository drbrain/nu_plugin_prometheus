use crate::{client::Targets, source::Source, Prometheus};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, SyntaxShape, Type, Value};
use prometheus_http_query::TargetState;

#[derive(Clone, Default)]
pub struct TargetsCommand;

impl SimplePluginCommand for TargetsCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus targets"
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
            .optional("state", SyntaxShape::String, "Target state filter")
            .input_output_types(vec![
                (Type::Nothing, Type::record()),
                (Type::Nothing, Type::table()),
            ])
    }

    fn usage(&self) -> &str {
        "Show target discovery state"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let state: Option<String> = call.opt(0)?.map(|state: String| state.to_ascii_lowercase());

        let target_state = match state.as_deref() {
            Some("active") => Some(TargetState::Active),
            Some("any") => Some(TargetState::Any),
            Some("dropped") => Some(TargetState::Dropped),
            Some(_) => {
                return Err(LabeledError::new("Invalid state").with_label(
                    "Must be active, any, or dropped",
                    call.nth(0).unwrap().span(),
                ));
            }
            None => None,
        };

        let source = Source::from(call, engine)?;

        Targets::new(source.try_into()?, input.span(), target_state).run()
    }
}
