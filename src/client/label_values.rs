use crate::Client;
use nu_protocol::{LabeledError, Span, Value};
use prometheus_http_query::LabelValuesQueryBuilder;

pub struct LabelValues {
    query: LabelValuesQueryBuilder,
    span: Span,
}

impl LabelValues {
    pub fn new(query: LabelValuesQueryBuilder, span: Span) -> Self {
        Self { query, span }
    }

    pub fn run(self) -> Result<Value, LabeledError> {
        let Self { ref query, span } = self;

        self.runtime()?.block_on(async {
            let response = query
                .clone()
                .get()
                .await
                .map_err(|error| self.labeled_error(error, span))?;

            let names = response
                .iter()
                .map(|name| Value::string(name, Span::unknown()))
                .collect();

            Ok(Value::list(names, Span::unknown()))
        })
    }
}

impl Client for LabelValues {}
