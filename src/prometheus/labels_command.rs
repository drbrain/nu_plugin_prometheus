use crate::{
    client::{LabelNames, LabelsBuilder},
    source::Source,
    Prometheus,
};
use nu_plugin::SimplePluginCommand;
use nu_protocol::{Signature, SyntaxShape, Type};

#[derive(Clone, Default)]
pub struct LabelsCommand;

impl SimplePluginCommand for LabelsCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus labels"
    }

    fn signature(&self) -> nu_protocol::Signature {
        Signature::build(self.name())
            .usage(self.usage())
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

    fn usage(&self) -> &str {
        "Query for label names"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        selectors: &nu_protocol::Value,
    ) -> Result<nu_protocol::Value, nu_protocol::LabeledError> {
        let span = selectors.span();
        let source = Source::from(call, engine)?;

        let builder = LabelsBuilder::new(source.try_into()?);

        let start = call.get_flag("start")?;
        let end = call.get_flag("end")?;

        LabelNames::new(builder.names(start, end, selectors)?, span).run()
    }
}
