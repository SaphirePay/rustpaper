//! Small classes of the API: colors of dyes, transformations, number helpers, inputs.
use crate::entities::{Ageable, Display};
use crate::meta_lists::FireworkEffect;
use crate::{Block, Color, Location, Player};
use serde_json::{Value, json};

impl Ageable {
    /// bukkit: org/bukkit/entity/Ageable.setBaby
    pub fn set_baby(&self) -> bool {
        self.set_age(-24000)
    }
    /// bukkit: org/bukkit/entity/Ageable.setAdult
    pub fn set_adult(&self) -> bool {
        self.set_age(0)
    }
    /// bukkit: org/bukkit/entity/Ageable.isAdult
    pub fn is_adult(&self) -> bool {
        self.get_age() >= 0
    }
    /// bukkit: org/bukkit/entity/Ageable.canBreed, org/bukkit/entity/Breedable.canBreed
    pub fn can_breed(&self) -> bool {
        self.get_age() == 0
    }
    /// bukkit: org/bukkit/entity/Ageable.setBreed, org/bukkit/entity/Breedable.setBreed
    pub fn set_breed(&self, can: bool) -> bool {
        self.set_age(if can { 0 } else { 6000 })
    }
}

const DYES: [(&str, u32, u32); 16] = [
    ("WHITE", 0xF9FFFE, 0xF0F0F0),
    ("ORANGE", 0xF9801D, 0xEB8844),
    ("MAGENTA", 0xC74EBD, 0xC354CD),
    ("LIGHT_BLUE", 0x3AB3DA, 0x6689D3),
    ("YELLOW", 0xFED83D, 0xDECF2A),
    ("LIME", 0x80C71F, 0x41CD34),
    ("PINK", 0xF38BAA, 0xD88198),
    ("GRAY", 0x474F52, 0x434343),
    ("LIGHT_GRAY", 0x9D9D97, 0xABABAB),
    ("CYAN", 0x169C9C, 0x287697),
    ("PURPLE", 0x8932B8, 0x7B2FBE),
    ("BLUE", 0x3C44AA, 0x253192),
    ("BROWN", 0x835432, 0x51301A),
    ("GREEN", 0x5E7C16, 0x3B511A),
    ("RED", 0xB02E26, 0xB3312C),
    ("BLACK", 0x1D1D21, 0x1E1B1B),
];

fn color_of(rgb: u32) -> Color {
    Color::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// bukkit: org/bukkit/DyeColor.getColor
pub fn dye_color(dye: &str) -> Option<Color> {
    DYES.iter().find(|(name, ..)| *name == dye).map(|(_, rgb, _)| color_of(*rgb))
}

/// bukkit: org/bukkit/DyeColor.getFireworkColor
pub fn dye_firework_color(dye: &str) -> Option<Color> {
    DYES.iter().find(|(name, ..)| *name == dye).map(|(_, _, rgb)| color_of(*rgb))
}

/// bukkit: org/bukkit/DyeColor.getByColor
pub fn dye_by_color(color: &Color) -> Option<&'static str> {
    DYES.iter().find(|(_, rgb, _)| *rgb == color.as_rgb()).map(|(name, ..)| *name)
}

/// bukkit: org/bukkit/DyeColor.getByFireworkColor
pub fn dye_by_firework_color(color: &Color) -> Option<&'static str> {
    DYES.iter().find(|(_, _, rgb)| *rgb == color.as_rgb()).map(|(name, ..)| *name)
}

impl FireworkEffect {
    fn colors_at(&self, key: &str) -> Vec<Color> {
        self.saved_value()[key].as_array().into_iter().flatten().filter_map(Value::as_u64).map(|rgb| color_of(rgb as u32)).collect()
    }
    /// bukkit: org/bukkit/FireworkEffect.getColors
    pub fn get_colors(&self) -> Vec<Color> {
        self.colors_at("colors")
    }
    /// bukkit: org/bukkit/FireworkEffect.getFadeColors
    pub fn get_fade_colors(&self) -> Vec<Color> {
        self.colors_at("fade_colors")
    }
}

/// `org.bukkit.util.Transformation`: where a display entity puts what it shows.
#[derive(Clone, Debug, PartialEq)]
pub struct Transformation {
    pub translation: [f32; 3],
    pub left_rotation: [f32; 4],
    pub scale: [f32; 3],
    pub right_rotation: [f32; 4],
}

impl Display {
    /// bukkit: org/bukkit/entity/Display.getTransformation
    pub fn get_transformation(&self) -> Transformation {
        let saved = self.data_at("transformation");
        let list = |key: &str, i: usize, default: f32| saved[key][i].as_f64().map_or(default, |value| value as f32);
        Transformation {
            translation: [0, 1, 2].map(|i| list("translation", i, 0.0)),
            left_rotation: [0, 1, 2, 3].map(|i| list("left_rotation", i, if i == 3 { 1.0 } else { 0.0 })),
            scale: [0, 1, 2].map(|i| list("scale", i, 1.0)),
            right_rotation: [0, 1, 2, 3].map(|i| list("right_rotation", i, if i == 3 { 1.0 } else { 0.0 })),
        }
    }
    /// bukkit: org/bukkit/entity/Display.setTransformation
    pub fn set_transformation(&self, at: &Transformation) -> bool {
        let floats = |values: &[f32]| values.iter().map(|value| format!("{value:?}f")).collect::<Vec<_>>().join(",");
        self.merge(&format!("{{transformation:{{translation:[{}],left_rotation:[{}],scale:[{}],right_rotation:[{}]}}}}", floats(&at.translation), floats(&at.left_rotation), floats(&at.scale), floats(&at.right_rotation)))
    }
}

impl Transformation {
    /// bukkit: org/bukkit/util/Transformation.getTranslation
    pub fn get_translation(&self) -> [f32; 3] {
        self.translation
    }
    /// bukkit: org/bukkit/util/Transformation.getLeftRotation
    pub fn get_left_rotation(&self) -> [f32; 4] {
        self.left_rotation
    }
    /// bukkit: org/bukkit/util/Transformation.getScale
    pub fn get_scale(&self) -> [f32; 3] {
        self.scale
    }
    /// bukkit: org/bukkit/util/Transformation.getRightRotation
    pub fn get_right_rotation(&self) -> [f32; 4] {
        self.right_rotation
    }
}

/// bukkit: org/bukkit/util/NumberConversions.floor
pub fn floor(value: f64) -> i32 {
    value.floor() as i32
}

/// bukkit: org/bukkit/util/NumberConversions.ceil
pub fn ceil(value: f64) -> i32 {
    value.ceil() as i32
}

/// bukkit: org/bukkit/util/NumberConversions.round
pub fn round(value: f64) -> i32 {
    floor(value + 0.5)
}

/// bukkit: org/bukkit/util/NumberConversions.square
pub fn square(value: f64) -> f64 {
    value * value
}

/// bukkit: org/bukkit/util/NumberConversions.isFinite
pub fn is_finite(value: f64) -> bool {
    value.is_finite()
}

/// bukkit: org/bukkit/util/NumberConversions.toInt
pub fn to_int(value: &Value) -> i32 {
    value.as_i64().map_or_else(|| value.as_f64().map_or_else(|| value.as_str().and_then(|text| text.parse().ok()).unwrap_or(0), |number| number as i32), |number| number as i32)
}

/// bukkit: org/bukkit/util/NumberConversions.toDouble
pub fn to_double(value: &Value) -> f64 {
    value.as_f64().or_else(|| value.as_str().and_then(|text| text.parse().ok())).unwrap_or(0.0)
}

/// bukkit: org/bukkit/util/NumberConversions.toLong
pub fn to_long(value: &Value) -> i64 {
    value.as_i64().or_else(|| value.as_str().and_then(|text| text.parse().ok())).unwrap_or(0)
}

/// `org.bukkit.Input`: the keys a player holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Input(u8);

impl Player {
    /// bukkit: org/bukkit/entity/Player.getCurrentInput
    pub fn get_current_input(&self) -> Input {
        Input(self.player_state()["sprint_input"].as_u64().unwrap_or(0) as u8)
    }
}

impl Input {
    /// bukkit: org/bukkit/Input.isForward
    pub fn is_forward(self) -> bool {
        self.0 & 1 != 0
    }
    /// bukkit: org/bukkit/Input.isBackward
    pub fn is_backward(self) -> bool {
        self.0 & 2 != 0
    }
    /// bukkit: org/bukkit/Input.isLeft
    pub fn is_left(self) -> bool {
        self.0 & 4 != 0
    }
    /// bukkit: org/bukkit/Input.isRight
    pub fn is_right(self) -> bool {
        self.0 & 8 != 0
    }
    /// bukkit: org/bukkit/Input.isJump
    pub fn is_jump(self) -> bool {
        self.0 & 16 != 0
    }
    /// bukkit: org/bukkit/Input.isSneak
    pub fn is_sneak(self) -> bool {
        self.0 & 32 != 0
    }
    /// bukkit: org/bukkit/Input.isSprint
    pub fn is_sprint(self) -> bool {
        self.0 & 64 != 0
    }
}

/// `org.bukkit.util.BlockIterator`: the blocks along a ray, a step at a time.
pub struct BlockIterator {
    at: crate::util::Vector,
    step: crate::util::Vector,
    world: crate::World,
    left: f64,
    last: Option<(i32, i32, i32)>,
}

impl BlockIterator {
    /// bukkit: org/bukkit/util/BlockIterator.<init>
    pub fn new(start: &Location, distance: f64) -> Self {
        let mut step = start.get_direction();
        step.multiply(0.1);
        Self { at: start.to_vector(), step, world: start.get_world().clone(), left: distance, last: None }
    }
}

/// bukkit: org/bukkit/util/BlockIterator.next, org/bukkit/util/BlockIterator.hasNext
impl Iterator for BlockIterator {
    type Item = Block;
    fn next(&mut self) -> Option<Block> {
        while self.left > 0.0 {
            let cell = (self.at.get_block_x(), self.at.get_block_y(), self.at.get_block_z());
            self.at.add(&self.step);
            self.left -= 0.1;
            if self.last != Some(cell) {
                self.last = Some(cell);
                return Some(self.world.get_block_at(cell.0, cell.1, cell.2));
            }
        }
        None
    }
}

/// `com.destroystokyo.paper.profile.ProfileProperty`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

impl ProfileProperty {
    /// bukkit: com/destroystokyo/paper/profile/ProfileProperty.<init>
    pub fn new(name: &str, value: &str, signature: Option<&str>) -> Self {
        Self { name: name.to_owned(), value: value.to_owned(), signature: signature.map(str::to_owned) }
    }
    /// bukkit: com/destroystokyo/paper/profile/ProfileProperty.getName
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// bukkit: com/destroystokyo/paper/profile/ProfileProperty.getValue
    pub fn get_value(&self) -> &str {
        &self.value
    }
    /// bukkit: com/destroystokyo/paper/profile/ProfileProperty.getSignature
    pub fn get_signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }
    /// bukkit: com/destroystokyo/paper/profile/ProfileProperty.isSigned
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

/// bukkit: org/bukkit/GameMode.getValue
pub fn game_mode_value(mode: &str) -> i32 {
    crate::enums::game_mode::values().iter().position(|known| *known == mode).map_or(0, |index| match index {
        0 => 1,
        1 => 0,
        other => other as i32,
    })
}

/// bukkit: org/bukkit/GameMode.getByValue
pub fn game_mode_by_value(value: i32) -> Option<&'static str> {
    crate::enums::game_mode::values().iter().copied().find(|mode| game_mode_value(mode) == value)
}

/// bukkit: org/bukkit/Difficulty.getValue
pub fn difficulty_value(difficulty: &str) -> i32 {
    crate::enums::difficulty::values().iter().position(|known| *known == difficulty).map_or(0, |index| index as i32)
}

/// bukkit: org/bukkit/Difficulty.getByValue
pub fn difficulty_by_value(value: usize) -> Option<&'static str> {
    crate::enums::difficulty::values().get(value).copied()
}

/// bukkit: io/papermc/paper/ServerBuildInfo.buildInfo, io/papermc/paper/ServerBuildInfo.brandName, io/papermc/paper/ServerBuildInfo.minecraftVersionId
pub fn build_info() -> Value {
    json!({ "brandName": crate::get_name(), "minecraftVersionId": crate::server::get_minecraft_version() })
}

/// bukkit: org/bukkit/scheduler/BukkitTask.getTaskId
pub fn task_id(task: u32) -> u32 {
    task
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskLaterAsynchronously
pub fn run_task_later_asynchronously(delay: u64, task: impl FnOnce() + Send + 'static) {
    crate::scheduler::run_task_asynchronously(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay * 50));
        task();
    });
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskTimerAsynchronously
pub fn run_task_timer_asynchronously(delay: u64, period: u64, mut task: impl FnMut() -> bool + Send + 'static) {
    crate::scheduler::run_task_asynchronously(move || {
        std::thread::sleep(std::time::Duration::from_millis(delay * 50));
        while task() {
            std::thread::sleep(std::time::Duration::from_millis(period * 50));
        }
    });
}
