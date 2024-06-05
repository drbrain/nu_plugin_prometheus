mod labels_command;
mod query_command;
mod query_range_command;
mod sources_command;
mod targets_command;

use crate::prometheus::{
    labels_command::LabelsCommand, query_command::QueryCommand,
    query_range_command::QueryRangeCommand, sources_command::SourcesCommand,
    targets_command::TargetsCommand,
};
use nu_plugin::Plugin;

#[derive(Clone)]
pub struct Prometheus;

impl Plugin for Prometheus {
    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(LabelsCommand),
            Box::new(QueryCommand),
            Box::new(QueryRangeCommand),
            Box::new(SourcesCommand),
            Box::new(TargetsCommand),
        ]
    }
}
