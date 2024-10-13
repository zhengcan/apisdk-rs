#[cfg(feature = "opentelemetry_0_24")]
mod opentelemetry_0_24 {
    pub use opentelemetry_0_24_pkg::trace::*;
    pub use opentelemetry_0_24_pkg::KeyValue;
}

#[cfg(feature = "opentelemetry_0_24")]
pub use opentelemetry_0_24::*;

#[cfg(feature = "opentelemetry_0_25")]
mod opentelemetry_0_25 {
    pub use opentelemetry_0_25_pkg::trace::*;
    pub use opentelemetry_0_25_pkg::KeyValue;
}

#[cfg(feature = "opentelemetry_0_25")]
pub use opentelemetry_0_25::*;

#[cfg(feature = "opentelemetry_0_26")]
mod opentelemetry_0_26 {
    pub use opentelemetry_0_26_pkg::trace::*;
    pub use opentelemetry_0_26_pkg::KeyValue;
}

#[cfg(feature = "opentelemetry_0_26")]
pub use opentelemetry_0_26::*;
