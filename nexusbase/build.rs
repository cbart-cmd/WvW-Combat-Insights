use std::{env, fs, path::PathBuf};

#[path = "project_config.rs"]
mod project_config;

fn main() {
    println!("cargo:rerun-if-changed=addon.toml");
    println!("cargo:rerun-if-changed=nexusbase/project_config.rs");
    let source = fs::read_to_string("addon.toml").expect("cannot read addon.toml");
    let config = project_config::parse(&source).expect("invalid addon.toml");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR missing"));
    fs::write(out.join("project.rs"), config.constants())
        .expect("cannot generate project constants");
    fs::write(out.join("export.rs"), config.export()).expect("cannot generate Nexus export");
}
