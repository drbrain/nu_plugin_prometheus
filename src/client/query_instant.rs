use crate::{
    Client,
    query::{matrix_to_value, scalar_to_value, vector_to_value},
};
use nu_protocol::{IntoPipelineData, LabeledError, PipelineData, Span};
use prometheus_http_query::{InstantQueryBuilder, response::Data};

pub struct QueryInstant {
    query: InstantQueryBuilder,
    query_span: Span,
    flatten: bool,
    call_span: Span,
}

impl QueryInstant {
    pub fn new(
        query: InstantQueryBuilder,
        query_span: Span,
        flatten: bool,
        call_span: Span,
    ) -> Self {
        Self {
            query,
            query_span,
            flatten,
            call_span,
        }
    }

    pub fn run(self) -> Result<PipelineData, LabeledError> {
        let QueryInstant {
            ref query,
            query_span,
            flatten,
            call_span,
        } = self;

        self.runtime()?.block_on(async {
            let response = query
                .clone()
                .get()
                .await
                .map_err(|error| self.labeled_error(error, query_span))?;

            let data = match response.into_inner().0 {
                Data::Vector(v) => vector_to_value(v, flatten, call_span),
                Data::Matrix(m) => matrix_to_value(m, flatten, call_span),
                Data::Scalar(s) => scalar_to_value(&s, call_span).into_pipeline_data(),
            };

            Ok(data)
        })
    }
}

impl Client for QueryInstant {}
