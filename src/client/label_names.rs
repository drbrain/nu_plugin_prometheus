use crate::Client;
use nu_protocol::{LabeledError, Span, Value};
use prometheus_http_query::LabelNamesQueryBuilder;

pub struct LabelNames {
    query: LabelNamesQueryBuilder,
    span: Span,
}

impl LabelNames {
    pub fn new(query: LabelNamesQueryBuilder, span: Span) -> Self {
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

impl Client for LabelNames {}
