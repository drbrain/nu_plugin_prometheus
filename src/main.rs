mod client;
mod plugin;
mod query;

use crate::plugin::Plugin;
use nu_plugin::{serve_plugin, JsonSerializer};

fn main() {
    serve_plugin(&mut Plugin::new(), JsonSerializer)
}
