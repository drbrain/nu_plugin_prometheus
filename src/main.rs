mod client;
mod prometheus;
mod query;
mod source;

use client::Client;
use nu_plugin::{MsgPackSerializer, serve_plugin};
use prometheus::Prometheus;
use source::Source;

fn main() {
    serve_plugin(&Prometheus, MsgPackSerializer)
}
