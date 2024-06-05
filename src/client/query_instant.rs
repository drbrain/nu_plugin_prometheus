use crate::{
    query::{matrix_to_value, scalar_to_value, vector_to_value},
    Client,
};
use nu_protocol::{LabeledError, Span, Value};
use prometheus_http_query::{response::Data, InstantQueryBuilder};

pub struct QueryInstant {
    query: InstantQueryBuilder,
    span: Span,
    flatten: bool,
}

impl QueryInstant {
    pub fn new(query: InstantQueryBuilder, span: Span, flatten: bool) -> Self {
        Self {
            query,
            span,
            flatten,
        }
    }

    pub fn run(self) -> Result<Value, LabeledError> {
        let QueryInstant {
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
                Data::Vector(v) => vector_to_value(v, flatten),
                Data::Matrix(m) => matrix_to_value(m, flatten),
                Data::Scalar(s) => scalar_to_value(s),
            };

            Ok(value)
        })
    }
}

impl Client for QueryInstant {}
