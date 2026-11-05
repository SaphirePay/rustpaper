// What `Hub` keeps private and Paper lets a plugin do to one client's player list.
use super::{Hub, play};

/// The whole line of the list, to one client: `showPlayer`.
pub fn list_to(hub: &Hub, viewer: u128, target: u128) {
    let Some(entry) = hub.guests.iter().find(|guest| guest.uuid == target).map(|guest| hub.player_info(target, &guest.name, guest.game_mode, 0)) else {
        return;
    };
    if let Some(guest) = hub.guests.iter().find(|guest| guest.uuid == viewer) {
        guest.send(&play::Clientbound::PlayerInfoUpdate(play::PlayerInfoUpdate { actions: Hub::INFO_ACTIONS, entries: vec![entry] }));
    }
}

/// Off one client's list: `hidePlayer`.
pub fn unlist_to(hub: &Hub, viewer: u128, target: u128) {
    if let Some(guest) = hub.guests.iter().find(|guest| guest.uuid == viewer) {
        guest.send(&play::Clientbound::PlayerInfoRemove(play::PlayerInfoRemove { uuids: vec![target] }));
    }
}
