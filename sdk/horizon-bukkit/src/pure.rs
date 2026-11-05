//! Bukkit's classes that need no game: positions, noise, chat paging, conversations; and spawners,
//! which are a block's saved data.
#![allow(missing_docs)]
use crate::{Block, Plugin, Player};
use horizon_mod::{Answer, Value, query};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// ---- Positions ----

/// `io.papermc.paper.math.Position`, `FinePosition` and `BlockPosition`: one type, as Rust has no
/// need of the split; `is_block` says which it was made as.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
    block: bool,
}

impl Position {
    /// bukkit: io/papermc/paper/math/Position.block
    pub fn block(x: i32, y: i32, z: i32) -> Self {
        Self { x: f64::from(x), y: f64::from(y), z: f64::from(z), block: true }
    }
    /// bukkit: io/papermc/paper/math/Position.fine
    pub fn fine(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, block: false }
    }
    /// bukkit: io/papermc/paper/math/Position.blockX, io/papermc/paper/math/FinePosition.blockX
    pub fn block_x(&self) -> i32 {
        self.x.floor() as i32
    }
    /// bukkit: io/papermc/paper/math/Position.blockY, io/papermc/paper/math/FinePosition.blockY
    pub fn block_y(&self) -> i32 {
        self.y.floor() as i32
    }
    /// bukkit: io/papermc/paper/math/Position.blockZ, io/papermc/paper/math/FinePosition.blockZ
    pub fn block_z(&self) -> i32 {
        self.z.floor() as i32
    }
    /// bukkit: io/papermc/paper/math/Position.x, io/papermc/paper/math/BlockPosition.x
    pub fn x(&self) -> f64 {
        self.x
    }
    /// bukkit: io/papermc/paper/math/Position.y, io/papermc/paper/math/BlockPosition.y
    pub fn y(&self) -> f64 {
        self.y
    }
    /// bukkit: io/papermc/paper/math/Position.z, io/papermc/paper/math/BlockPosition.z
    pub fn z(&self) -> f64 {
        self.z
    }
    /// bukkit: io/papermc/paper/math/Position.isBlock, io/papermc/paper/math/BlockPosition.isBlock, io/papermc/paper/math/FinePosition.isBlock
    pub fn is_block(&self) -> bool {
        self.block
    }
    /// bukkit: io/papermc/paper/math/Position.isFine, io/papermc/paper/math/BlockPosition.isFine, io/papermc/paper/math/FinePosition.isFine
    pub fn is_fine(&self) -> bool {
        !self.block
    }
    /// bukkit: io/papermc/paper/math/Position.isFinite
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
    /// bukkit: io/papermc/paper/math/Position.offset, io/papermc/paper/math/BlockPosition.offset, io/papermc/paper/math/FinePosition.offset
    pub fn offset(&self, x: i32, y: i32, z: i32) -> Self {
        Self { x: self.x + f64::from(x), y: self.y + f64::from(y), z: self.z + f64::from(z), block: self.block }
    }
    /// bukkit: io/papermc/paper/math/Position.toBlock, io/papermc/paper/math/BlockPosition.toBlock, io/papermc/paper/math/FinePosition.toBlock
    pub fn to_block(&self) -> Self {
        Self::block(self.block_x(), self.block_y(), self.block_z())
    }
    /// bukkit: io/papermc/paper/math/Position.toCenter
    pub fn to_center(&self) -> Self {
        Self::fine(f64::from(self.block_x()) + 0.5, f64::from(self.block_y()) + 0.5, f64::from(self.block_z()) + 0.5)
    }
}

/// `io.papermc.paper.math.Rotation`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl Rotation {
    /// bukkit: io/papermc/paper/math/Rotation.rotation
    pub fn rotation(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }
    /// bukkit: io/papermc/paper/math/Rotation.yaw
    pub fn yaw(&self) -> f32 {
        self.yaw
    }
    /// bukkit: io/papermc/paper/math/Rotation.pitch
    pub fn pitch(&self) -> f32 {
        self.pitch
    }
}

/// `io.papermc.paper.math.Rotations`: an armor stand's limb, in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotations {
    x: f64,
    y: f64,
    z: f64,
}

impl Rotations {
    /// bukkit: io/papermc/paper/math/Rotations.ofDegrees
    pub fn of_degrees(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    /// bukkit: io/papermc/paper/math/Rotations.x
    pub fn x(&self) -> f64 {
        self.x
    }
    /// bukkit: io/papermc/paper/math/Rotations.y
    pub fn y(&self) -> f64 {
        self.y
    }
    /// bukkit: io/papermc/paper/math/Rotations.z
    pub fn z(&self) -> f64 {
        self.z
    }
    /// bukkit: io/papermc/paper/math/Rotations.withX
    pub fn with_x(self, x: f64) -> Self {
        Self { x, ..self }
    }
    /// bukkit: io/papermc/paper/math/Rotations.withY
    pub fn with_y(self, y: f64) -> Self {
        Self { y, ..self }
    }
    /// bukkit: io/papermc/paper/math/Rotations.withZ
    pub fn with_z(self, z: f64) -> Self {
        Self { z, ..self }
    }
    /// bukkit: io/papermc/paper/math/Rotations.add
    pub fn add(self, x: f64, y: f64, z: f64) -> Self {
        Self { x: self.x + x, y: self.y + y, z: self.z + z }
    }
    /// bukkit: io/papermc/paper/math/Rotations.subtract
    pub fn subtract(self, x: f64, y: f64, z: f64) -> Self {
        self.add(-x, -y, -z)
    }
}

/// `io.papermc.paper.math.Angle`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Angle {
    degrees: f32,
    relative: bool,
}

impl Angle {
    /// bukkit: io/papermc/paper/math/Angle.absolute
    pub fn absolute(degrees: f32) -> Self {
        Self { degrees, relative: false }
    }
    /// bukkit: io/papermc/paper/math/Angle.relative
    pub fn relative(degrees: f32) -> Self {
        Self { degrees, relative: true }
    }
    /// bukkit: io/papermc/paper/math/Angle.degrees
    pub fn degrees(&self) -> f32 {
        self.degrees
    }
}

// ---- Noise ----

/// The permutation of Ken Perlin's reference noise.
pub(crate) const PERMUTATION: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

pub(crate) fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub(crate) fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

pub(crate) fn grad(hash: u8, x: f64, y: f64, z: f64) -> f64 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

/// bukkit: org/bukkit/util/noise/NoiseGenerator.floor
pub fn noise_floor(x: f64) -> i32 {
    x.floor() as i32
}

/// `org.bukkit.util.noise.PerlinNoiseGenerator`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PerlinNoiseGenerator;

impl PerlinNoiseGenerator {
    /// bukkit: org/bukkit/util/noise/PerlinNoiseGenerator.getInstance, org/bukkit/util/noise/NoiseGenerator.noise
    pub fn get_instance() -> Self {
        Self
    }
    /// bukkit: org/bukkit/util/noise/PerlinNoiseGenerator.getNoise, org/bukkit/util/noise/PerlinNoiseGenerator.noise
    pub fn noise(x: f64, y: f64, z: f64) -> f64 {
        let at = |i: usize| PERMUTATION[i & 255];
        let (fx, fy, fz) = (x.floor(), y.floor(), z.floor());
        let (ix, iy, iz) = (fx as i64 as usize, fy as i64 as usize, fz as i64 as usize);
        let (x, y, z) = (x - fx, y - fy, z - fz);
        let (u, v, w) = (fade(x), fade(y), fade(z));
        let a = usize::from(at(ix)).wrapping_add(iy);
        let (aa, ab) = (usize::from(at(a)).wrapping_add(iz), usize::from(at(a.wrapping_add(1))).wrapping_add(iz));
        let b = usize::from(at(ix.wrapping_add(1))).wrapping_add(iy);
        let (ba, bb) = (usize::from(at(b)).wrapping_add(iz), usize::from(at(b.wrapping_add(1))).wrapping_add(iz));
        lerp(
            w,
            lerp(v, lerp(u, grad(at(aa), x, y, z), grad(at(ba), x - 1.0, y, z)), lerp(u, grad(at(ab), x, y - 1.0, z), grad(at(bb), x - 1.0, y - 1.0, z))),
            lerp(v, lerp(u, grad(at(aa.wrapping_add(1)), x, y, z - 1.0), grad(at(ba.wrapping_add(1)), x - 1.0, y, z - 1.0)), lerp(u, grad(at(ab.wrapping_add(1)), x, y - 1.0, z - 1.0), grad(at(bb.wrapping_add(1)), x - 1.0, y - 1.0, z - 1.0))),
        )
    }
}

/// `org.bukkit.util.noise.OctaveGenerator`: octaves of Perlin noise at a scale per axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OctaveGenerator {
    pub(crate) octaves: i32,
    x_scale: f64,
    y_scale: f64,
    z_scale: f64,
}

impl OctaveGenerator {
    pub fn new(octaves: i32) -> Self {
        Self { octaves, x_scale: 1.0, y_scale: 1.0, z_scale: 1.0 }
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.setScale
    pub fn set_scale(&mut self, scale: f64) {
        (self.x_scale, self.y_scale, self.z_scale) = (scale, scale, scale);
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.getXScale
    pub fn get_x_scale(&self) -> f64 {
        self.x_scale
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.setXScale
    pub fn set_x_scale(&mut self, scale: f64) {
        self.x_scale = scale;
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.getYScale
    pub fn get_y_scale(&self) -> f64 {
        self.y_scale
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.setYScale
    pub fn set_y_scale(&mut self, scale: f64) {
        self.y_scale = scale;
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.getZScale
    pub fn get_z_scale(&self) -> f64 {
        self.z_scale
    }
    /// bukkit: org/bukkit/util/noise/OctaveGenerator.setZScale
    pub fn set_z_scale(&mut self, scale: f64) {
        self.z_scale = scale;
    }
    /// Every octave on Perlin's table: `fam_core::PerlinOctaveGenerator` is Java's, seeded.
    pub fn noise(&self, x: f64, y: f64, z: f64, frequency: f64, amplitude: f64) -> f64 {
        let (mut total, mut freq, mut amp) = (0.0, 1.0, 1.0);
        for _ in 0..self.octaves {
            total += PerlinNoiseGenerator::noise(x * self.x_scale * freq, y * self.y_scale * freq, z * self.z_scale * freq) * amp;
            freq *= frequency;
            amp *= amplitude;
        }
        total
    }
}

// ---- Chat paging ----

/// `org.bukkit.util.ChatPaginator$ChatPage`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatPage {
    pub lines: Vec<String>,
    pub page_number: usize,
    pub total_pages: usize,
}

impl ChatPage {
    /// bukkit: org/bukkit/util/ChatPaginator$ChatPage.getLines
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }
    /// bukkit: org/bukkit/util/ChatPaginator$ChatPage.getPageNumber
    pub fn get_page_number(&self) -> usize {
        self.page_number
    }
    /// bukkit: org/bukkit/util/ChatPaginator$ChatPage.getTotalPages
    pub fn get_total_pages(&self) -> usize {
        self.total_pages
    }
}

/// bukkit: org/bukkit/util/ChatPaginator.wordWrap
pub fn word_wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split(' ') {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}

/// bukkit: org/bukkit/util/ChatPaginator.paginate
pub fn paginate(text: &str, page: usize) -> ChatPage {
    const WIDTH: usize = 55;
    const HEIGHT: usize = 9;
    let lines = word_wrap(text, WIDTH);
    let total_pages = lines.len().div_ceil(HEIGHT).max(1);
    let page_number = page.clamp(1, total_pages);
    ChatPage { lines: lines.into_iter().skip((page_number - 1) * HEIGHT).take(HEIGHT).collect(), page_number, total_pages }
}

/// bukkit: org/bukkit/util/NumberConversions.toFloat
pub fn to_float(value: &Value) -> f32 {
    value.as_f64().or_else(|| value.as_str()?.trim().parse().ok()).unwrap_or(0.0) as f32
}

/// bukkit: org/bukkit/util/NumberConversions.toShort
pub fn to_short(value: &Value) -> i16 {
    value.as_i64().or_else(|| value.as_str()?.trim().parse().ok()).unwrap_or(0) as i16
}

/// bukkit: org/bukkit/util/NumberConversions.toByte
pub fn to_byte(value: &Value) -> i8 {
    value.as_i64().or_else(|| value.as_str()?.trim().parse().ok()).unwrap_or(0) as i8
}

/// bukkit: org/bukkit/util/FileUtil.copy
pub fn file_copy(from: &std::path::Path, to: &std::path::Path) -> bool {
    std::fs::copy(from, to).is_ok()
}

// ---- Conversations ----

/// `org.bukkit.conversations.ConversationContext`.
#[derive(Default)]
pub struct ConversationContext {
    data: HashMap<String, String>,
}

impl ConversationContext {
    /// bukkit: org/bukkit/conversations/ConversationContext.getSessionData
    pub fn get_session_data(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(String::as_str)
    }
    /// bukkit: org/bukkit/conversations/ConversationContext.setSessionData
    pub fn set_session_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_owned(), value.to_owned());
    }
    /// bukkit: org/bukkit/conversations/ConversationContext.getAllSessionData
    pub fn get_all_session_data(&self) -> &HashMap<String, String> {
        &self.data
    }
}

/// `org.bukkit.conversations.Prompt`: `None` from `accept_input` ends the conversation.
pub trait Prompt: Send + Sync {
    /// bukkit: org/bukkit/conversations/Prompt.getPromptText
    fn get_prompt_text(&self, context: &ConversationContext) -> String;
    /// bukkit: org/bukkit/conversations/Prompt.blocksForInput, org/bukkit/conversations/StringPrompt.blocksForInput
    fn blocks_for_input(&self, _context: &ConversationContext) -> bool {
        true
    }
    /// bukkit: org/bukkit/conversations/Prompt.acceptInput
    fn accept_input(&self, context: &mut ConversationContext, input: &str) -> Option<Box<dyn Prompt>>;
}

/// bukkit: org/bukkit/conversations/ConversationFactory.<init>, org/bukkit/conversations/ConversationFactory.withModality, org/bukkit/conversations/ConversationFactory.withLocalEcho, org/bukkit/conversations/Conversation.begin
///
/// `org.bukkit.conversations.ConversationFactory` and `Conversation`.
pub struct Conversation {
    prompt: Box<dyn Prompt>,
    context: ConversationContext,
    prefix: String,
    escape: Option<String>,
    modal: bool,
}

static TALKING: Mutex<Option<HashMap<String, Conversation>>> = Mutex::new(None);
static LISTENING: OnceLock<()> = OnceLock::new();

impl Conversation {
    pub fn new(first: Box<dyn Prompt>) -> Self {
        Self { prompt: first, context: ConversationContext::default(), prefix: String::new(), escape: None, modal: true }
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.withPrefix
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        prefix.clone_into(&mut self.prefix);
        self
    }
    /// bukkit: org/bukkit/conversations/ConversationFactory.withEscapeSequence, org/bukkit/conversations/ExactMatchConversationCanceller.cancelBasedOnInput
    pub fn with_escape_sequence(mut self, text: &str) -> Self {
        self.escape = Some(text.to_owned());
        self
    }
    /// bukkit: org/bukkit/conversations/Conversation.isModal
    pub fn is_modal(&self) -> bool {
        self.modal
    }
    fn show(&self, player: &Player) {
        player.send_message(&format!("{}{}", self.prefix, self.prompt.get_prompt_text(&self.context)));
    }
}

impl Plugin {
    /// bukkit: org/bukkit/conversations/Conversable.beginConversation
    ///
    /// What the player types next goes to the prompt, not to chat.
    pub fn begin_conversation(&self, player: &Player, conversation: Conversation) {
        conversation.show(player);
        TALKING.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(player.uuid().to_owned(), conversation);
        LISTENING.get_or_init(|| {
            self.host.on("allow_chat", |value| {
                let (Some(uuid), Some(message)) = (value["uuid"].as_str(), value["message"].as_str()) else {
                    return Answer::Pass;
                };
                let taken = TALKING.lock().unwrap_or_else(|e| e.into_inner()).as_mut().and_then(|all| all.remove(uuid));
                let Some(mut talk) = taken else {
                    return Answer::Pass;
                };
                if talk.escape.as_deref() == Some(message) {
                    return Answer::Fail;
                }
                if let Some(next) = talk.prompt.accept_input(&mut talk.context, message) {
                    talk.prompt = next;
                    if let Some(player) = Player::read(&query(json!({ "op": "player", "uuid": uuid }))) {
                        talk.show(&player);
                    }
                    TALKING.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_default().insert(uuid.to_owned(), talk);
                }
                Answer::Fail
            });
        });
    }
}

impl Player {
    /// bukkit: org/bukkit/conversations/Conversable.isConversing
    pub fn is_conversing(&self) -> bool {
        TALKING.lock().unwrap_or_else(|e| e.into_inner()).as_ref().is_some_and(|all| all.contains_key(self.uuid()))
    }
    /// bukkit: org/bukkit/conversations/Conversable.abandonConversation
    pub fn abandon_conversation(&self) {
        if let Some(all) = TALKING.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            all.remove(self.uuid());
        }
    }
}

// ---- Spawners ----

/// bukkit: org/bukkit/spawner/BaseSpawner.getDelay, org/bukkit/spawner/BaseSpawner.setDelay, org/bukkit/spawner/BaseSpawner.getRequiredPlayerRange, org/bukkit/spawner/BaseSpawner.setRequiredPlayerRange, org/bukkit/spawner/BaseSpawner.getSpawnRange, org/bukkit/spawner/BaseSpawner.setSpawnRange, org/bukkit/spawner/Spawner.getMinSpawnDelay, org/bukkit/spawner/Spawner.setMinSpawnDelay, org/bukkit/spawner/Spawner.getMaxSpawnDelay, org/bukkit/spawner/Spawner.setMaxSpawnDelay, org/bukkit/spawner/Spawner.getSpawnCount, org/bukkit/spawner/Spawner.setSpawnCount, org/bukkit/spawner/Spawner.getMaxNearbyEntities, org/bukkit/spawner/Spawner.setMaxNearbyEntities
///
/// `org.bukkit.spawner.Spawner` and `BaseSpawner`: a spawner block's saved data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spawner(pub Block);

macro_rules! spawner_numbers {
    ($(($getter:ident, $setter:ident, $key:literal)),* $(,)?) => {
        impl Spawner {
            fn data(&self) -> Value {
                query(json!({ "op": "block_data", "dimension": self.0.get_world().dimension(), "pos": [self.0.get_x(), self.0.get_y(), self.0.get_z()] }))
            }
            fn set(&self, key: &str, value: i32) {
                query(json!({ "op": "block_merge", "dimension": self.0.get_world().dimension(), "pos": [self.0.get_x(), self.0.get_y(), self.0.get_z()], "snbt": format!("{{{key}:{value}s}}") }));
            }
            $(
                pub fn $getter(&self) -> i64 {
                    self.data()[$key].as_i64().unwrap_or(0)
                }
                pub fn $setter(&self, value: i32) {
                    self.set($key, value);
                }
            )*
        }
    };
}

spawner_numbers!(
    (get_delay, set_delay, "Delay"),
    (get_min_spawn_delay, set_min_spawn_delay, "MinSpawnDelay"),
    (get_max_spawn_delay, set_max_spawn_delay, "MaxSpawnDelay"),
    (get_spawn_count, set_spawn_count, "SpawnCount"),
    (get_max_nearby_entities, set_max_nearby_entities, "MaxNearbyEntities"),
    (get_required_player_range, set_required_player_range, "RequiredPlayerRange"),
    (get_spawn_range, set_spawn_range, "SpawnRange"),
);
