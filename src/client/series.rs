use crate::{Client, client::labeled_error, signals::run_with_signal};
use nu_protocol::{
    IntoInterruptiblePipelineData, LabeledError, PipelineData, Signals, Span, Value, record,
};
use prometheus_http_query::SeriesQueryBuilder;

pub struct Series {
    builder: SeriesQueryBuilder,
    span: Span,
}

impl Series {
    pub fn new(builder: SeriesQueryBuilder, span: Span) -> Self {
        Self { builder, span }
    }

    pub fn run(self, signals: &Signals, span: Span) -> Result<PipelineData, LabeledError> {
        let Self {
            ref builder,
            span: selector_span,
        } = self;

        self.runtime()?.block_on(async {
            let series = run_with_signal(signals, span, builder.clone().get())
                .await?
                .map_err(|error| labeled_error(error, selector_span))?;

            let result = series
                .into_iter()
                .map(move |labels| {
                    let mut record = record!();

                    let mut names: Vec<_> = labels.keys().collect();
                    names.sort();

                    for name in names {
                        let value = labels.get(name).unwrap();

                        record.push(name, Value::string(value, span));
                    }

                    Value::record(record, span)
                })
                .into_pipeline_data(span, signals.clone());

            Ok(result)
        })
    }
}

impl Client for Series {}
