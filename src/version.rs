//! The version of this executable, independent of adjacent package files.

pub const INTERNAL_BUILD: &str = env!("CLICKPULSE_INTERNAL_BUILD");
/// The number in parentheses identifies revisions sharing a preview version.
pub const DISPLAY: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CLICKPULSE_INTERNAL_BUILD"),
    ")"
);
