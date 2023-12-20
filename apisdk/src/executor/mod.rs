mod execute;
mod form;
mod macros;

pub use form::*;
pub use macros::*;

/// Internal struct & functions
#[doc(hidden)]
pub mod internal {
    pub use super::execute::RequestConfigurator;
    pub use super::execute::_send;
    pub use super::execute::_send_form;
    pub use super::execute::_send_json;
    pub use super::execute::_send_multipart;
    pub use super::execute::_send_raw;
}
