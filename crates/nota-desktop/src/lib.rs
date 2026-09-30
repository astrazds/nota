#![forbid(unsafe_code)]

pub mod fonts;
pub mod selection;
pub mod visual_contract;
#[cfg(feature = "preview-webkit")]
pub mod webkit_preview;

pub const APPLICATION_ID: &str = "net.astrazds.Nota";
