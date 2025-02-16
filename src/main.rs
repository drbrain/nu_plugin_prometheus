mod client;
mod prometheus;
mod query;
mod source;

use client::Client;
use nu_plugin::{serve_plugin, MsgPackSerializer};
use prometheus::Prometheus;
use query::Query;
use source::Source;

fn main() {
    serve_plugin(&Prometheus, MsgPackSerializer)
}
