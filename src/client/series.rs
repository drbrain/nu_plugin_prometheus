use crate::Client;
use nu_protocol::{record, LabeledError, Span, Value};
use prometheus_http_query::SeriesQueryBuilder;

pub struct Series {
    builder: SeriesQueryBuilder,
    span: Span,
}

impl Series {
    pub fn new(builder: SeriesQueryBuilder, span: Span) -> Self {
        Self { builder, span }
    }

    pub fn run(self) -> Result<Value, LabeledError> {
        let Self { ref builder, span } = self;

        self.runtime()?.block_on(async {
            let series = builder
                .clone()
                .get()
                .await
                .map_err(|error| self.labeled_error(error, span))?;

            let result: Vec<_> = series
                .iter()
                .map(|labels| {
                    let mut record = record!();

                    let mut names: Vec<_> = labels.keys().collect();
                    names.sort();

                    for name in names {
                        let value = labels.get(name).unwrap();

                        record.push(name, Value::string(value, Span::unknown()));
                    }

                    Value::record(record, Span::unknown())
                })
                .collect();

            Ok(Value::list(result, Span::unknown()))
        })
    }
}

impl Client for Series {}
