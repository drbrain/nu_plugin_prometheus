use crate::{
    client::{LabelNames, LabelNamesBuilder},
    source::Source,
    Prometheus,
};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, SyntaxShape, Type, Value};

#[derive(Clone, Default)]
pub struct LabelNamesCommand;

impl SimplePluginCommand for LabelNamesCommand {
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
        selectors: &Value,
    ) -> Result<Value, LabeledError> {
        let span = selectors.span();
        let source = Source::from(call, engine)?;

        let builder = LabelNamesBuilder::new(source.try_into()?);

        let start = call.get_flag("start")?;
        let end = call.get_flag("end")?;

        LabelNames::new(builder.names(start, end, selectors)?, span).run()
    }
}
