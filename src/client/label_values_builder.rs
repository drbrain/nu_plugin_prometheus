use crate::client::SelectorParser;
use chrono::{DateTime, FixedOffset};
use nu_protocol::{LabeledError, Value};
use prometheus_http_query::{Client, LabelValuesQueryBuilder};

pub struct LabelValuesBuilder {
    client: Client,
}

impl LabelValuesBuilder {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn values(
        self,
        label: &Value,
        start: Option<DateTime<FixedOffset>>,
        end: Option<DateTime<FixedOffset>>,
        selectors: &Vec<Value>,
    ) -> Result<LabelValuesQueryBuilder, LabeledError> {
        let label = label.as_str()?.to_string();

        let mut builder = self.client.label_values(label);

        for selector in selectors {
            let selector = SelectorParser::parse(selector)?;

            builder = builder.selectors(vec![selector]);
        }

        if let Some(start) = start {
            builder = builder.start(start.timestamp());
        }

        if let Some(end) = end {
            builder = builder.end(end.timestamp());
        }

        Ok(builder)
    }
}
