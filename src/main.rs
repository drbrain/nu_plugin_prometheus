mod client;
mod prometheus;
mod query;
mod source;

use client::Client;
use nu_plugin::{serve_plugin, JsonSerializer};
use prometheus::Prometheus;

fn main() {
    serve_plugin(&Prometheus, JsonSerializer)
}
