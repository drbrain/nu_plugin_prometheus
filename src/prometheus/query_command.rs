use crate::{Prometheus, client::QueryBuilder, source::Source};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{LabeledError, PipelineData, PipelineMetadata, Signature, SyntaxShape, Type};

#[derive(Clone, Default)]
pub struct QueryCommand;

impl PluginCommand for QueryCommand {
    type Plugin = Prometheus;

    fn name(&self) -> &str {
        "prometheus query"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .description(self.description())
            .named(
                "at",
                SyntaxShape::DateTime,
                "Evaluation timestamp for an instant query",
                None,
            )
            .named("timeout", SyntaxShape::Number, "Evaluation timeout", None)
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
            .switch("no-flatten", "Do not flatten labels into record", None)
            .input_output_type(Type::String, Type::Any)
    }

    fn description(&self) -> &str {
        "Run an instant query"
    }

    fn run(
        &self,
        _plugin: &Prometheus,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        query: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let call_span = call.head;

        let (query, query_span, _) = query.collect_string_strict(call_span)?;

        let source = Source::from(call, engine)?;

        let mut query_builder = QueryBuilder::new(source.try_into()?);

        if let Some(timeout) = call.get_flag("timeout")? {
            query_builder.timeout(timeout);
        }

        if !call.has_flag("no-flatten")? {
            query_builder.flatten();
        }

        let at = call.get_flag("at")?;

        query_builder
            .instant(at, &query, query_span, call_span)
            .run()
            .map(|pipeline| {
                let metadata = PipelineMetadata::default()
                    .with_table_width_priority_columns(call_span, ["name", "value"]);

                pipeline.set_metadata(Some(metadata))
            })
    }
}
