//! Sidecar subprocess management.

pub mod client;
pub mod manager;

pub use client::{Sidecar, DEFAULT_REQUEST_TIMEOUT};
pub use manager::SidecarManager;
