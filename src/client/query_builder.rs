use crate::client::Query;
use chrono::{DateTime, FixedOffset};
use nu_protocol::Value;
use prometheus_http_query::Client;

pub struct QueryBuilder {
    at: Option<DateTime<FixedOffset>>,
    client: Client,
    flatten: bool,
    timeout: Option<i64>,
}

impl QueryBuilder {
    pub fn new(client: Client) -> Self {
        QueryBuilder {
            at: None,
            client,
            flatten: false,
            timeout: None,
        }
    }

    pub fn at(&mut self, at: DateTime<FixedOffset>) {
        self.at = Some(at);
    }

    pub fn flatten(&mut self) {
        self.flatten = true;
    }

    pub fn timeout(&mut self, timeout: i64) {
        self.timeout = Some(timeout);
    }

    pub fn build(self, query: &Value) -> Query {
        let span = query.span();

        let query = query.clone().into_string().expect("Query must be a String");

        let mut query = self.client.query(query);

        if let Some(at) = self.at {
            query = query.at(at.timestamp());
        }

        if let Some(timeout) = self.timeout {
            query = query.timeout(timeout);
        }

        Query::new(query, span, self.flatten)
    }
}
