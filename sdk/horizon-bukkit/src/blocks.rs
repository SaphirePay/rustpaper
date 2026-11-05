//! Written by plugin-tools/tools/sdk_blocks.py: do not edit.
#![allow(missing_docs)]
use crate::block_data::{BlockData, DataClass, StateClass, TileState};

/// bukkit: org/bukkit/block/data/Ageable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgeableData(pub BlockData);

impl DataClass for AgeableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl AgeableData {
    /// bukkit: org/bukkit/block/data/Ageable.getAge
    pub fn get_age(&self) -> i32 {
        self.0.property("age").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/Ageable.setAge
    pub fn set_age(&mut self, value: i32) {
        self.0.set_property("age", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/Ageable.getMaximumAge
    pub fn get_maximum_age(&self) -> i32 {
        self.0.maximum_of("age")
    }
}

/// bukkit: org/bukkit/block/data/AnaloguePowerable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnaloguePowerableData(pub BlockData);

impl DataClass for AnaloguePowerableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl AnaloguePowerableData {
    /// bukkit: org/bukkit/block/data/AnaloguePowerable.getPower
    pub fn get_power(&self) -> i32 {
        self.0.property("power").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/AnaloguePowerable.setPower
    pub fn set_power(&mut self, value: i32) {
        self.0.set_property("power", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/AnaloguePowerable.getMaximumPower
    pub fn get_maximum_power(&self) -> i32 {
        self.0.maximum_of("power")
    }
}

/// bukkit: org/bukkit/block/data/Attachable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachableData(pub BlockData);

impl DataClass for AttachableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl AttachableData {
    /// bukkit: org/bukkit/block/data/Attachable.isAttached
    pub fn is_attached(&self) -> bool {
        self.0.property("attached") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/Attachable.setAttached
    pub fn set_attached(&mut self, value: bool) {
        self.0.set_property("attached", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/Bisected
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BisectedData(pub BlockData);

impl DataClass for BisectedData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BisectedData {
    /// bukkit: org/bukkit/block/data/Bisected.getHalf
    pub fn get_half(&self) -> String {
        self.0.property("half").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/Bisected.setHalf
    pub fn set_half(&mut self, value: &str) {
        self.0.set_property("half", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/Brushable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrushableData(pub BlockData);

impl DataClass for BrushableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BrushableData {
    /// bukkit: org/bukkit/block/data/Brushable.getDusted
    pub fn get_dusted(&self) -> i32 {
        self.0.property("dusted").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/Brushable.setDusted
    pub fn set_dusted(&mut self, value: i32) {
        self.0.set_property("dusted", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/Brushable.getMaximumDusted
    pub fn get_maximum_dusted(&self) -> i32 {
        self.0.maximum_of("dusted")
    }
}

/// bukkit: org/bukkit/block/data/Directional
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectionalData(pub BlockData);

impl DataClass for DirectionalData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl DirectionalData {
    /// bukkit: org/bukkit/block/data/Directional.getFacing
    pub fn get_facing(&self) -> String {
        self.0.property("facing").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/Directional.setFacing
    pub fn set_facing(&mut self, value: &str) {
        self.0.set_property("facing", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/FaceAttachable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaceAttachableData(pub BlockData);

impl DataClass for FaceAttachableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl FaceAttachableData {
    /// bukkit: org/bukkit/block/data/FaceAttachable.getAttachedFace
    pub fn get_attached_face(&self) -> String {
        self.0.property("face").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/FaceAttachable.setAttachedFace
    pub fn set_attached_face(&mut self, value: &str) {
        self.0.set_property("face", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/Hangable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HangableData(pub BlockData);

impl DataClass for HangableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl HangableData {
    /// bukkit: org/bukkit/block/data/Hangable.isHanging
    pub fn is_hanging(&self) -> bool {
        self.0.property("hanging") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/Hangable.setHanging
    pub fn set_hanging(&mut self, value: bool) {
        self.0.set_property("hanging", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/Hatchable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HatchableData(pub BlockData);

impl DataClass for HatchableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl HatchableData {
    /// bukkit: org/bukkit/block/data/Hatchable.getHatch
    pub fn get_hatch(&self) -> i32 {
        self.0.property("hatch").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/Hatchable.setHatch
    pub fn set_hatch(&mut self, value: i32) {
        self.0.set_property("hatch", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/Hatchable.getMaximumHatch
    pub fn get_maximum_hatch(&self) -> i32 {
        self.0.maximum_of("hatch")
    }
}

/// bukkit: org/bukkit/block/data/Levelled
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelledData(pub BlockData);

impl DataClass for LevelledData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl LevelledData {
    /// bukkit: org/bukkit/block/data/Levelled.getLevel
    pub fn get_level(&self) -> i32 {
        self.0.property("level").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/Levelled.setLevel
    pub fn set_level(&mut self, value: i32) {
        self.0.set_property("level", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/Levelled.getMaximumLevel
    pub fn get_maximum_level(&self) -> i32 {
        self.0.maximum_of("level")
    }
}

/// bukkit: org/bukkit/block/data/Lightable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightableData(pub BlockData);

impl DataClass for LightableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl LightableData {
    /// bukkit: org/bukkit/block/data/Lightable.isLit
    pub fn is_lit(&self) -> bool {
        self.0.property("lit") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/Lightable.setLit
    pub fn set_lit(&mut self, value: bool) {
        self.0.set_property("lit", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/Openable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenableData(pub BlockData);

impl DataClass for OpenableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl OpenableData {
    /// bukkit: org/bukkit/block/data/Openable.isOpen
    pub fn is_open(&self) -> bool {
        self.0.property("open") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/Openable.setOpen
    pub fn set_open(&mut self, value: bool) {
        self.0.set_property("open", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/Orientable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrientableData(pub BlockData);

impl DataClass for OrientableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl OrientableData {
    /// bukkit: org/bukkit/block/data/Orientable.getAxis
    pub fn get_axis(&self) -> String {
        self.0.property("axis").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/Orientable.setAxis
    pub fn set_axis(&mut self, value: &str) {
        self.0.set_property("axis", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/Powerable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PowerableData(pub BlockData);

impl DataClass for PowerableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl PowerableData {
    /// bukkit: org/bukkit/block/data/Powerable.isPowered
    pub fn is_powered(&self) -> bool {
        self.0.property("powered") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/Powerable.setPowered
    pub fn set_powered(&mut self, value: bool) {
        self.0.set_property("powered", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/Rail
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RailData(pub BlockData);

impl DataClass for RailData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl RailData {
    /// bukkit: org/bukkit/block/data/Rail.getShape
    pub fn get_shape(&self) -> String {
        self.0.property("shape").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/Rail.setShape
    pub fn set_shape(&mut self, value: &str) {
        self.0.set_property("shape", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/Rotatable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RotatableData(pub BlockData);

impl DataClass for RotatableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl RotatableData {
    /// bukkit: org/bukkit/block/data/Rotatable.getRotationNumber
    pub fn get_rotation_number(&self) -> i32 {
        self.0.property("rotation").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/Rotatable.setRotationNumber
    pub fn set_rotation_number(&mut self, value: i32) {
        self.0.set_property("rotation", &value.to_string());
    }
}

/// bukkit: org/bukkit/block/data/Segmentable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentableData(pub BlockData);

impl DataClass for SegmentableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SegmentableData {
    /// bukkit: org/bukkit/block/data/Segmentable.getSegmentAmount
    pub fn get_segment_amount(&self) -> i32 {
        self.0.property("segment_amount").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/Segmentable.setSegmentAmount
    pub fn set_segment_amount(&mut self, value: i32) {
        self.0.set_property("segment_amount", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/Segmentable.getMaximumSegmentAmount
    pub fn get_maximum_segment_amount(&self) -> i32 {
        self.0.maximum_of("segment_amount")
    }
}

/// bukkit: org/bukkit/block/data/SideChaining
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideChainingData(pub BlockData);

impl DataClass for SideChainingData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SideChainingData {
    /// bukkit: org/bukkit/block/data/SideChaining.getSideChain
    pub fn get_side_chain(&self) -> String {
        self.0.property("side_chain").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/SideChaining.setSideChain
    pub fn set_side_chain(&mut self, value: &str) {
        self.0.set_property("side_chain", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/Snowable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnowableData(pub BlockData);

impl DataClass for SnowableData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl SnowableData {
    /// bukkit: org/bukkit/block/data/Snowable.isSnowy
    pub fn is_snowy(&self) -> bool {
        self.0.property("snowy") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/Snowable.setSnowy
    pub fn set_snowy(&mut self, value: bool) {
        self.0.set_property("snowy", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/Waterlogged
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaterloggedData(pub BlockData);

impl DataClass for WaterloggedData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl WaterloggedData {
    /// bukkit: org/bukkit/block/data/Waterlogged.isWaterlogged
    pub fn is_waterlogged(&self) -> bool {
        self.0.property("waterlogged") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/Waterlogged.setWaterlogged
    pub fn set_waterlogged(&mut self, value: bool) {
        self.0.set_property("waterlogged", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Bamboo
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BambooData(pub BlockData);

impl DataClass for BambooData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BambooData {
    /// bukkit: org/bukkit/block/data/type/Bamboo.getLeaves
    pub fn get_leaves(&self) -> String {
        self.0.property("leaves").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Bamboo.setLeaves
    pub fn set_leaves(&mut self, value: &str) {
        self.0.set_property("leaves", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/Bed
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BedData(pub BlockData);

impl DataClass for BedData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BedData {
    /// bukkit: org/bukkit/block/data/type/Bed.getPart
    pub fn get_part(&self) -> String {
        self.0.property("part").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Bed.setPart
    pub fn set_part(&mut self, value: &str) {
        self.0.set_property("part", &value.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/type/Bed.isOccupied
    pub fn is_occupied(&self) -> bool {
        self.0.property("occupied") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Bed.setOccupied
    pub fn set_occupied(&mut self, value: bool) {
        self.0.set_property("occupied", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Beehive
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeehiveData(pub BlockData);

impl DataClass for BeehiveData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BeehiveData {
    /// bukkit: org/bukkit/block/data/type/Beehive.getHoneyLevel
    pub fn get_honey_level(&self) -> i32 {
        self.0.property("honey_level").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Beehive.setHoneyLevel
    pub fn set_honey_level(&mut self, value: i32) {
        self.0.set_property("honey_level", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Beehive.getMaximumHoneyLevel
    pub fn get_maximum_honey_level(&self) -> i32 {
        self.0.maximum_of("honey_level")
    }
}

/// bukkit: org/bukkit/block/data/type/Bell
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BellData(pub BlockData);

impl DataClass for BellData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BellData {
    /// bukkit: org/bukkit/block/data/type/Bell.getAttachment
    pub fn get_attachment(&self) -> String {
        self.0.property("attachment").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Bell.setAttachment
    pub fn set_attachment(&mut self, value: &str) {
        self.0.set_property("attachment", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/BigDripleaf
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigDripleafData(pub BlockData);

impl DataClass for BigDripleafData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BigDripleafData {
    /// bukkit: org/bukkit/block/data/type/BigDripleaf.getTilt
    pub fn get_tilt(&self) -> String {
        self.0.property("tilt").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/BigDripleaf.setTilt
    pub fn set_tilt(&mut self, value: &str) {
        self.0.set_property("tilt", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/BubbleColumn
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BubbleColumnData(pub BlockData);

impl DataClass for BubbleColumnData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl BubbleColumnData {
    /// bukkit: org/bukkit/block/data/type/BubbleColumn.isDrag
    pub fn is_drag(&self) -> bool {
        self.0.property("drag") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/BubbleColumn.setDrag
    pub fn set_drag(&mut self, value: bool) {
        self.0.set_property("drag", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Cake
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CakeData(pub BlockData);

impl DataClass for CakeData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CakeData {
    /// bukkit: org/bukkit/block/data/type/Cake.getBites
    pub fn get_bites(&self) -> i32 {
        self.0.property("bites").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Cake.setBites
    pub fn set_bites(&mut self, value: i32) {
        self.0.set_property("bites", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Cake.getMaximumBites
    pub fn get_maximum_bites(&self) -> i32 {
        self.0.maximum_of("bites")
    }
}

/// bukkit: org/bukkit/block/data/type/Campfire
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampfireData(pub BlockData);

impl DataClass for CampfireData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CampfireData {
    /// bukkit: org/bukkit/block/data/type/Campfire.isSignalFire
    pub fn is_signal_fire(&self) -> bool {
        self.0.property("signal_fire") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Campfire.setSignalFire
    pub fn set_signal_fire(&mut self, value: bool) {
        self.0.set_property("signal_fire", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Candle
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandleData(pub BlockData);

impl DataClass for CandleData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CandleData {
    /// bukkit: org/bukkit/block/data/type/Candle.getCandles
    pub fn get_candles(&self) -> i32 {
        self.0.property("candles").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Candle.setCandles
    pub fn set_candles(&mut self, value: i32) {
        self.0.set_property("candles", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Candle.getMaximumCandles
    pub fn get_maximum_candles(&self) -> i32 {
        self.0.maximum_of("candles")
    }
}

/// bukkit: org/bukkit/block/data/type/CaveVinesPlant
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaveVinesPlantData(pub BlockData);

impl DataClass for CaveVinesPlantData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CaveVinesPlantData {
    /// bukkit: org/bukkit/block/data/type/CaveVinesPlant.isBerries
    pub fn is_berries(&self) -> bool {
        self.0.property("berries") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/CaveVinesPlant.setBerries
    pub fn set_berries(&mut self, value: bool) {
        self.0.set_property("berries", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Chest
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChestData(pub BlockData);

impl DataClass for ChestData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl ChestData {
    /// bukkit: org/bukkit/block/data/type/Chest.getType
    pub fn get_type(&self) -> String {
        self.0.property("type").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Chest.setType
    pub fn set_type(&mut self, value: &str) {
        self.0.set_property("type", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/CommandBlock
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandBlockData(pub BlockData);

impl DataClass for CommandBlockData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CommandBlockData {
    /// bukkit: org/bukkit/block/data/type/CommandBlock.isConditional
    pub fn is_conditional(&self) -> bool {
        self.0.property("conditional") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/CommandBlock.setConditional
    pub fn set_conditional(&mut self, value: bool) {
        self.0.set_property("conditional", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Comparator
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComparatorData(pub BlockData);

impl DataClass for ComparatorData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl ComparatorData {
    /// bukkit: org/bukkit/block/data/type/Comparator.getMode
    pub fn get_mode(&self) -> String {
        self.0.property("mode").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Comparator.setMode
    pub fn set_mode(&mut self, value: &str) {
        self.0.set_property("mode", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/CopperGolemStatue
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopperGolemStatueData(pub BlockData);

impl DataClass for CopperGolemStatueData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CopperGolemStatueData {
    /// bukkit: org/bukkit/block/data/type/CopperGolemStatue.getCopperGolemPose
    pub fn get_copper_golem_pose(&self) -> String {
        self.0.property("copper_golem_pose").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/CopperGolemStatue.setCopperGolemPose
    pub fn set_copper_golem_pose(&mut self, value: &str) {
        self.0.set_property("copper_golem_pose", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/Crafter
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrafterData(pub BlockData);

impl DataClass for CrafterData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CrafterData {
    /// bukkit: org/bukkit/block/data/type/Crafter.isCrafting
    pub fn is_crafting(&self) -> bool {
        self.0.property("crafting") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Crafter.setCrafting
    pub fn set_crafting(&mut self, value: bool) {
        self.0.set_property("crafting", if value { "true" } else { "false" });
    }
    /// bukkit: org/bukkit/block/data/type/Crafter.isTriggered
    pub fn is_triggered(&self) -> bool {
        self.0.property("triggered") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Crafter.setTriggered
    pub fn set_triggered(&mut self, value: bool) {
        self.0.set_property("triggered", if value { "true" } else { "false" });
    }
    /// bukkit: org/bukkit/block/data/type/Crafter.getOrientation
    pub fn get_orientation(&self) -> String {
        self.0.property("orientation").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Crafter.setOrientation
    pub fn set_orientation(&mut self, value: &str) {
        self.0.set_property("orientation", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/CreakingHeart
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreakingHeartData(pub BlockData);

impl DataClass for CreakingHeartData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl CreakingHeartData {
    /// bukkit: org/bukkit/block/data/type/CreakingHeart.getCreakingHeartState
    pub fn get_creaking_heart_state(&self) -> String {
        self.0.property("creaking_heart_state").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/CreakingHeart.setCreakingHeartState
    pub fn set_creaking_heart_state(&mut self, value: &str) {
        self.0.set_property("creaking_heart_state", &value.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/type/CreakingHeart.isNatural
    pub fn is_natural(&self) -> bool {
        self.0.property("natural") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/CreakingHeart.setNatural
    pub fn set_natural(&mut self, value: bool) {
        self.0.set_property("natural", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/DaylightDetector
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DaylightDetectorData(pub BlockData);

impl DataClass for DaylightDetectorData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl DaylightDetectorData {
    /// bukkit: org/bukkit/block/data/type/DaylightDetector.isInverted
    pub fn is_inverted(&self) -> bool {
        self.0.property("inverted") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/DaylightDetector.setInverted
    pub fn set_inverted(&mut self, value: bool) {
        self.0.set_property("inverted", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/DecoratedPot
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecoratedPotData(pub BlockData);

impl DataClass for DecoratedPotData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl DecoratedPotData {
    /// bukkit: org/bukkit/block/data/type/DecoratedPot.isCracked
    pub fn is_cracked(&self) -> bool {
        self.0.property("cracked") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/DecoratedPot.setCracked
    pub fn set_cracked(&mut self, value: bool) {
        self.0.set_property("cracked", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Dispenser
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispenserData(pub BlockData);

impl DataClass for DispenserData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl DispenserData {
    /// bukkit: org/bukkit/block/data/type/Dispenser.isTriggered
    pub fn is_triggered(&self) -> bool {
        self.0.property("triggered") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Dispenser.setTriggered
    pub fn set_triggered(&mut self, value: bool) {
        self.0.set_property("triggered", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Door
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoorData(pub BlockData);

impl DataClass for DoorData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl DoorData {
    /// bukkit: org/bukkit/block/data/type/Door.getHinge
    pub fn get_hinge(&self) -> String {
        self.0.property("hinge").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Door.setHinge
    pub fn set_hinge(&mut self, value: &str) {
        self.0.set_property("hinge", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/DriedGhast
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriedGhastData(pub BlockData);

impl DataClass for DriedGhastData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl DriedGhastData {
    /// bukkit: org/bukkit/block/data/type/DriedGhast.getHydration
    pub fn get_hydration(&self) -> i32 {
        self.0.property("hydration").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/DriedGhast.setHydration
    pub fn set_hydration(&mut self, value: i32) {
        self.0.set_property("hydration", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/DriedGhast.getMaximumHydration
    pub fn get_maximum_hydration(&self) -> i32 {
        self.0.maximum_of("hydration")
    }
}

/// bukkit: org/bukkit/block/data/type/EndPortalFrame
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndPortalFrameData(pub BlockData);

impl DataClass for EndPortalFrameData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl EndPortalFrameData {
    /// bukkit: org/bukkit/block/data/type/EndPortalFrame.hasEye
    pub fn has_eye(&self) -> bool {
        self.0.property("eye") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/EndPortalFrame.setEye
    pub fn set_eye(&mut self, value: bool) {
        self.0.set_property("eye", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Farmland
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FarmlandData(pub BlockData);

impl DataClass for FarmlandData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl FarmlandData {
    /// bukkit: org/bukkit/block/data/type/Farmland.getMoisture
    pub fn get_moisture(&self) -> i32 {
        self.0.property("moisture").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Farmland.setMoisture
    pub fn set_moisture(&mut self, value: i32) {
        self.0.set_property("moisture", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Farmland.getMaximumMoisture
    pub fn get_maximum_moisture(&self) -> i32 {
        self.0.maximum_of("moisture")
    }
}

/// bukkit: org/bukkit/block/data/type/FlowerBed
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowerBedData(pub BlockData);

impl DataClass for FlowerBedData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl FlowerBedData {
    /// bukkit: org/bukkit/block/data/type/FlowerBed.getFlowerAmount
    pub fn get_flower_amount(&self) -> i32 {
        self.0.property("flower_amount").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/FlowerBed.setFlowerAmount
    pub fn set_flower_amount(&mut self, value: i32) {
        self.0.set_property("flower_amount", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/FlowerBed.getMaximumFlowerAmount
    pub fn get_maximum_flower_amount(&self) -> i32 {
        self.0.maximum_of("flower_amount")
    }
}

/// bukkit: org/bukkit/block/data/type/Gate
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateData(pub BlockData);

impl DataClass for GateData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl GateData {
    /// bukkit: org/bukkit/block/data/type/Gate.isInWall
    pub fn is_in_wall(&self) -> bool {
        self.0.property("in_wall") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Gate.setInWall
    pub fn set_in_wall(&mut self, value: bool) {
        self.0.set_property("in_wall", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/HangingMoss
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HangingMossData(pub BlockData);

impl DataClass for HangingMossData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl HangingMossData {
    /// bukkit: org/bukkit/block/data/type/HangingMoss.isTip
    pub fn is_tip(&self) -> bool {
        self.0.property("tip") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/HangingMoss.setTip
    pub fn set_tip(&mut self, value: bool) {
        self.0.set_property("tip", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Hopper
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HopperData(pub BlockData);

impl DataClass for HopperData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl HopperData {
    /// bukkit: org/bukkit/block/data/type/Hopper.isEnabled
    pub fn is_enabled(&self) -> bool {
        self.0.property("enabled") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Hopper.setEnabled
    pub fn set_enabled(&mut self, value: bool) {
        self.0.set_property("enabled", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Jigsaw
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JigsawData(pub BlockData);

impl DataClass for JigsawData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl JigsawData {
    /// bukkit: org/bukkit/block/data/type/Jigsaw.getOrientation
    pub fn get_orientation(&self) -> String {
        self.0.property("orientation").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/Jigsaw.setOrientation
    pub fn set_orientation(&mut self, value: &str) {
        self.0.set_property("orientation", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/data/type/Jukebox
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JukeboxData(pub BlockData);

impl DataClass for JukeboxData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl JukeboxData {
    /// bukkit: org/bukkit/block/data/type/Jukebox.hasRecord
    pub fn has_record(&self) -> bool {
        self.0.property("has_record") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Jukebox.setHasRecord
    pub fn set_has_record(&mut self, value: bool) {
        self.0.set_property("has_record", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/Leaves
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeavesData(pub BlockData);

impl DataClass for LeavesData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl LeavesData {
    /// bukkit: org/bukkit/block/data/type/Leaves.isPersistent
    pub fn is_persistent(&self) -> bool {
        self.0.property("persistent") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Leaves.setPersistent
    pub fn set_persistent(&mut self, value: bool) {
        self.0.set_property("persistent", if value { "true" } else { "false" });
    }
    /// bukkit: org/bukkit/block/data/type/Leaves.getDistance
    pub fn get_distance(&self) -> i32 {
        self.0.property("distance").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/Leaves.setDistance
    pub fn set_distance(&mut self, value: i32) {
        self.0.set_property("distance", &value.to_string());
    }
    /// bukkit: org/bukkit/block/data/type/Leaves.getMaximumDistance
    pub fn get_maximum_distance(&self) -> i32 {
        self.0.maximum_of("distance")
    }
}

/// bukkit: org/bukkit/block/data/type/Lectern
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LecternData(pub BlockData);

impl DataClass for LecternData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl LecternData {
    /// bukkit: org/bukkit/block/data/type/Lectern.hasBook
    pub fn has_book(&self) -> bool {
        self.0.property("has_book") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Lectern.setHasBook
    pub fn set_has_book(&mut self, value: bool) {
        self.0.set_property("has_book", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/MossyCarpet
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MossyCarpetData(pub BlockData);

impl DataClass for MossyCarpetData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl MossyCarpetData {
    /// bukkit: org/bukkit/block/data/type/MossyCarpet.isBottom
    pub fn is_bottom(&self) -> bool {
        self.0.property("bottom") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/MossyCarpet.setBottom
    pub fn set_bottom(&mut self, value: bool) {
        self.0.set_property("bottom", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/NoteBlock
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteBlockData(pub BlockData);

impl DataClass for NoteBlockData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl NoteBlockData {
    /// bukkit: org/bukkit/block/data/type/NoteBlock.getInstrumentName
    pub fn get_instrument_name(&self) -> String {
        self.0.property("instrument").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/NoteBlock.setInstrumentName
    pub fn set_instrument_name(&mut self, value: &str) {
        self.0.set_property("instrument", &value.to_ascii_lowercase());
    }
    /// bukkit: org/bukkit/block/data/type/NoteBlock.getNoteNumber
    pub fn get_note_number(&self) -> i32 {
        self.0.property("note").and_then(|value| value.parse().ok()).unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/data/type/NoteBlock.setNoteNumber
    pub fn set_note_number(&mut self, value: i32) {
        self.0.set_property("note", &value.to_string());
    }
}

/// bukkit: org/bukkit/block/data/type/Piston
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PistonData(pub BlockData);

impl DataClass for PistonData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl PistonData {
    /// bukkit: org/bukkit/block/data/type/Piston.isExtended
    pub fn is_extended(&self) -> bool {
        self.0.property("extended") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/Piston.setExtended
    pub fn set_extended(&mut self, value: bool) {
        self.0.set_property("extended", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/PistonHead
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PistonHeadData(pub BlockData);

impl DataClass for PistonHeadData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl PistonHeadData {
    /// bukkit: org/bukkit/block/data/type/PistonHead.isShort
    pub fn is_short(&self) -> bool {
        self.0.property("short") == Some("true")
    }
    /// bukkit: org/bukkit/block/data/type/PistonHead.setShort
    pub fn set_short(&mut self, value: bool) {
        self.0.set_property("short", if value { "true" } else { "false" });
    }
}

/// bukkit: org/bukkit/block/data/type/PotentSulfur
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PotentSulfurData(pub BlockData);

impl DataClass for PotentSulfurData {
    fn of(data: BlockData) -> Self {
        Self(data)
    }
    fn data(&self) -> &BlockData {
        &self.0
    }
}

impl PotentSulfurData {
    /// bukkit: org/bukkit/block/data/type/PotentSulfur.getPotentSulfurState
    pub fn get_potent_sulfur_state(&self) -> String {
        self.0.property("potent_sulfur_state").unwrap_or_default().to_ascii_uppercase()
    }
    /// bukkit: org/bukkit/block/data/type/PotentSulfur.setPotentSulfurState
    pub fn set_potent_sulfur_state(&mut self, value: &str) {
        self.0.set_property("potent_sulfur_state", &value.to_ascii_lowercase());
    }
}

/// bukkit: org/bukkit/block/Banner
#[derive(Clone, Debug, PartialEq)]
pub struct Banner(pub TileState);

impl StateClass for Banner {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Banner {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

/// bukkit: org/bukkit/block/Beacon
#[derive(Clone, Debug, PartialEq)]
pub struct Beacon(pub TileState);

impl StateClass for Beacon {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Beacon {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Beacon {
    /// bukkit: org/bukkit/block/Beacon.getTier
    pub fn get_tier(&self) -> i32 {
        self.0.saved("Levels").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Beacon.getPrimaryEffectId
    pub fn get_primary_effect_id(&self) -> Option<String> {
        self.0.saved("primary_effect").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Beacon.setPrimaryEffect
    pub fn set_primary_effect(&mut self, value: &str) {
        self.0.change("primary_effect", crate::data::quoted(value));
    }
    /// bukkit: org/bukkit/block/Beacon.getSecondaryEffectId
    pub fn get_secondary_effect_id(&self) -> Option<String> {
        self.0.saved("secondary_effect").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Beacon.setSecondaryEffect
    pub fn set_secondary_effect(&mut self, value: &str) {
        self.0.change("secondary_effect", crate::data::quoted(value));
    }
}

/// bukkit: org/bukkit/block/Beehive
#[derive(Clone, Debug, PartialEq)]
pub struct Beehive(pub TileState);

impl StateClass for Beehive {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Beehive {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Beehive {
    /// bukkit: org/bukkit/block/Beehive.getFlower
    pub fn get_flower(&self) -> Option<crate::Location> {
        self.0.location_of(&self.0.saved("flower_pos"))
    }
    /// bukkit: org/bukkit/block/Beehive.setFlower
    pub fn set_flower(&mut self, value: &crate::Location) {
        self.0.change("flower_pos", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()));
    }
}

/// bukkit: org/bukkit/block/BrewingStand
#[derive(Clone, Debug, PartialEq)]
pub struct BrewingStand(pub TileState);

impl StateClass for BrewingStand {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for BrewingStand {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl BrewingStand {
    /// bukkit: org/bukkit/block/BrewingStand.getBrewingTime
    pub fn get_brewing_time(&self) -> i32 {
        self.0.saved("BrewTime").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/BrewingStand.setBrewingTime
    pub fn set_brewing_time(&mut self, value: i32) {
        self.0.change("BrewTime", format!("{value}s"));
    }
    /// bukkit: org/bukkit/block/BrewingStand.getFuelLevel
    pub fn get_fuel_level(&self) -> i32 {
        self.0.saved("Fuel").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/BrewingStand.setFuelLevel
    pub fn set_fuel_level(&mut self, value: i32) {
        self.0.change("Fuel", format!("{value}b"));
    }
}

/// bukkit: org/bukkit/block/BrushableBlock
#[derive(Clone, Debug, PartialEq)]
pub struct BrushableBlock(pub TileState);

impl StateClass for BrushableBlock {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for BrushableBlock {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl BrushableBlock {
    /// bukkit: org/bukkit/block/BrushableBlock.getItem
    pub fn get_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.0.saved("item"))
    }
    /// bukkit: org/bukkit/block/BrushableBlock.setItem
    pub fn set_item(&mut self, value: &crate::ItemStack) {
        self.0.change("item", value.to_snbt());
    }
}

/// bukkit: org/bukkit/block/CommandBlock
#[derive(Clone, Debug, PartialEq)]
pub struct CommandBlock(pub TileState);

impl StateClass for CommandBlock {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for CommandBlock {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl CommandBlock {
    /// bukkit: org/bukkit/block/CommandBlock.getCommand
    pub fn get_command(&self) -> Option<String> {
        self.0.saved("Command").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/CommandBlock.setCommand
    pub fn set_command(&mut self, value: &str) {
        self.0.change("Command", crate::data::quoted(value));
    }
}

/// bukkit: org/bukkit/block/Crafter
#[derive(Clone, Debug, PartialEq)]
pub struct Crafter(pub TileState);

impl StateClass for Crafter {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Crafter {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Crafter {
    /// bukkit: org/bukkit/block/Crafter.getCraftingTicks
    pub fn get_crafting_ticks(&self) -> i32 {
        self.0.saved("crafting_ticks_remaining").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Crafter.setCraftingTicks
    pub fn set_crafting_ticks(&mut self, value: i32) {
        self.0.change("crafting_ticks_remaining", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/Crafter.isTriggered
    pub fn is_triggered(&self) -> bool {
        crate::data::flag(&self.0.saved("triggered"))
    }
    /// bukkit: org/bukkit/block/Crafter.setTriggered
    pub fn set_triggered(&mut self, value: bool) {
        self.0.change("triggered", format!("{value}"));
    }
}

/// bukkit: org/bukkit/block/EndGateway
#[derive(Clone, Debug, PartialEq)]
pub struct EndGateway(pub TileState);

impl StateClass for EndGateway {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for EndGateway {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl EndGateway {
    /// bukkit: org/bukkit/block/EndGateway.getExitLocation
    pub fn get_exit_location(&self) -> Option<crate::Location> {
        self.0.location_of(&self.0.saved("exit_portal"))
    }
    /// bukkit: org/bukkit/block/EndGateway.setExitLocation
    pub fn set_exit_location(&mut self, value: &crate::Location) {
        self.0.change("exit_portal", format!("[I;{},{},{}]", value.get_block_x(), value.get_block_y(), value.get_block_z()));
    }
    /// bukkit: org/bukkit/block/EndGateway.isExactTeleport
    pub fn is_exact_teleport(&self) -> bool {
        crate::data::flag(&self.0.saved("ExactTeleport"))
    }
    /// bukkit: org/bukkit/block/EndGateway.setExactTeleport
    pub fn set_exact_teleport(&mut self, value: bool) {
        self.0.change("ExactTeleport", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/EndGateway.getAge
    pub fn get_age(&self) -> i64 {
        self.0.saved("Age").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/EndGateway.setAge
    pub fn set_age(&mut self, value: i64) {
        self.0.change("Age", format!("{value}L"));
    }
}

/// bukkit: org/bukkit/block/Furnace
#[derive(Clone, Debug, PartialEq)]
pub struct Furnace(pub TileState);

impl StateClass for Furnace {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Furnace {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Furnace {
    /// bukkit: org/bukkit/block/Furnace.getBurnTime
    pub fn get_burn_time(&self) -> i32 {
        self.0.saved("lit_time_remaining").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Furnace.setBurnTime
    pub fn set_burn_time(&mut self, value: i32) {
        self.0.change("lit_time_remaining", format!("{value}s"));
    }
    /// bukkit: org/bukkit/block/Furnace.getCookTime
    pub fn get_cook_time(&self) -> i32 {
        self.0.saved("cooking_time_spent").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Furnace.setCookTime
    pub fn set_cook_time(&mut self, value: i32) {
        self.0.change("cooking_time_spent", format!("{value}s"));
    }
    /// bukkit: org/bukkit/block/Furnace.getCookTimeTotal
    pub fn get_cook_time_total(&self) -> i32 {
        self.0.saved("cooking_total_time").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Furnace.setCookTimeTotal
    pub fn set_cook_time_total(&mut self, value: i32) {
        self.0.change("cooking_total_time", format!("{value}s"));
    }
}

/// bukkit: org/bukkit/block/Hopper
#[derive(Clone, Debug, PartialEq)]
pub struct Hopper(pub TileState);

impl StateClass for Hopper {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Hopper {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Hopper {
    /// bukkit: org/bukkit/block/Hopper.getTransferCooldown
    pub fn get_transfer_cooldown(&self) -> i32 {
        self.0.saved("TransferCooldown").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Hopper.setTransferCooldown
    pub fn set_transfer_cooldown(&mut self, value: i32) {
        self.0.change("TransferCooldown", format!("{value}"));
    }
}

/// bukkit: org/bukkit/block/Jukebox
#[derive(Clone, Debug, PartialEq)]
pub struct Jukebox(pub TileState);

impl StateClass for Jukebox {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Jukebox {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Jukebox {
    /// bukkit: org/bukkit/block/Jukebox.getRecord
    pub fn get_record(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.0.saved("RecordItem"))
    }
    /// bukkit: org/bukkit/block/Jukebox.setRecord
    pub fn set_record(&mut self, value: &crate::ItemStack) {
        self.0.change("RecordItem", value.to_snbt());
    }
}

/// bukkit: org/bukkit/block/Lectern
#[derive(Clone, Debug, PartialEq)]
pub struct Lectern(pub TileState);

impl StateClass for Lectern {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Lectern {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Lectern {
    /// bukkit: org/bukkit/block/Lectern.getPage
    pub fn get_page(&self) -> i32 {
        self.0.saved("Page").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/Lectern.setPage
    pub fn set_page(&mut self, value: i32) {
        self.0.change("Page", format!("{value}"));
    }
}

/// bukkit: org/bukkit/block/SculkSensor
#[derive(Clone, Debug, PartialEq)]
pub struct SculkSensor(pub TileState);

impl StateClass for SculkSensor {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for SculkSensor {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl SculkSensor {
    /// bukkit: org/bukkit/block/SculkSensor.getLastVibrationFrequency
    pub fn get_last_vibration_frequency(&self) -> i32 {
        self.0.saved("last_vibration_frequency").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/SculkSensor.setLastVibrationFrequency
    pub fn set_last_vibration_frequency(&mut self, value: i32) {
        self.0.change("last_vibration_frequency", format!("{value}"));
    }
}

/// bukkit: org/bukkit/block/Sign
#[derive(Clone, Debug, PartialEq)]
pub struct Sign(pub TileState);

impl StateClass for Sign {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Sign {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Sign {
    /// bukkit: org/bukkit/block/Sign.isWaxed
    pub fn is_waxed(&self) -> bool {
        crate::data::flag(&self.0.saved("is_waxed"))
    }
    /// bukkit: org/bukkit/block/Sign.setWaxed
    pub fn set_waxed(&mut self, value: bool) {
        self.0.change("is_waxed", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/Sign.isGlowingText
    pub fn is_glowing_text(&self) -> bool {
        crate::data::flag(&self.0.saved("front_text.has_glowing_text"))
    }
    /// bukkit: org/bukkit/block/Sign.setGlowingText
    pub fn set_glowing_text(&mut self, value: bool) {
        self.0.change("front_text.has_glowing_text", format!("{value}"));
    }
}

/// bukkit: org/bukkit/block/Structure
#[derive(Clone, Debug, PartialEq)]
pub struct Structure(pub TileState);

impl StateClass for Structure {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Structure {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Structure {
    /// bukkit: org/bukkit/block/Structure.getStructureName
    pub fn get_structure_name(&self) -> Option<String> {
        self.0.saved("name").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Structure.setStructureName
    pub fn set_structure_name(&mut self, value: &str) {
        self.0.change("name", crate::data::quoted(value));
    }
    /// bukkit: org/bukkit/block/Structure.getAuthor
    pub fn get_author(&self) -> Option<String> {
        self.0.saved("author").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Structure.setAuthor
    pub fn set_author(&mut self, value: &str) {
        self.0.change("author", crate::data::quoted(value));
    }
    /// bukkit: org/bukkit/block/Structure.getIntegrity
    pub fn get_integrity(&self) -> f32 {
        self.0.saved("integrity").as_f64().unwrap_or(0.0) as f32
    }
    /// bukkit: org/bukkit/block/Structure.setIntegrity
    pub fn set_integrity(&mut self, value: f32) {
        self.0.change("integrity", format!("{value:?}f"));
    }
    /// bukkit: org/bukkit/block/Structure.getSeed
    pub fn get_seed(&self) -> i64 {
        self.0.saved("seed").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/Structure.setSeed
    pub fn set_seed(&mut self, value: i64) {
        self.0.change("seed", format!("{value}L"));
    }
    /// bukkit: org/bukkit/block/Structure.isIgnoreEntities
    pub fn is_ignore_entities(&self) -> bool {
        crate::data::flag(&self.0.saved("ignoreEntities"))
    }
    /// bukkit: org/bukkit/block/Structure.setIgnoreEntities
    pub fn set_ignore_entities(&mut self, value: bool) {
        self.0.change("ignoreEntities", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/Structure.isShowAir
    pub fn is_show_air(&self) -> bool {
        crate::data::flag(&self.0.saved("showair"))
    }
    /// bukkit: org/bukkit/block/Structure.setShowAir
    pub fn set_show_air(&mut self, value: bool) {
        self.0.change("showair", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/Structure.isBoundingBoxVisible
    pub fn is_bounding_box_visible(&self) -> bool {
        crate::data::flag(&self.0.saved("showboundingbox"))
    }
    /// bukkit: org/bukkit/block/Structure.setBoundingBoxVisible
    pub fn set_bounding_box_visible(&mut self, value: bool) {
        self.0.change("showboundingbox", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/Structure.getMetadata
    pub fn get_metadata(&self) -> Option<String> {
        self.0.saved("metadata").as_str().map(str::to_owned)
    }
    /// bukkit: org/bukkit/block/Structure.setMetadata
    pub fn set_metadata(&mut self, value: &str) {
        self.0.change("metadata", crate::data::quoted(value));
    }
}

/// bukkit: org/bukkit/block/TrialSpawner
#[derive(Clone, Debug, PartialEq)]
pub struct TrialSpawner(pub TileState);

impl StateClass for TrialSpawner {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for TrialSpawner {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl TrialSpawner {
    /// bukkit: org/bukkit/block/TrialSpawner.getCooldownLength
    pub fn get_cooldown_length(&self) -> i32 {
        self.0.saved("target_cooldown_length").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/TrialSpawner.setCooldownLength
    pub fn set_cooldown_length(&mut self, value: i32) {
        self.0.change("target_cooldown_length", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/TrialSpawner.getRequiredPlayerRange
    pub fn get_required_player_range(&self) -> i32 {
        self.0.saved("required_player_range").as_i64().unwrap_or(0) as i32
    }
    /// bukkit: org/bukkit/block/TrialSpawner.setRequiredPlayerRange
    pub fn set_required_player_range(&mut self, value: i32) {
        self.0.change("required_player_range", format!("{value}"));
    }
    /// bukkit: org/bukkit/block/TrialSpawner.getCooldownEnd
    pub fn get_cooldown_end(&self) -> i64 {
        self.0.saved("cooldown_end_at").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/TrialSpawner.setCooldownEnd
    pub fn set_cooldown_end(&mut self, value: i64) {
        self.0.change("cooldown_end_at", format!("{value}L"));
    }
    /// bukkit: org/bukkit/block/TrialSpawner.getNextSpawnAttempt
    pub fn get_next_spawn_attempt(&self) -> i64 {
        self.0.saved("next_mob_spawns_at").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/TrialSpawner.setNextSpawnAttempt
    pub fn set_next_spawn_attempt(&mut self, value: i64) {
        self.0.change("next_mob_spawns_at", format!("{value}L"));
    }
}

/// bukkit: org/bukkit/block/Vault
#[derive(Clone, Debug, PartialEq)]
pub struct Vault(pub TileState);

impl StateClass for Vault {
    fn of(state: TileState) -> Self {
        Self(state)
    }
}

impl std::ops::Deref for Vault {
    type Target = TileState;
    fn deref(&self) -> &TileState {
        &self.0
    }
}

impl Vault {
    /// bukkit: org/bukkit/block/Vault.getActivationRange
    pub fn get_activation_range(&self) -> f64 {
        self.0.saved("config.activation_range").as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/block/Vault.setActivationRange
    pub fn set_activation_range(&mut self, value: f64) {
        self.0.change("config.activation_range", format!("{value:?}d"));
    }
    /// bukkit: org/bukkit/block/Vault.getDeactivationRange
    pub fn get_deactivation_range(&self) -> f64 {
        self.0.saved("config.deactivation_range").as_f64().unwrap_or(0.0)
    }
    /// bukkit: org/bukkit/block/Vault.setDeactivationRange
    pub fn set_deactivation_range(&mut self, value: f64) {
        self.0.change("config.deactivation_range", format!("{value:?}d"));
    }
    /// bukkit: org/bukkit/block/Vault.getKeyItem
    pub fn get_key_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.0.saved("config.key_item"))
    }
    /// bukkit: org/bukkit/block/Vault.setKeyItem
    pub fn set_key_item(&mut self, value: &crate::ItemStack) {
        self.0.change("config.key_item", value.to_snbt());
    }
    /// bukkit: org/bukkit/block/Vault.getDisplayedItem
    pub fn get_displayed_item(&self) -> Option<crate::ItemStack> {
        crate::ItemStack::read(&self.0.saved("shared_data.display_item"))
    }
    /// bukkit: org/bukkit/block/Vault.setDisplayedItem
    pub fn set_displayed_item(&mut self, value: &crate::ItemStack) {
        self.0.change("shared_data.display_item", value.to_snbt());
    }
    /// bukkit: org/bukkit/block/Vault.getNextStateUpdateTime
    pub fn get_next_state_update_time(&self) -> i64 {
        self.0.saved("server_data.state_updating_resumes_at").as_i64().unwrap_or(0)
    }
    /// bukkit: org/bukkit/block/Vault.setNextStateUpdateTime
    pub fn set_next_state_update_time(&mut self, value: i64) {
        self.0.change("server_data.state_updating_resumes_at", format!("{value}L"));
    }
}
