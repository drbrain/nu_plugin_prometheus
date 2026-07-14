use crate::{
    Prometheus,
    client::{LabelValues, LabelValuesBuilder},
    source::Source,
};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, Signature, SyntaxShape, Type};

#[derive(Clone, Default)]
pub struct LabelValuesCommand;

impl PluginCommand for LabelValuesCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus label values"
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
            .rest(
                "selectors",
                SyntaxShape::String,
                "Series selectors to filter by",
            )
            .input_output_types(vec![
                (Type::Nothing, Type::table()),
                (Type::String, Type::table()),
                (Type::List(Box::new(Type::String)), Type::table()),
            ])
    }

    fn description(&self) -> &str {
        "Query for label values"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let call_span = call.head;

        let label = input.into_value(call_span)?;
        let label_span = label.span();

        let source = Source::from(call, engine)?;

        let builder = LabelValuesBuilder::new(source.try_into()?);

        let start = call.get_flag("start")?;
        let end = call.get_flag("end")?;
        let selectors = call.rest(0)?;

        LabelValues::new(
            builder.values(&label, start, end, &selectors)?,
            label_span,
            call_span,
        )
        .run(engine.signals())
    }
}
