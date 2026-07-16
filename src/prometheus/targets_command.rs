use crate::{Prometheus, client::Targets, source::Source};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, SyntaxShape, Type};
use prometheus_http_query::TargetState;

#[derive(Clone, Default)]
pub struct TargetsCommand;

impl PluginCommand for TargetsCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus targets"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .description(self.description())
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
            .optional("state", SyntaxShape::String, "Target state filter")
            .input_output_types(vec![
                (Type::Nothing, Type::record()),
                (Type::Nothing, Type::table()),
            ])
    }

    fn description(&self) -> &str {
        "Query for target discovery state"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let call_span = call.head;

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

        Targets::new(source.try_into()?, target_state).run(engine.signals(), call_span)
    }
}
