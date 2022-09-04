pub mod api;
pub mod core;
pub mod error;
pub mod model;
pub use directories::ProjectDirs;

pub mod prelude {
    pub use crate::api::get_package_index;
    pub use crate::core::*;
    pub use crate::model::{LocalIndex, LocalMod, Mod};
}
