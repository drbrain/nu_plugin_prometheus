use crate::client::{QueryInstant, QueryRange};
use chrono::{DateTime, FixedOffset};
use nu_protocol::Value;
use prometheus_http_query::Client;

pub struct QueryBuilder {
    client: Client,
    flatten: bool,
    timeout: Option<i64>,
}

impl QueryBuilder {
    pub fn new(client: Client) -> Self {
        QueryBuilder {
            client,
            flatten: false,
            timeout: None,
        }
    }

    pub fn flatten(&mut self) {
        self.flatten = true;
    }

    pub fn timeout(&mut self, timeout: i64) {
        self.timeout = Some(timeout);
    }

    pub fn instant(self, at: Option<DateTime<FixedOffset>>, query: &Value) -> QueryInstant {
        let span = query.span();

        let query = query.clone().into_string().expect("Query must be a String");

        let mut query = self.client.query(query);

        if let Some(at) = at {
            query = query.at(at.timestamp());
        }

        if let Some(timeout) = self.timeout {
            query = query.timeout(timeout);
        }

        QueryInstant::new(query, span, self.flatten)
    }

    pub fn range(
        self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        step: f64,
        query: &Value,
    ) -> QueryRange {
        let span = query.span();

        let query = query.clone().into_string().expect("Query must be a String");

        let start = start.timestamp();
        let end = end.timestamp();

        let mut query = self.client.query_range(query, start, end, step);

        if let Some(timeout) = self.timeout {
            query = query.timeout(timeout);
        }

        QueryRange::new(query, span, self.flatten)
    }
}
