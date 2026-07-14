use crate::{
    Client,
    client::labeled_error,
    query::{matrix_to_value, scalar_to_value, vector_to_value},
    signals::run_with_signal,
};
use nu_protocol::{IntoPipelineData, LabeledError, PipelineData, Signals, Span};
use prometheus_http_query::{RangeQueryBuilder, response::Data};

pub struct QueryRange {
    query: RangeQueryBuilder,
    query_span: Span,
    flatten: bool,
    call_span: Span,
}

impl QueryRange {
    pub fn new(query: RangeQueryBuilder, query_span: Span, flatten: bool, call_span: Span) -> Self {
        Self {
            query,
            query_span,
            flatten,
            call_span,
        }
    }

    pub fn run(self, signals: &Signals) -> Result<PipelineData, LabeledError> {
        let QueryRange {
            ref query,
            query_span,
            flatten,
            call_span,
        } = self;

        self.runtime()?.block_on(async {
            let response = run_with_signal(signals, call_span, query.clone().get())
                .await?
                .map_err(|error| labeled_error(error, query_span))?;

            let pipeline = match response.into_inner().0 {
                Data::Vector(v) => vector_to_value(v, flatten, call_span, signals),
                Data::Matrix(m) => matrix_to_value(m, flatten, call_span, signals),
                Data::Scalar(s) => scalar_to_value(&s, call_span).into_pipeline_data(),
            };

            Ok(pipeline)
        })
    }
}

impl Client for QueryRange {}
