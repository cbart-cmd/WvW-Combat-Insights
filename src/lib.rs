//! WvW Combat Insights for Nexus: team overview, player highlights and squad healing.

#[path = "..\\nexusbase\\mod.rs"]
pub mod nexusbase;

pub mod danger;
pub mod evtc;
pub mod i18n;
mod inflate;
pub mod logs;
pub mod settings;
pub mod zip;

mod app;
mod live;
mod ui;

include!(concat!(env!("OUT_DIR"), "/export.rs"));
