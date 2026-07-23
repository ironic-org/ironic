//! Development inspection UI and composable plugin contracts.
//!
//! # Features
//!
//! - `devtools`: Read-only web UI over compiled modules and routes
//! - `plugins`: Statically linked plugin registration framework

#[cfg(feature = "devtools")]
pub mod devtools;
#[cfg(feature = "plugins")]
pub mod plugins;
