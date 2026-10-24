//! Local Nexus infrastructure. Project identity is generated from addon.toml.

pub mod arc;
pub mod gw2;
pub mod language;
pub(crate) mod lifecycle;
pub mod paths;
pub mod project {
    include!(concat!(env!("OUT_DIR"), "/project.rs"));
}
