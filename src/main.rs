mod client;
mod prometheus;
mod query;
mod source;

use client::Client;
use nu_plugin::{serve_plugin, JsonSerializer};
use prometheus::Prometheus;
use query::Query;
use source::Source;

fn main() {
    serve_plugin(&Prometheus, JsonSerializer)
}
