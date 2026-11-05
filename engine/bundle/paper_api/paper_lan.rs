// What Paper lets a plugin change while the server runs and `lan` keeps private.
use std::sync::atomic::Ordering;

pub fn set_motd(shared: &super::Shared, motd: &str) {
    *shared.motd.lock().unwrap() = motd.to_owned();
}

pub fn set_max_players(shared: &super::Shared, count: usize) {
    shared.max_players.store(count, Ordering::Relaxed);
}
