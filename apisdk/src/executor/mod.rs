mod execute;
mod form;
mod macros;

pub use form::*;
pub use macros::*;

/// Internal struct & functions
#[doc(hidden)]
pub mod __internal {
    pub use super::execute::send;
    pub use super::execute::send_form;
    pub use super::execute::send_json;
    pub use super::execute::send_multipart;
    pub use super::execute::send_raw;
    pub use super::execute::send_xml;
    pub use super::execute::RequestConfigurator;
}
