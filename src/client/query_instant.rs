use crate::{Client, Query};
use nu_protocol::{LabeledError, Span, Value};
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

    pub fn run(self) -> Result<Value, LabeledError> {
        let QueryInstant {
            ref query,
            query_span: span,
            flatten,
            ..
        } = self;

        self.runtime()?.block_on(async {
            let response = query
                .clone()
                .get()
                .await
                .map_err(|error| self.labeled_error(error, span))?;

            let value = match response.data() {
                Data::Vector(v) => self.vector_to_value(v, flatten),
                Data::Matrix(m) => self.matrix_to_value(m, flatten),
                Data::Scalar(s) => self.scalar_to_value(s),
            };

            Ok(value)
        })
    }
}

impl Client for QueryInstant {}

impl Query for QueryInstant {
    fn call_span(&self) -> Span {
        self.call_span
    }
}
