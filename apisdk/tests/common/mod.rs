use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

mod api;
#[allow(unused)]
mod server;

pub use api::*;
pub use server::*;

pub fn init_logger() {
    let registry = Registry::default().with(
        Layer::default()
            .with_ansi(true)
            .without_time()
            .with_writer(std::io::stdout.with_max_level(Level::TRACE)),
    );
    if tracing::subscriber::set_global_default(registry).is_ok() {
        let _ = LogTracer::init();
    }
}
