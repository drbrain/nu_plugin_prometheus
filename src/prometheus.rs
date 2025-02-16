mod label_names_command;
mod label_values_command;
mod metric_metadata_command;
mod parse_command;
mod prometheus_command;
mod query_command;
mod query_range_command;
mod scrape_command;
mod series_command;
mod sources_command;
mod targets_command;

use crate::prometheus::{
    label_names_command::LabelNamesCommand, label_values_command::LabelValuesCommand,
    metric_metadata_command::MetricMetadataCommand, prometheus_command::PrometheusCommand,
    query_command::QueryCommand, query_range_command::QueryRangeCommand,
    series_command::SeriesCommand, sources_command::SourcesCommand,
    targets_command::TargetsCommand,
};
use nu_plugin::Plugin;
use parse_command::ParseCommand;
use scrape_command::ScrapeCommand;

#[derive(Clone)]
pub struct Prometheus;

impl Plugin for Prometheus {
    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(LabelNamesCommand),
            Box::new(LabelValuesCommand),
            Box::new(MetricMetadataCommand),
            Box::new(ParseCommand),
            Box::new(PrometheusCommand),
            Box::new(QueryCommand),
            Box::new(QueryRangeCommand),
            Box::new(SeriesCommand),
            Box::new(ScrapeCommand),
            Box::new(SourcesCommand),
            Box::new(TargetsCommand),
        ]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }
}
