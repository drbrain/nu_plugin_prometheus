mod label_names;
mod label_names_builder;
mod label_values;
mod label_values_builder;
mod metric_metadata;
mod parse;
mod query_builder;
mod query_instant;
mod query_range;
mod scrape;
mod selector_parser;
mod series;
mod targets;

pub use label_names::LabelNames;
pub use label_names_builder::LabelNamesBuilder;
pub use label_values::LabelValues;
pub use label_values_builder::LabelValuesBuilder;
pub use metric_metadata::MetricMetadata;
use nu_protocol::{LabeledError, Span};
pub use parse::Parse;
pub use parse::ParseFormat;
pub use query_builder::QueryBuilder;
pub use query_instant::QueryInstant;
pub use query_range::QueryRange;
pub use scrape::Scrape;
pub use selector_parser::SelectorParser;
pub use series::Series;
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
