//! Typed ArcDPS transport; combat interpretation belongs to the addon.

pub use nexus::event::arc::*;
use nexus::event::{Event, RawEventConsume};

pub fn subscribe<T>(event: &Event<T>, callback: RawEventConsume<T>) {
    event.subscribe(callback).revert_on_unload();
}

pub fn replay_roster() {
    REPLAY_SELF_JOIN.raise(&());
    REPLAY_SQUAD_JOIN.raise(&());
}
