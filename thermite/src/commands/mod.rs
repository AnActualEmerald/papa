//================
//Command handlers
//================
mod clear;
pub use clear::clear;
mod config;
pub use config::update_config;
mod disable;
pub(crate) use disable::disable;
mod enable;
pub(crate) use enable::enable;
mod exclude;
pub(crate) use exclude::exclude;
mod include;
pub(crate) use include::include;
mod install;
pub(crate) use install::*;
mod list;
pub(crate) use list::list;
mod remove;
pub(crate) use remove::remove;
mod search;
pub(crate) use search::search;
mod update;
pub(crate) use update::update;
#[cfg(feature = "cluster")]
mod cluster;
#[cfg(feature = "cluster")]
pub(crate) use cluster::*;

//=================
//Command utilities
//=================
mod utils;
