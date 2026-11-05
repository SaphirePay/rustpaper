//! `org.bukkit.util`: the math every plugin leans on, with no game behind it.
use crate::{Location, World};
use serde_json::json;

/// `org.bukkit.util.Vector`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

const EPSILON: f64 = 0.000001;

impl Vector {
    /// bukkit: org/bukkit/util/Vector.<init>
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    /// bukkit: org/bukkit/util/Vector.<init>
    ///
    /// `new Vector()`: zero.
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
    /// bukkit: org/bukkit/util/Vector.getX
    pub fn get_x(&self) -> f64 {
        self.x
    }
    /// bukkit: org/bukkit/util/Vector.getY
    pub fn get_y(&self) -> f64 {
        self.y
    }
    /// bukkit: org/bukkit/util/Vector.getZ
    pub fn get_z(&self) -> f64 {
        self.z
    }
    /// bukkit: org/bukkit/util/Vector.setX
    pub fn set_x(&mut self, x: f64) -> &mut Self {
        self.x = x;
        self
    }
    /// bukkit: org/bukkit/util/Vector.setY
    pub fn set_y(&mut self, y: f64) -> &mut Self {
        self.y = y;
        self
    }
    /// bukkit: org/bukkit/util/Vector.setZ
    pub fn set_z(&mut self, z: f64) -> &mut Self {
        self.z = z;
        self
    }
    /// bukkit: org/bukkit/util/Vector.getBlockX
    pub fn get_block_x(&self) -> i32 {
        self.x.floor() as i32
    }
    /// bukkit: org/bukkit/util/Vector.getBlockY
    pub fn get_block_y(&self) -> i32 {
        self.y.floor() as i32
    }
    /// bukkit: org/bukkit/util/Vector.getBlockZ
    pub fn get_block_z(&self) -> i32 {
        self.z.floor() as i32
    }
    /// bukkit: org/bukkit/util/Vector.add
    pub fn add(&mut self, other: &Self) -> &mut Self {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self
    }
    /// bukkit: org/bukkit/util/Vector.subtract
    pub fn subtract(&mut self, other: &Self) -> &mut Self {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self
    }
    /// bukkit: org/bukkit/util/Vector.multiply
    pub fn multiply(&mut self, factor: f64) -> &mut Self {
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
        self
    }
    /// bukkit: org/bukkit/util/Vector.divide
    pub fn divide(&mut self, other: &Self) -> &mut Self {
        self.x /= other.x;
        self.y /= other.y;
        self.z /= other.z;
        self
    }
    /// bukkit: org/bukkit/util/Vector.copy
    pub fn copy(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }
    /// bukkit: org/bukkit/util/Vector.length
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }
    /// bukkit: org/bukkit/util/Vector.lengthSquared
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    /// bukkit: org/bukkit/util/Vector.distance
    pub fn distance(&self, other: &Self) -> f64 {
        self.distance_squared(other).sqrt()
    }
    /// bukkit: org/bukkit/util/Vector.distanceSquared
    pub fn distance_squared(&self, other: &Self) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)
    }
    /// bukkit: org/bukkit/util/Vector.angle
    pub fn angle(&self, other: &Self) -> f32 {
        let cosine = self.dot(other) / (self.length() * other.length());
        cosine.clamp(-1.0, 1.0).acos() as f32
    }
    /// bukkit: org/bukkit/util/Vector.midpoint
    pub fn midpoint(&mut self, other: &Self) -> &mut Self {
        self.x = (self.x + other.x) / 2.0;
        self.y = (self.y + other.y) / 2.0;
        self.z = (self.z + other.z) / 2.0;
        self
    }
    /// bukkit: org/bukkit/util/Vector.getMidpoint
    pub fn get_midpoint(&self, other: &Self) -> Self {
        Self::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0, (self.z + other.z) / 2.0)
    }
    /// bukkit: org/bukkit/util/Vector.dot
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    /// bukkit: org/bukkit/util/Vector.crossProduct
    pub fn cross_product(&mut self, other: &Self) -> &mut Self {
        *self = self.get_cross_product(other);
        self
    }
    /// bukkit: org/bukkit/util/Vector.getCrossProduct
    pub fn get_cross_product(&self, other: &Self) -> Self {
        Self::new(self.y * other.z - other.y * self.z, self.z * other.x - other.z * self.x, self.x * other.y - other.x * self.y)
    }
    /// bukkit: org/bukkit/util/Vector.normalize
    pub fn normalize(&mut self) -> &mut Self {
        let length = self.length();
        self.x /= length;
        self.y /= length;
        self.z /= length;
        self
    }
    /// bukkit: org/bukkit/util/Vector.zero
    pub fn zero(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
    /// bukkit: org/bukkit/util/Vector.isZero
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
    /// bukkit: org/bukkit/util/Vector.isNormalized
    pub fn is_normalized(&self) -> bool {
        (self.length_squared() - 1.0).abs() < EPSILON
    }
    /// bukkit: org/bukkit/util/Vector.isInAABB
    pub fn is_in_aabb(&self, min: &Self, max: &Self) -> bool {
        self.x >= min.x && self.x <= max.x && self.y >= min.y && self.y <= max.y && self.z >= min.z && self.z <= max.z
    }
    /// bukkit: org/bukkit/util/Vector.isInSphere
    pub fn is_in_sphere(&self, origin: &Self, radius: f64) -> bool {
        self.distance_squared(origin) <= radius * radius
    }
    /// bukkit: org/bukkit/util/Vector.rotateAroundX
    pub fn rotate_around_x(&mut self, angle: f64) -> &mut Self {
        let (sine, cosine) = angle.sin_cos();
        let (y, z) = (cosine * self.y - sine * self.z, sine * self.y + cosine * self.z);
        self.y = y;
        self.z = z;
        self
    }
    /// bukkit: org/bukkit/util/Vector.rotateAroundY
    pub fn rotate_around_y(&mut self, angle: f64) -> &mut Self {
        let (sine, cosine) = angle.sin_cos();
        let (x, z) = (cosine * self.x + sine * self.z, -sine * self.x + cosine * self.z);
        self.x = x;
        self.z = z;
        self
    }
    /// bukkit: org/bukkit/util/Vector.rotateAroundZ
    pub fn rotate_around_z(&mut self, angle: f64) -> &mut Self {
        let (sine, cosine) = angle.sin_cos();
        let (x, y) = (cosine * self.x - sine * self.y, sine * self.x + cosine * self.y);
        self.x = x;
        self.y = y;
        self
    }
    /// bukkit: org/bukkit/util/Vector.rotateAroundNonUnitAxis
    pub fn rotate_around_non_unit_axis(&mut self, axis: &Self, angle: f64) -> &mut Self {
        let (a, b, c) = (axis.x, axis.y, axis.z);
        let (x, y, z) = (self.x, self.y, self.z);
        let (sine, cosine) = angle.sin_cos();
        let dot = self.dot(axis);
        let xp = a * dot * (1.0 - cosine) + x * cosine + (-c * y + b * z) * sine;
        let yp = b * dot * (1.0 - cosine) + y * cosine + (c * x - a * z) * sine;
        let zp = c * dot * (1.0 - cosine) + z * cosine + (-b * x + a * y) * sine;
        *self = Self::new(xp, yp, zp);
        self
    }
    /// bukkit: org/bukkit/util/Vector.rotateAroundAxis
    pub fn rotate_around_axis(&mut self, axis: &Self, angle: f64) -> &mut Self {
        let unit = if axis.is_normalized() {
            *axis
        } else {
            let mut unit = *axis;
            *unit.normalize()
        };
        self.rotate_around_non_unit_axis(&unit, angle)
    }
    /// bukkit: org/bukkit/util/Vector.toLocation
    pub fn to_location(&self, world: &World) -> Location {
        Location::new(world.clone(), self.x, self.y, self.z, 0.0, 0.0)
    }
    /// bukkit: org/bukkit/util/Vector.toBlockVector
    pub fn to_block_vector(&self) -> BlockVector {
        BlockVector::new(self.x, self.y, self.z)
    }
    /// bukkit: org/bukkit/util/Vector.getEpsilon
    pub fn get_epsilon() -> f64 {
        EPSILON
    }
    /// bukkit: org/bukkit/util/Vector.getMinimum
    pub fn get_minimum(first: &Self, second: &Self) -> Self {
        Self::new(first.x.min(second.x), first.y.min(second.y), first.z.min(second.z))
    }
    /// bukkit: org/bukkit/util/Vector.getMaximum
    pub fn get_maximum(first: &Self, second: &Self) -> Self {
        Self::new(first.x.max(second.x), first.y.max(second.y), first.z.max(second.z))
    }
    /// bukkit: org/bukkit/util/Vector.checkFinite
    pub fn check_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
    /// bukkit: org/bukkit/util/Vector.serialize
    pub fn serialize(&self) -> serde_json::Value {
        json!({ "x": self.x, "y": self.y, "z": self.z })
    }
}

/// bukkit: org/bukkit/util/BlockVector
///
/// A vector that compares by its block.
#[derive(Clone, Copy, Debug)]
pub struct BlockVector(pub Vector);

impl BlockVector {
    /// bukkit: org/bukkit/util/BlockVector.<init>
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(Vector::new(x, y, z))
    }
}

impl PartialEq for BlockVector {
    fn eq(&self, other: &Self) -> bool {
        (self.0.get_block_x(), self.0.get_block_y(), self.0.get_block_z()) == (other.0.get_block_x(), other.0.get_block_y(), other.0.get_block_z())
    }
}

impl Location {
    /// bukkit: org/bukkit/Location.toVector
    pub fn to_vector(&self) -> Vector {
        Vector::new(self.x, self.y, self.z)
    }
    /// bukkit: org/bukkit/Location.getDirection
    pub fn get_direction(&self) -> Vector {
        let (yaw, pitch) = (f64::from(self.yaw).to_radians(), f64::from(self.pitch).to_radians());
        let flat = pitch.cos();
        Vector::new(-flat * yaw.sin(), -pitch.sin(), flat * yaw.cos())
    }
    /// bukkit: org/bukkit/Location.setDirection
    pub fn set_direction(&mut self, vector: &Vector) -> &mut Self {
        let tau = std::f64::consts::TAU;
        if vector.x == 0.0 && vector.z == 0.0 {
            self.pitch = if vector.y > 0.0 { -90.0 } else { 90.0 };
            return self;
        }
        let theta = (-vector.x).atan2(vector.z);
        self.yaw = ((theta + tau) % tau).to_degrees() as f32;
        let flat = (vector.x * vector.x + vector.z * vector.z).sqrt();
        self.pitch = (-vector.y).atan2(flat).to_degrees() as f32;
        self
    }
    /// bukkit: org/bukkit/Location.setX
    pub fn set_x(&mut self, x: f64) {
        self.x = x;
    }
    /// bukkit: org/bukkit/Location.setY
    pub fn set_y(&mut self, y: f64) {
        self.y = y;
    }
    /// bukkit: org/bukkit/Location.setZ
    pub fn set_z(&mut self, z: f64) {
        self.z = z;
    }
    /// bukkit: org/bukkit/Location.setYaw
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }
    /// bukkit: org/bukkit/Location.setPitch
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }
    /// bukkit: org/bukkit/Location.setWorld
    pub fn set_world(&mut self, world: World) {
        self.world = world;
    }
    /// bukkit: org/bukkit/Location.isWorldLoaded
    pub fn is_world_loaded(&self) -> bool {
        crate::get_world(self.world.get_name()).is_some()
    }
    /// bukkit: org/bukkit/Location.subtract
    pub fn subtract(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.x -= x;
        self.y -= y;
        self.z -= z;
        self
    }
    /// bukkit: org/bukkit/Location.multiply
    pub fn multiply(&mut self, factor: f64) -> &mut Self {
        self.x *= factor;
        self.y *= factor;
        self.z *= factor;
        self
    }
    /// bukkit: org/bukkit/Location.zero
    pub fn zero(&mut self) -> &mut Self {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
        self
    }
    /// bukkit: org/bukkit/Location.length
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }
    /// bukkit: org/bukkit/Location.lengthSquared
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    /// bukkit: org/bukkit/Location.clone
    pub fn copy(&self) -> Self {
        self.clone()
    }
    /// bukkit: org/bukkit/Location.toBlockLocation
    pub fn to_block_location(&self) -> Self {
        let mut at = self.clone();
        at.x = at.x.floor();
        at.y = at.y.floor();
        at.z = at.z.floor();
        at.yaw = 0.0;
        at.pitch = 0.0;
        at
    }
    /// bukkit: org/bukkit/Location.toCenterLocation
    pub fn to_center_location(&self) -> Self {
        let mut at = self.to_block_location();
        at.x += 0.5;
        at.y += 0.5;
        at.z += 0.5;
        at.yaw = self.yaw;
        at.pitch = self.pitch;
        at
    }
    /// bukkit: org/bukkit/Location.checkFinite
    pub fn check_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.yaw.is_finite() && self.pitch.is_finite()
    }
    /// bukkit: org/bukkit/Location.isChunkLoaded
    pub fn is_chunk_loaded(&self) -> bool {
        self.get_chunk().is_loaded()
    }
    /// bukkit: org/bukkit/Location.locToBlock
    pub fn loc_to_block(value: f64) -> i32 {
        value.floor() as i32
    }
    /// bukkit: org/bukkit/Location.normalizeYaw
    pub fn normalize_yaw(yaw: f32) -> f32 {
        let turned = yaw.rem_euclid(360.0);
        if turned >= 180.0 { turned - 360.0 } else { turned }
    }
    /// bukkit: org/bukkit/Location.normalizePitch
    pub fn normalize_pitch(pitch: f32) -> f32 {
        if pitch > 90.0 {
            90.0
        } else if pitch < -90.0 {
            -90.0
        } else {
            pitch
        }
    }
    /// bukkit: org/bukkit/Location.serialize
    pub fn serialize(&self) -> serde_json::Value {
        json!({ "world": self.world.get_name(), "x": self.x, "y": self.y, "z": self.z, "yaw": self.yaw, "pitch": self.pitch })
    }
}

/// `org.bukkit.util.EulerAngle`, in radians.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EulerAngle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl EulerAngle {
    /// bukkit: org/bukkit/util/EulerAngle.<init>
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    /// bukkit: org/bukkit/util/EulerAngle.getX
    pub fn get_x(&self) -> f64 {
        self.x
    }
    /// bukkit: org/bukkit/util/EulerAngle.getY
    pub fn get_y(&self) -> f64 {
        self.y
    }
    /// bukkit: org/bukkit/util/EulerAngle.getZ
    pub fn get_z(&self) -> f64 {
        self.z
    }
    /// bukkit: org/bukkit/util/EulerAngle.setX
    pub fn set_x(&self, x: f64) -> Self {
        Self { x, ..*self }
    }
    /// bukkit: org/bukkit/util/EulerAngle.setY
    pub fn set_y(&self, y: f64) -> Self {
        Self { y, ..*self }
    }
    /// bukkit: org/bukkit/util/EulerAngle.setZ
    pub fn set_z(&self, z: f64) -> Self {
        Self { z, ..*self }
    }
    /// bukkit: org/bukkit/util/EulerAngle.add
    pub fn add(&self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.x + x, self.y + y, self.z + z)
    }
    /// bukkit: org/bukkit/util/EulerAngle.subtract
    pub fn subtract(&self, x: f64, y: f64, z: f64) -> Self {
        self.add(-x, -y, -z)
    }
}

/// `org.bukkit.util.BoundingBox`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    pub min: Vector,
    pub max: Vector,
}

impl BoundingBox {
    /// bukkit: org/bukkit/util/BoundingBox.<init>, org/bukkit/util/BoundingBox.of
    pub fn new(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self { min: Vector::new(x1.min(x2), y1.min(y2), z1.min(z2)), max: Vector::new(x1.max(x2), y1.max(y2), z1.max(z2)) }
    }
    /// bukkit: org/bukkit/util/BoundingBox.of
    pub fn of(first: &Vector, second: &Vector) -> Self {
        Self::new(first.x, first.y, first.z, second.x, second.y, second.z)
    }
    /// bukkit: org/bukkit/util/BoundingBox.resize
    pub fn resize(&mut self, x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> &mut Self {
        *self = Self::new(x1, y1, z1, x2, y2, z2);
        self
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMinX
    pub fn get_min_x(&self) -> f64 {
        self.min.x
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMinY
    pub fn get_min_y(&self) -> f64 {
        self.min.y
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMinZ
    pub fn get_min_z(&self) -> f64 {
        self.min.z
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMin
    pub fn get_min(&self) -> Vector {
        self.min
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMaxX
    pub fn get_max_x(&self) -> f64 {
        self.max.x
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMaxY
    pub fn get_max_y(&self) -> f64 {
        self.max.y
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMaxZ
    pub fn get_max_z(&self) -> f64 {
        self.max.z
    }
    /// bukkit: org/bukkit/util/BoundingBox.getMax
    pub fn get_max(&self) -> Vector {
        self.max
    }
    /// bukkit: org/bukkit/util/BoundingBox.getWidthX
    pub fn get_width_x(&self) -> f64 {
        self.max.x - self.min.x
    }
    /// bukkit: org/bukkit/util/BoundingBox.getWidthZ
    pub fn get_width_z(&self) -> f64 {
        self.max.z - self.min.z
    }
    /// bukkit: org/bukkit/util/BoundingBox.getHeight
    pub fn get_height(&self) -> f64 {
        self.max.y - self.min.y
    }
    /// bukkit: org/bukkit/util/BoundingBox.getVolume
    pub fn get_volume(&self) -> f64 {
        self.get_height() * self.get_width_x() * self.get_width_z()
    }
    /// bukkit: org/bukkit/util/BoundingBox.getCenterX
    pub fn get_center_x(&self) -> f64 {
        self.min.x + self.get_width_x() * 0.5
    }
    /// bukkit: org/bukkit/util/BoundingBox.getCenterY
    pub fn get_center_y(&self) -> f64 {
        self.min.y + self.get_height() * 0.5
    }
    /// bukkit: org/bukkit/util/BoundingBox.getCenterZ
    pub fn get_center_z(&self) -> f64 {
        self.min.z + self.get_width_z() * 0.5
    }
    /// bukkit: org/bukkit/util/BoundingBox.getCenter
    pub fn get_center(&self) -> Vector {
        Vector::new(self.get_center_x(), self.get_center_y(), self.get_center_z())
    }
    /// bukkit: org/bukkit/util/BoundingBox.copy
    pub fn copy(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }
    /// bukkit: org/bukkit/util/BoundingBox.expand
    pub fn expand(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.expand_sides(x, y, z, x, y, z)
    }
    /// bukkit: org/bukkit/util/BoundingBox.expand
    pub fn expand_sides(&mut self, neg_x: f64, neg_y: f64, neg_z: f64, pos_x: f64, pos_y: f64, pos_z: f64) -> &mut Self {
        let center = self.get_center();
        let mut low = Vector::new(self.min.x - neg_x, self.min.y - neg_y, self.min.z - neg_z);
        let mut high = Vector::new(self.max.x + pos_x, self.max.y + pos_y, self.max.z + pos_z);
        // Java's clamp: a box shrunk past itself meets at the point on its center's line.
        if low.x > high.x {
            let middle = center.x.clamp(high.x, low.x);
            low.x = middle;
            high.x = middle;
        }
        if low.y > high.y {
            let middle = center.y.clamp(high.y, low.y);
            low.y = middle;
            high.y = middle;
        }
        if low.z > high.z {
            let middle = center.z.clamp(high.z, low.z);
            low.z = middle;
            high.z = middle;
        }
        self.min = low;
        self.max = high;
        self
    }
    /// bukkit: org/bukkit/util/BoundingBox.expandDirectional
    pub fn expand_directional(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.expand_sides((-x).max(0.0), (-y).max(0.0), (-z).max(0.0), x.max(0.0), y.max(0.0), z.max(0.0))
    }
    /// bukkit: org/bukkit/util/BoundingBox.union
    pub fn union(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        *self = Self::new(self.min.x.min(x), self.min.y.min(y), self.min.z.min(z), self.max.x.max(x), self.max.y.max(y), self.max.z.max(z));
        self
    }
    /// bukkit: org/bukkit/util/BoundingBox.union
    pub fn union_box(&mut self, other: &Self) -> &mut Self {
        *self = Self::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y), self.min.z.min(other.min.z), self.max.x.max(other.max.x), self.max.y.max(other.max.y), self.max.z.max(other.max.z));
        self
    }
    /// bukkit: org/bukkit/util/BoundingBox.overlaps
    pub fn overlaps(&self, other: &Self) -> bool {
        self.min.x < other.max.x && self.max.x > other.min.x && self.min.y < other.max.y && self.max.y > other.min.y && self.min.z < other.max.z && self.max.z > other.min.z
    }
    /// bukkit: org/bukkit/util/BoundingBox.intersection
    ///
    /// `None` for boxes that don't overlap, where Java throws.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        self.overlaps(other).then(|| Self::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y), self.min.z.max(other.min.z), self.max.x.min(other.max.x), self.max.y.min(other.max.y), self.max.z.min(other.max.z)))
    }
    /// bukkit: org/bukkit/util/BoundingBox.shift
    pub fn shift(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        let by = Vector::new(x, y, z);
        self.min.add(&by);
        self.max.add(&by);
        self
    }
    /// bukkit: org/bukkit/util/BoundingBox.contains
    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min.x && x < self.max.x && y >= self.min.y && y < self.max.y && z >= self.min.z && z < self.max.z
    }
    /// bukkit: org/bukkit/util/BoundingBox.contains
    pub fn contains_box(&self, other: &Self) -> bool {
        self.min.x <= other.min.x && other.max.x <= self.max.x && self.min.y <= other.min.y && other.max.y <= self.max.y && self.min.z <= other.min.z && other.max.z <= self.max.z
    }
    /// bukkit: org/bukkit/util/BoundingBox.rayTrace
    pub fn ray_trace(&self, start: &Vector, direction: &Vector, distance: f64) -> Option<RayTraceResult> {
        let mut direction = *direction;
        direction.normalize();
        let mut near = f64::NEG_INFINITY;
        let mut far = f64::INFINITY;
        let mut face = "SELF";
        let axes = [(start.x, direction.x, self.min.x, self.max.x, "WEST", "EAST"), (start.y, direction.y, self.min.y, self.max.y, "DOWN", "UP"), (start.z, direction.z, self.min.z, self.max.z, "NORTH", "SOUTH")];
        for (from, way, low, high, low_face, high_face) in axes {
            if way == 0.0 {
                if from < low || from > high {
                    return None;
                }
                continue;
            }
            let (first, second, entered) = if way > 0.0 { ((low - from) / way, (high - from) / way, low_face) } else { ((high - from) / way, (low - from) / way, high_face) };
            if first > near {
                near = first;
                face = entered;
            }
            far = far.min(second);
        }
        if near > far || far < 0.0 || near > distance {
            return None;
        }
        let along = near.max(0.0);
        Some(RayTraceResult { position: Vector::new(start.x + direction.x * along, start.y + direction.y * along, start.z + direction.z * along), block: None, face: Some(face), entity: None })
    }
    /// bukkit: org/bukkit/util/BoundingBox.serialize
    pub fn serialize(&self) -> serde_json::Value {
        json!({ "minX": self.min.x, "minY": self.min.y, "minZ": self.min.z, "maxX": self.max.x, "maxY": self.max.y, "maxZ": self.max.z })
    }
}

/// `org.bukkit.util.RayTraceResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct RayTraceResult {
    pub position: Vector,
    pub block: Option<crate::Block>,
    pub face: Option<&'static str>,
    pub entity: Option<crate::entity::Entity>,
}

impl RayTraceResult {
    /// bukkit: org/bukkit/util/RayTraceResult.getHitPosition
    pub fn get_hit_position(&self) -> Vector {
        self.position
    }
    /// bukkit: org/bukkit/util/RayTraceResult.getHitBlock
    pub fn get_hit_block(&self) -> Option<&crate::Block> {
        self.block.as_ref()
    }
    /// bukkit: org/bukkit/util/RayTraceResult.getHitBlockFace
    pub fn get_hit_block_face(&self) -> Option<&'static str> {
        self.face
    }
    /// bukkit: org/bukkit/util/RayTraceResult.getHitEntity
    pub fn get_hit_entity(&self) -> Option<&crate::entity::Entity> {
        self.entity.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_traces_to_its_near_face() {
        let bounds = BoundingBox::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        let hit = bounds.ray_trace(&Vector::new(-2.0, 0.5, 0.5), &Vector::new(1.0, 0.0, 0.0), 10.0).expect("a hit");
        assert_eq!(hit.position, Vector::new(0.0, 0.5, 0.5));
        assert_eq!(hit.face, Some("WEST"));
        assert!(bounds.ray_trace(&Vector::new(-2.0, 5.0, 0.5), &Vector::new(1.0, 0.0, 0.0), 10.0).is_none());
    }

    #[test]
    fn vectors_rotate_as_java_writes_them() {
        let mut vector = Vector::new(1.0, 0.0, 0.0);
        vector.rotate_around_y(std::f64::consts::FRAC_PI_2);
        assert!((vector.z + 1.0).abs() < 1e-9);
    }
}
