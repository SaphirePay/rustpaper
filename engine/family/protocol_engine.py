# Packets a plugin can read, change, cancel and send (ProtocolLib on Horizon, owner 11-03: "finish
# ProtocolLib"). Every packet's fields become values by name and kind; the play packets going to a player
# pass a hook a plugin listens to; and a plugin sends a packet it built.
EDITS = [
    # Each field as a value: numbers, text, positions, UUIDs, and lists or options of them. Anything else
    # (chunk data, NBT) reads as "opaque" and isn't written. Chosen at compile time (autoref), so a packet
    # with a field of any type still builds.
    ('horizon-protocol/src/buf.rs', """/// Define a packet struct and its codec from (field: type => codec) triples,
/// like vanilla's `StreamCodec.composite`.
#[macro_export]
macro_rules! packet {
    ($(#[$meta:meta])* $name:ident { $($field:ident : $ty:ty => $codec:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            $(pub $field: $ty),*
        }
""", """/// A field type a plugin reads and writes as a value.
pub trait FieldValue {
    const KIND: &'static str;
    fn field_json(&self) -> serde_json::Value;
    fn field_set(&mut self, value: &serde_json::Value) -> bool;
}
macro_rules! number_field {
    ($($t:ty => $kind:literal, $read:ident);* $(;)?) => {
        $(impl FieldValue for $t {
            const KIND: &'static str = $kind;
            fn field_json(&self) -> serde_json::Value {
                serde_json::json!(*self)
            }
            fn field_set(&mut self, value: &serde_json::Value) -> bool {
                value.$read().map(|v| *self = v as $t).is_some()
            }
        })*
    };
}
number_field! {
    i8 => "byte", as_i64; u8 => "byte", as_i64; i16 => "short", as_i64; u16 => "short", as_i64;
    i32 => "int", as_i64; u32 => "int", as_i64; i64 => "long", as_i64; u64 => "long", as_i64;
    f32 => "float", as_f64; f64 => "double", as_f64;
}
impl FieldValue for bool {
    const KIND: &'static str = "boolean";
    fn field_json(&self) -> serde_json::Value {
        serde_json::json!(*self)
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        value.as_bool().map(|v| *self = v).is_some()
    }
}
impl FieldValue for String {
    const KIND: &'static str = "string";
    fn field_json(&self) -> serde_json::Value {
        serde_json::json!(self)
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        value.as_str().map(|v| *self = v.to_owned()).is_some()
    }
}
/// A UUID as its 32 hex digits.
impl FieldValue for u128 {
    const KIND: &'static str = "uuid";
    fn field_json(&self) -> serde_json::Value {
        serde_json::json!(format!("{self:032x}"))
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        value.as_str().and_then(|v| u128::from_str_radix(&v.replace('-', ""), 16).ok()).map(|v| *self = v).is_some()
    }
}
/// A block position, or any three numbers.
impl<T: FieldValue + Copy> FieldValue for [T; 3] {
    const KIND: &'static str = "position";
    fn field_json(&self) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(FieldValue::field_json).collect())
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        let Some(items) = value.as_array().filter(|items| items.len() == 3) else {
            return false;
        };
        let mut next = *self;
        for (slot, item) in next.iter_mut().zip(items) {
            if !slot.field_set(item) {
                return false;
            }
        }
        *self = next;
        true
    }
}
/// A chunk position, or any two numbers.
impl<T: FieldValue + Copy> FieldValue for [T; 2] {
    const KIND: &'static str = "pair";
    fn field_json(&self) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(FieldValue::field_json).collect())
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        let Some(items) = value.as_array().filter(|items| items.len() == 2) else {
            return false;
        };
        let mut next = *self;
        for (slot, item) in next.iter_mut().zip(items) {
            if !slot.field_set(item) {
                return false;
            }
        }
        *self = next;
        true
    }
}
impl<T: FieldValue + Default> FieldValue for Vec<T> {
    const KIND: &'static str = "list";
    fn field_json(&self) -> serde_json::Value {
        serde_json::Value::Array(self.iter().map(FieldValue::field_json).collect())
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        let Some(items) = value.as_array() else {
            return false;
        };
        let mut next = Vec::with_capacity(items.len());
        for item in items {
            let mut slot = T::default();
            if !slot.field_set(item) {
                return false;
            }
            next.push(slot);
        }
        *self = next;
        true
    }
}
impl<T: FieldValue + Default> FieldValue for Option<T> {
    const KIND: &'static str = "optional";
    fn field_json(&self) -> serde_json::Value {
        self.as_ref().map_or(serde_json::Value::Null, FieldValue::field_json)
    }
    fn field_set(&mut self, value: &serde_json::Value) -> bool {
        if value.is_null() {
            *self = None;
            return true;
        }
        let mut slot = T::default();
        let ok = slot.field_set(value);
        if ok {
            *self = Some(slot);
        }
        ok
    }
}

/// A field, for the compile-time choice between a value and "opaque".
pub struct Probe<'a, T>(pub &'a T);
pub struct ProbeMut<'a, T>(pub std::cell::RefCell<&'a mut T>);
pub trait FieldKnown {
    fn field_kind(&self) -> &'static str;
    fn field_value(&self) -> serde_json::Value;
}
impl<T: FieldValue> FieldKnown for &Probe<'_, T> {
    fn field_kind(&self) -> &'static str {
        T::KIND
    }
    fn field_value(&self) -> serde_json::Value {
        self.0.field_json()
    }
}
pub trait FieldOpaque {
    fn field_kind(&self) -> &'static str {
        "opaque"
    }
    fn field_value(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
impl<T> FieldOpaque for Probe<'_, T> {}
pub trait FieldKnownMut {
    fn field_put(&self, value: &serde_json::Value) -> bool;
}
impl<T: FieldValue> FieldKnownMut for &ProbeMut<'_, T> {
    fn field_put(&self, value: &serde_json::Value) -> bool {
        self.0.borrow_mut().field_set(value)
    }
}
pub trait FieldOpaqueMut {
    fn field_put(&self, _: &serde_json::Value) -> bool {
        false
    }
}
impl<T> FieldOpaqueMut for ProbeMut<'_, T> {}

/// A packet's fields by name, kind and value, in the order they're sent.
pub trait PacketFields {
    fn fields(&self) -> Vec<(&'static str, &'static str, serde_json::Value)>;
    fn set_field(&mut self, name: &str, value: &serde_json::Value) -> bool;
}
pub struct Packet<'a, T>(pub std::cell::RefCell<&'a mut T>);
pub trait PacketKnown {
    fn packet_fields(&self) -> Vec<(&'static str, &'static str, serde_json::Value)>;
    fn packet_set(&self, name: &str, value: &serde_json::Value) -> bool;
}
impl<T: PacketFields> PacketKnown for &Packet<'_, T> {
    fn packet_fields(&self) -> Vec<(&'static str, &'static str, serde_json::Value)> {
        self.0.borrow().fields()
    }
    fn packet_set(&self, name: &str, value: &serde_json::Value) -> bool {
        self.0.borrow_mut().set_field(name, value)
    }
}
pub trait PacketOpaque {
    fn packet_fields(&self) -> Vec<(&'static str, &'static str, serde_json::Value)> {
        Vec::new()
    }
    fn packet_set(&self, _: &str, _: &serde_json::Value) -> bool {
        false
    }
}
impl<T> PacketOpaque for Packet<'_, T> {}

/// Define a packet struct and its codec from (field: type => codec) triples,
/// like vanilla's `StreamCodec.composite`.
#[macro_export]
macro_rules! packet {
    ($(#[$meta:meta])* $name:ident { $($field:ident : $ty:ty => $codec:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            $(pub $field: $ty),*
        }
        impl $crate::buf::PacketFields for $name {
            #[allow(unused_imports)]
            fn fields(&self) -> Vec<(&'static str, &'static str, serde_json::Value)> {
                use $crate::buf::{FieldKnown as _, FieldOpaque as _};
                vec![$((stringify!($field), (&&$crate::buf::Probe(&self.$field)).field_kind(), (&&$crate::buf::Probe(&self.$field)).field_value())),*]
            }
            #[allow(unused_imports, unused_variables)]
            fn set_field(&mut self, name: &str, value: &serde_json::Value) -> bool {
                use $crate::buf::{FieldKnownMut as _, FieldOpaqueMut as _};
                $(if name == stringify!($field) {
                    return (&&$crate::buf::ProbeMut(std::cell::RefCell::new(&mut self.$field))).field_put(value);
                })*
                false
            }
        }
"""),
    # The play packets by name, their fields, and a blank one to fill (ProtocolLib's createPacket).
    ('horizon-protocol/src/packets/mod.rs', """            /// Decode a packet body (after the id).
            pub fn decode(id: i32, payload: &[u8]) -> $crate::Result<Self> {""", """            /// The packet's name, as Horizon models it (`BlockUpdate`).
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant(_) => stringify!($variant),)*
                    Self::Unknown(..) => "Unknown",
                }
            }
            /// Its fields by name, kind and value; empty for a packet whose fields aren't values.
            #[allow(unused_imports)]
            pub fn fields(&mut self) -> Vec<(&'static str, &'static str, serde_json::Value)> {
                use $crate::buf::{PacketKnown as _, PacketOpaque as _};
                match self {
                    $(Self::$variant(p) => (&&$crate::buf::Packet(std::cell::RefCell::new(p))).packet_fields(),)*
                    Self::Unknown(..) => Vec::new(),
                }
            }
            /// Changes one field; false when it has no such field, or the value doesn't fit.
            #[allow(unused_imports)]
            pub fn set_field(&mut self, field: &str, value: &serde_json::Value) -> bool {
                use $crate::buf::{PacketKnown as _, PacketOpaque as _};
                match self {
                    $(Self::$variant(p) => (&&$crate::buf::Packet(std::cell::RefCell::new(p))).packet_set(field, value),)*
                    Self::Unknown(..) => false,
                }
            }
            /// A packet of this name with zeroes, empty text and empty lists, to fill with `set_field`.
            pub fn blank(name: &str) -> Option<Self> {
                let zeroes = vec![0_u8; 1 << 16];
                $(if name == stringify!($variant) {
                    let mut r = $crate::buf::Reader::new(&zeroes);
                    return <$ty as $crate::buf::Codec<$ty>>::read(&mut r).ok().map(Self::$variant);
                })*
                None
            }
            /// Decode a packet body (after the id).
            pub fn decode(id: i32, payload: &[u8]) -> $crate::Result<Self> {"""),
    # The play packets going to a player pass the plugins that listen to them (ProtocolLib's onPacketSending),
    # at the one door every packet goes through, encoded or not (chunks are written straight to bytes, and
    # a packet for many players is encoded once): each listener reads the fields, may change them
    # (`packet_edit`) or cancel the packet. Only a packet of a listened name is decoded. A packet a plugin
    # sends itself (`packet_send`) goes out quietly, past the listeners.
    ('horizon-client/src/lan/conn.rs', """    /// Told to go to configuration: its client no longer reads play packets.
    pub paused: Arc<std::sync::atomic::AtomicBool>,
}""", """    /// Told to go to configuration: its client no longer reads play packets.
    pub paused: Arc<std::sync::atomic::AtomicBool>,
    /// Whose packets these are (UUID, name), for the plugins that listen to packets.
    pub owner: Arc<std::sync::OnceLock<(u128, String)>>,
}"""),
    ('horizon-client/src/lan/conn.rs', """            watch: None,
            paused: Arc::default(),
        }
    }""", """            watch: None,
            paused: Arc::default(),
            owner: Arc::default(),
        }
    }
    /// A packet on its way: the plugins that listen to its kind read it, change it or cancel it (None).
    fn heard_by_plugins(&self, body: Vec<u8>) -> Option<Vec<u8>> {
        if !horizon_mod_runtime::any_packet_listened() {
            return Some(body);
        }
        let mut r = horizon_protocol::buf::Reader::new(&body);
        let Ok(id) = r.varint() else {
            return Some(body);
        };
        let Ok(mut packet) = horizon_protocol::packets::play::Clientbound::decode(id, r.rest()) else {
            return Some(body);
        };
        if !horizon_mod_runtime::packet_listened(packet.name()) {
            return Some(body);
        }
        let (uuid, name) = self.owner.get().cloned().unwrap_or_default();
        let fields: Vec<serde_json::Value> = packet.fields().into_iter().map(|(field, kind, value)| serde_json::json!([field, kind, value])).collect();
        let payload = serde_json::json!({ "uuid": format!("{uuid:032x}"), "player": name, "packet": packet.name(), "fields": fields });
        horizon_mod_runtime::begin_packet();
        let code = horizon_mod_runtime::fire("pl_send", &payload.to_string());
        let edits = horizon_mod_runtime::take_packet_edits();
        // 2: a listener cancelled it.
        if code == 2 {
            return None;
        }
        if edits.is_empty() {
            return Some(body);
        }
        for (field, value) in &edits {
            packet.set_field(field, value);
        }
        Some(packet.encode())
    }
    /// Sends without asking the plugins that listen to packets: a packet a plugin built itself.
    pub fn send_quiet(&self, body: Vec<u8>) {
        if self.paused.load(Ordering::Relaxed) {
            return;
        }
        self.pending.fetch_add(body.len(), Ordering::Relaxed);
        if HOLDING.get() {
            self.held().push(body);
        } else {
            self.flush();
            self.deliver(Out::Packet(body));
        }
    }"""),
    ('horizon-client/src/lan/conn.rs', """    pub fn send(&self, body: Vec<u8>) {
        if self.paused.load(Ordering::Relaxed) {
            return;
        }
        self.pending.fetch_add(body.len(), Ordering::Relaxed);""", """    pub fn send(&self, body: Vec<u8>) {
        if self.paused.load(Ordering::Relaxed) {
            return;
        }
        let Some(body) = self.heard_by_plugins(body) else {
            return;
        };
        self.pending.fetch_add(body.len(), Ordering::Relaxed);"""),
    ('horizon-client/src/lan/conn.rs', """        if !HOLDING.get() {
            bodies.into_iter().for_each(|body| self.send(body));
            return;
        }""", """        if !HOLDING.get() {
            bodies.into_iter().for_each(|body| self.send(body));
            return;
        }
        let bodies: Vec<Vec<u8>> = bodies.into_iter().filter_map(|body| self.heard_by_plugins(body)).collect();"""),
    # The player the sender belongs to, for the packet listeners.
    ('horizon-client/src/lan/guest.rs', """            name: j.name,
            properties: j.properties,""", """            name: {
                let _ = j.sender.owner.set((j.uuid, j.name.clone()));
                j.name
            },
            properties: j.properties,"""),
    # A packet a plugin sends itself: past the listeners.
    ('horizon-client/src/lan/guest.rs', """    pub fn send(&self, packet: &play::Clientbound) {
        if self.anticheat.record.is_some() {
            self.note_change(packet);
        }
        self.sender.send(packet.encode());
    }
""", """    pub fn send(&self, packet: &play::Clientbound) {
        if self.anticheat.record.is_some() {
            self.note_change(packet);
        }
        self.sender.send(packet.encode());
    }
    /// Sends without asking the plugins' packet listeners.
    pub fn send_quietly(&self, packet: &play::Clientbound) {
        if self.anticheat.record.is_some() {
            self.note_change(packet);
        }
        self.sender.send_quiet(packet.encode());
    }
"""),
]
