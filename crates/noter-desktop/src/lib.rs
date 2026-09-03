#![forbid(unsafe_code)]

pub mod app;
pub mod persistence;
pub mod preview;
pub mod selection;
pub mod storage;
#[cfg(feature = "preview-webkit")]
pub mod webkit_preview;

pub const APPLICATION_ID: &str = "net.astrazds.Noter";
