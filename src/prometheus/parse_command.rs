use crate::{
    Prometheus,
    client::{Parse, ParseFormat},
};
use nu_plugin::{EngineInterface, EvaluatedCall, SimplePluginCommand};
use nu_protocol::{
    DynamicSuggestion, LabeledError, Signature, Span, SyntaxShape, Type, Value,
    engine::{ArgType, ExperimentalMarker},
};

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

    fn get_dynamic_completion(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        _call: nu_plugin::DynamicCompletionCall,
        arg_type: ArgType,
        _experimental: ExperimentalMarker,
    ) -> Option<Vec<DynamicSuggestion>> {
        match arg_type {
            ArgType::Flag(flag) => match flag.as_ref() {
                "format" => Some(vec![
                    DynamicSuggestion {
                        value: "openmetrics".to_string(),
                        kind: Some(nu_protocol::SuggestionKind::Value(Type::String)),
                        ..Default::default()
                    },
                    DynamicSuggestion {
                        value: "prometheus".to_string(),
                        kind: Some(nu_protocol::SuggestionKind::Value(Type::String)),
                        description: Some("default".into()),
                        ..Default::default()
                    },
                ]),
                _ => None,
            },
            _ => None,
        }
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
