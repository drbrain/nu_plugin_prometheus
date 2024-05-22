mod query_command;
mod sources_command;

use crate::prometheus::{query_command::QueryCommand, sources_command::SourcesCommand};
use nu_plugin::Plugin;

#[derive(Clone)]
pub struct Prometheus;

impl Plugin for Prometheus {
    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(QueryCommand), Box::new(SourcesCommand)]
    }
}
