mod client;
mod prometheus;
mod source;
mod sub_command;

pub(crate) use crate::sub_command::SubCommand;
use nu_plugin::{serve_plugin, JsonSerializer};
use prometheus::Prometheus;

fn main() {
    serve_plugin(&mut Prometheus::new(), JsonSerializer)
}
