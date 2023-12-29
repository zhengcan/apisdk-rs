use url::Url;

mod resolver;
mod rewriter;

pub use resolver::*;
pub use rewriter::*;

#[cfg(feature = "dns")]
mod hickory;

#[cfg(feature = "dns")]
pub use hickory::*;

/// This trait provides URL related functions
pub trait UrlOps {
    /// Merge path
    fn merge_path(self, path: &str) -> Self;
}

impl UrlOps for Url {
    /// Merge the url and path
    /// - path: relative path
    fn merge_path(mut self, path: &str) -> Self {
        let base_path = self.path();
        let new_path = match (base_path.ends_with('/'), path.starts_with('/')) {
            (true, true) => format!("{}{}", base_path, &path[1..]),
            (true, false) | (false, true) => format!("{}{}", base_path, path),
            (false, false) => format!("{}/{}", base_path, path),
        };
        self.set_path(&new_path);
        self
    }
}
