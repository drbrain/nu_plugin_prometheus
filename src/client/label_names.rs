use crate::{Client, client::labeled_error};
use nu_protocol::{
    IntoInterruptiblePipelineData, LabeledError, PipelineData, Signals, Span, Value,
};
use prometheus_http_query::LabelNamesQueryBuilder;

pub struct LabelNames {
    query: LabelNamesQueryBuilder,
    selectors_span: Span,
    call_span: Span,
}

impl LabelNames {
    pub fn new(query: LabelNamesQueryBuilder, selectors_span: Span, call_span: Span) -> Self {
        Self {
            query,
            selectors_span,
            call_span,
        }
    }

    pub fn run(self) -> Result<PipelineData, LabeledError> {
        let runtime = self.runtime()?;

        let Self {
            query,
            selectors_span: query_span,
            call_span,
        } = self;

        runtime.block_on(async {
            let response = query
                .clone()
                .get()
                .await
                .map_err(|error| labeled_error(error, query_span))?;

            let names = response
                .into_iter()
                .map(move |name| Value::string(name, call_span))
                .into_pipeline_data(call_span, Signals::empty());

            Ok(names)
        })
    }
}

impl Client for LabelNames {}
