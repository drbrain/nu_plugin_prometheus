mod label_names;
mod labels_builder;
mod query_builder;
mod query_instant;
mod query_range;
mod targets;

pub use label_names::LabelNames;
pub use labels_builder::LabelsBuilder;
use nu_protocol::{LabeledError, Span};
pub use query_builder::QueryBuilder;
pub use query_instant::QueryInstant;
pub use query_range::QueryRange;
pub use targets::Targets;

pub trait Client {
    fn labeled_error(&self, error: prometheus_http_query::Error, span: Span) -> LabeledError {
        use prometheus_http_query::Error;

        match error {
            Error::Client(e) => {
                LabeledError::new("Prometheus client error").with_label(e.to_string(), span)
            }
            Error::EmptySeriesSelector => {
                LabeledError::new("Empty series selector").with_label("", span)
            }
            // This error should be impossible to reach because it should occur when building the client
            Error::ParseUrl(e) => LabeledError::new("Invalid URL").with_help(e.to_string()),
            Error::Prometheus(e) => {
                LabeledError::new("Prometheus error").with_label(e.to_string(), span)
            }
            e => LabeledError::new("Other error").with_label(e.to_string(), span),
        }
    }

    fn runtime(&self) -> Result<tokio::runtime::Runtime, LabeledError> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| LabeledError::new("Tokio runtime build error").with_help(e.to_string()))
    }
}
