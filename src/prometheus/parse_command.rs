use crate::{
    client::{Parse, ParseFormat},
    Prometheus,
};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{LabeledError, Signature, Span, SyntaxShape, Type, Value};

#[derive(Clone, Default)]
pub struct ParseCommand;

impl SimplePluginCommand for ParseCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus parse"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .description(self.description())
            .named(
                "format",
                SyntaxShape::String,
                "Metrics format, prometheus (default) or openmetrics",
                None,
            )
            .input_output_types(vec![(Type::String, Type::table())])
    }

    fn description(&self) -> &str {
        "Parse prometheus or openmetrics output"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        let format = call
            .get_flag_value("format")
            .unwrap_or(Value::string("prometheus", Span::unknown()));
        let format_span = format.span();
        let format = format.into_string()?;

        let mut parser = Parse::new(input);

        match format.as_str() {
            "prometheus" => parser.set_format(ParseFormat::Prometheus),
            "openmetrics" => parser.set_format(ParseFormat::Openmetrics),
            _ => {
                return Err(LabeledError::new("Invalid format")
                    .with_label("must be prometheus or openmetrics", format_span));
            }
        }

        parser.run()
    }
}
