// The connection's side of plugin channels: the lists each side tells the other on the game's own channels, and the
// players on their way from play back to configuration (Paper's reconfiguration), who keep their connection.
use std::sync::Mutex;

/// The channel that carries a side's list of channels it listens on, and its opposite: the game's own, which any
/// client reads, whatever it runs on.
pub const REGISTER: &str = "minecraft:register";
pub const UNREGISTER: &str = "minecraft:unregister";

/// `minecraft:register`'s format: the names, each ended by a zero byte.
pub fn channel_list(channels: impl IntoIterator<Item = String>) -> Vec<u8> {
    let mut data = Vec::new();
    for channel in channels {
        data.extend_from_slice(channel.as_bytes());
        data.push(0);
    }
    data
}

pub fn read_channels(data: &[u8]) -> Vec<String> {
    data.split(|b| *b == 0).filter(|name| !name.is_empty()).map(|name| String::from_utf8_lossy(name).into_owned()).collect()
}

/// Players on their way from play to configuration: the connection stays, so it isn't a quit.
static TO_CONFIGURATION: Mutex<Vec<u128>> = Mutex::new(Vec::new());

pub fn leaves_for_configuration(uuid: u128) {
    TO_CONFIGURATION.lock().unwrap_or_else(|e| e.into_inner()).push(uuid);
}

/// Whether this leaving was that: once.
pub fn left_for_configuration(uuid: u128) -> bool {
    let mut leaving = TO_CONFIGURATION.lock().unwrap_or_else(|e| e.into_inner());
    match leaving.iter().position(|known| *known == uuid) {
        Some(at) => {
            leaving.remove(at);
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_channel_list_reads_back() {
        let data = channel_list(["bungeecord:main".to_owned(), "plugin:x".to_owned()]);
        assert_eq!(read_channels(&data), ["bungeecord:main", "plugin:x"]);
    }
}
