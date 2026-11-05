// What the port of EssentialsX found the engine couldn't do: hiding a player from another, and a
// player running a command of their own.
use horizon_client::paper_hub as hub;
use crate::game::Game;
use horizon_client::lan::hub::paper_lan_hub as lists;
use serde_json::{Value, json};

fn uuid(value: &Value) -> Option<u128> {
    u128::from_str_radix(value.as_str()?, 16).ok()
}

/// The pairs again, by the entity ids the players have now.
fn sync_ids(game: &Game<'_>) {
    let Some(host) = game.lan.as_ref() else {
        return;
    };
    let id = |player: u128| host.hub.guests.iter().find(|guest| guest.uuid == player).map(|guest| guest.entity_id);
    let pairs = hub::HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).clone();
    hub::set_hidden_ids(pairs.into_iter().filter_map(|(viewer, target)| Some((id(viewer)?, id(target)?))).collect());
}

/// Puts one pair right: the tracker looks at a viewer's pairs again once it has moved.
pub(in crate::game) fn hide_sync(game: &mut Game<'_>, viewer: u128, target: u128, hide: bool) {
    sync_ids(game);
    let Some(host) = game.lan.as_mut() else {
        return;
    };
    if let Some(guest) = host.hub.guests.iter_mut().find(|guest| guest.uuid == viewer) {
        guest.moved = true;
    }
    if hide {
        lists::unlist_to(&host.hub, viewer, target);
    } else {
        lists::list_to(&host.hub, viewer, target);
    }
}

/// A player came: they keep being hidden from whoever hid them.
pub(in crate::game) fn on_join(game: &mut Game<'_>, player: u128) {
    let viewers: Vec<u128> = hub::HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).iter().filter(|pair| pair.1 == player).map(|pair| pair.0).collect();
    if !viewers.is_empty() {
        sync_ids(game);
    }
    for viewer in viewers {
        hide_sync(game, viewer, player, true);
    }
}

/// A player left: what they had hidden goes with them, as it did in their `CraftPlayer`.
pub(in crate::game) fn on_quit(game: &mut Game<'_>, player: u128) {
    hub::HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).retain(|pair| pair.0 != player);
    hub::FORCED_MENUS.lock().unwrap_or_else(|e| e.into_inner()).retain(|menu| menu.0 != player);
    sync_ids(game);
}

/// `Player.performCommand`, as if the player had typed it: their permission level, their place.
pub(in crate::game) fn run_as(game: &mut Game<'_>, player: u128, line: &str) {
    let Some(mut host) = game.lan.take() else {
        return;
    };
    if let Some(conn) = host.hub.guests.iter().find(|guest| guest.uuid == player).map(|guest| guest.conn) {
        game.guest_command(&mut host, conn, line, None);
    }
    game.lan = Some(host);
}

pub(in crate::game) fn query(_game: &mut Game<'_>, op: &str, query: &Value) -> Option<Value> {
    match op {
        "ess_hide" => {
            let (viewer, target) = (uuid(&query["viewer"])?, uuid(&query["target"])?);
            let hide = query["hide"].as_bool().unwrap_or(true);
            let mut pairs = hub::HIDDEN.lock().unwrap_or_else(|e| e.into_inner());
            let known = pairs.contains(&(viewer, target));
            if hide && !known {
                pairs.push((viewer, target));
            } else if !hide {
                pairs.retain(|pair| *pair != (viewer, target));
            }
            drop(pairs);
            if hide != known {
                horizon_mod_runtime_push(json!({ "op": "hide_sync", "player": query["viewer"], "target": query["target"], "hide": hide }));
            }
            Some(json!(hide != known))
        }
        "ess_can_see" => {
            let pair = (uuid(&query["viewer"])?, uuid(&query["target"])?);
            Some(json!(!hub::HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).contains(&pair)))
        }
        "ess_hidden" => {
            let viewer = uuid(&query["viewer"])?;
            Some(json!(hub::HIDDEN.lock().unwrap_or_else(|e| e.into_inner()).iter().filter(|pair| pair.0 == viewer).map(|pair| format!("{:032x}", pair.1)).collect::<Vec<_>>()))
        }
        "ess_run_as" => {
            horizon_mod_runtime_push(json!({ "op": "run_as", "player": query["uuid"], "text": query["line"] }));
            Some(json!(true))
        }
        _ => None,
    }
}

fn horizon_mod_runtime_push(action: Value) {
    super::paper_bridge::push_action(action.to_string());
}
