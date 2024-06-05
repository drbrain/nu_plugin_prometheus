use chrono::{DateTime, FixedOffset};
use nu_protocol::{LabeledError, Value};
use prometheus_http_query::{Client, LabelNamesQueryBuilder, Selector};

pub struct LabelsBuilder {
    client: Client,
}

impl LabelsBuilder {
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
            Value::String { val: selector, .. } => {
                builder.selectors(vec![Selector::new().metric(selector)])
            }
            Value::List { vals: values, .. } => {
                let mut selectors = vec![];

                for selector in values {
                    let Value::String { val: selector, .. } = selector else {
                        return Err(LabeledError::new("Invalid input type")
                            .with_label("must be a string", selector.span()));
                    };

                    selectors.push(Selector::new().metric(selector));
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
