use crate::{Client, client::labeled_error, signals::run_with_signal};
use nu_protocol::{
    IntoInterruptiblePipelineData, LabeledError, PipelineData, Signals, Span, Value,
};
use prometheus_http_query::LabelValuesQueryBuilder;

pub struct LabelValues {
    query: LabelValuesQueryBuilder,
    labels_span: Span,
    call_span: Span,
}

impl LabelValues {
    pub fn new(query: LabelValuesQueryBuilder, labels_span: Span, call_span: Span) -> Self {
        Self {
            query,
            labels_span,
            call_span,
        }
    }

    pub fn run(self, signals: &Signals) -> Result<PipelineData, LabeledError> {
        let runtime = self.runtime()?;

        let Self {
            query,
            labels_span,
            call_span,
        } = self;

        runtime.block_on(async {
            let response = run_with_signal(signals, call_span, query.clone().get())
                .await?
                .map_err(|error| labeled_error(error, labels_span))?;

            let names = response
                .into_iter()
                .map(move |name| Value::string(name, call_span))
                .into_pipeline_data(call_span, signals.clone());

            Ok(names)
        })
    }
}

impl Client for LabelValues {}
