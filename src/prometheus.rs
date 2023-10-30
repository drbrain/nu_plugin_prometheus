mod query_command;
mod sources_command;

use crate::{
    prometheus::{query_command::QueryCommand, sources_command::SourcesCommand},
    SubCommand,
};
use nu_plugin::{LabeledError, Plugin};
use nu_protocol::{Category, PluginSignature, Type, Value};

#[derive(Clone)]
pub struct Prometheus;

impl Prometheus {
    pub fn new() -> Self {
        Self
    }
}

impl Prometheus {
    fn prometheus_signature(&self) -> PluginSignature {
        PluginSignature::build("prometheus")
            .usage("Prometheus interface for nushell")
            .extra_usage("You must use a prometheus subcommand, this only shows help")
            .category(Category::Network)
            .search_terms(vec!["network".into(), "prometheus".into()])
            .input_output_type(Type::Nothing, Type::String)
    }
}

impl Plugin for Prometheus {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![
            self.prometheus_signature(),
            QueryCommand::default().signature(),
            SourcesCommand::default().signature(),
        ]
    }

    fn run(
        &mut self,
        name: &str,
        call: &nu_plugin::EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        match name {
            "prometheus query" => QueryCommand::default().run(name, call, input),
            "prometheus sources" => SourcesCommand::default().run(name, call, input),
            "prometheus" => {
                return Err(LabeledError {
                    label: "Missing subcommand".into(),
                    msg: "Use a prometheus subcommand".into(),
                    span: Some(call.head),
                });
            }
            _ => {
                return Err(LabeledError {
                    label: "Invalid invocation".into(),
                    msg: format!("Unknown prometheus command {name:?}"),
                    span: Some(call.head),
                });
            }
        }
    }
}
