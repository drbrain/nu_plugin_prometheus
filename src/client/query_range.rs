use crate::{Client, Query};
use nu_protocol::{LabeledError, Span, Value};
use prometheus_http_query::{response::Data, RangeQueryBuilder};

pub struct QueryRange {
    query: RangeQueryBuilder,
    span: Span,
    flatten: bool,
}

impl QueryRange {
    pub fn new(query: RangeQueryBuilder, span: Span, flatten: bool) -> Self {
        Self {
            query,
            span,
            flatten,
        }
    }

    pub fn run(self) -> Result<Value, LabeledError> {
        let QueryRange {
            ref query,
            span,
            flatten,
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

impl Client for QueryRange {}

impl Query for QueryRange {}
