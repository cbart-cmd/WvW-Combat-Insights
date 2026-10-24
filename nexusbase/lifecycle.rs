//! Addon-specific wiring. Infrastructure lives in the sibling base modules.

use crate::{app, i18n, live, ui};
use nexus::{
    event::event_subscribe,
    gui::{RenderType, register_render, render},
    keybind::{keybind_handler, register_keybind_with_string},
    paths::get_addon_dir,
};
use std::ffi::{CStr, c_char, c_void};

/// Datos del evento `EV_LOG_PARSED` de WvW Fight Analysis. `data` apunta a estructuras C++ que no se leen.
#[repr(C)]
struct LogParsedEventArgs {
    filename: *const c_char,
    data: *const c_void,
}

pub(crate) fn load() {
    i18n::initialize();
    app::start(
        get_addon_dir("arcdps")
            .ok()
            .map(|dir| dir.join("arcdps.ini")),
    );
    live::subscribe();
    unsafe {
        event_subscribe!("EV_LOG_PARSED" => LogParsedEventArgs, |args| {
            if let Some(args) = args
                && !args.filename.is_null()
            {
                let name = unsafe { CStr::from_ptr(args.filename) }.to_string_lossy().into_owned();
                app::log_parsed(name);
            }
        })
    }
    .revert_on_unload();
    register_keybind_with_string(
        "KB_FIGHT_ANALYSIS_DANGER_TOGGLE",
        keybind_handler!(|_, is_release| {
            if !is_release {
                ui::toggle();
            }
        }),
        "ALT+SHIFT+P",
    )
    .revert_on_unload();
    register_render(RenderType::Render, render!(|ui| ui::render(ui))).revert_on_unload();
    register_render(RenderType::OptionsRender, render!(|ui| ui::options(ui))).revert_on_unload();
}

pub(crate) fn unload() {
    app::stop();
}
