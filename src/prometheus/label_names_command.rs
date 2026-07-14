use crate::{
    Prometheus,
    client::{LabelNames, LabelNamesBuilder},
    source::Source,
};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, SyntaxShape, Type};

#[derive(Clone, Default)]
pub struct LabelNamesCommand;

impl PluginCommand for LabelNamesCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus label names"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .description(self.description())
            .named(
                "start",
                SyntaxShape::DateTime,
                "Start timestamp for a labels query",
                None,
            )
            .named(
                "end",
                SyntaxShape::DateTime,
                "End timestamp for a labels query",
                None,
            )
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
            .input_output_types(vec![
                (Type::Nothing, Type::table()),
                (Type::String, Type::table()),
                (Type::List(Box::new(Type::String)), Type::table()),
            ])
    }

    fn description(&self) -> &str {
        "Query for label names"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let call_span = call.head;

        let selectors = input.into_value(call_span)?;

        let source = Source::from(call, engine)?;

        let builder = LabelNamesBuilder::new(source.try_into()?);

        let start = call.get_flag("start")?;
        let end = call.get_flag("end")?;

        LabelNames::new(
            builder.names(start, end, &selectors)?,
            selectors.span(),
            call.head,
        )
        .run(engine.signals())
    }
}
