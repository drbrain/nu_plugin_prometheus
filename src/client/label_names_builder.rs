use crate::client::SelectorParser;
use chrono::{DateTime, FixedOffset};
use nu_protocol::{LabeledError, Value};
use prometheus_http_query::{Client, LabelNamesQueryBuilder};

pub struct LabelNamesBuilder {
    client: Client,
}

impl LabelNamesBuilder {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn names(
        self,
        start: Option<DateTime<FixedOffset>>,
        end: Option<DateTime<FixedOffset>>,
        selectors: &Value,
    ) -> Result<LabelNamesQueryBuilder, LabeledError> {
        let span = selectors.span();

        let builder = self.client.label_names();

        let mut builder = match selectors {
            Value::Nothing { .. } => builder,
            Value::String { .. } => builder.selectors(vec![SelectorParser::parse(selectors)?]),
            Value::List { vals: values, .. } => {
                let mut selectors = vec![];

                for selector in values {
                    selectors.push(SelectorParser::parse(selector)?);
                }

                builder.selectors(selectors)
            }
            _ => {
                return Err(LabeledError::new("Invalid input type")
                    .with_label("must be String or list of Strings", span));
            }
        };

        if let Some(start) = start {
            builder = builder.start(start.timestamp());
        }

        if let Some(end) = end {
            builder = builder.end(end.timestamp());
        }

        Ok(builder)
    }
}
