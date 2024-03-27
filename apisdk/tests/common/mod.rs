use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    layer::SubscriberExt,
    Registry,
};

mod api;

#[allow(unused)]
pub use api::*;

#[allow(unused)]
// #[cfg(not(target_arch = "wasm32"))]
mod server;

#[allow(unused)]
// #[cfg(not(target_arch = "wasm32"))]
pub use server::*;

// #[allow(unused)]
// #[cfg(target_arch = "wasm32")]
// pub async fn start_server() {
//     println!("Not available in wasm32");
// }

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
