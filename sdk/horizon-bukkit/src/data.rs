//! The game's saved data as JSON, and back as SNBT: what the typed calls of entities, block entities
//! and items are written over.
use serde_json::Value;

/// A string as SNBT writes it.
pub fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for letter in text.chars() {
        if matches!(letter, '"' | '\\') {
            out.push('\\');
        }
        out.push(letter);
    }
    out.push('"');
    out
}

/// A value as SNBT: whole numbers as ints and the rest as doubles, as no reader of these minds.
pub fn snbt(value: &Value) -> String {
    match value {
        Value::Null => "{}".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) if number.is_i64() || number.is_u64() => number.to_string(),
        Value::Number(number) => format!("{number}d"),
        Value::String(text) => quoted(text),
        Value::Array(values) => format!("[{}]", values.iter().map(snbt).collect::<Vec<_>>().join(",")),
        Value::Object(values) => format!("{{{}}}", values.iter().map(|(key, value)| format!("{}:{}", quoted(key), snbt(value))).collect::<Vec<_>>().join(",")),
    }
}

/// `{Fuse:5s}` for `Fuse` and `5s`; a dotted path nests.
pub fn patch(path: &str, value: &str) -> String {
    path.rsplit('.').fold(value.to_owned(), |inner, key| format!("{{{key}:{inner}}}"))
}

/// The value at a dotted path.
pub fn at<'a>(root: &'a Value, path: &str) -> &'a Value {
    path.split('.').fold(root, |value, key| &value[key])
}

/// A flag: the game saves them as bytes, and JSON keeps the number.
pub fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or_else(|| value.as_i64().unwrap_or(0) != 0)
}

/// A UUID saved as four ints, dashed as Java prints it.
pub fn uuid(value: &Value) -> Option<String> {
    let words = value.as_array()?;
    let word = |i: usize| words.get(i).and_then(Value::as_i64).map(|word| word as u32);
    let hex = format!("{:08x}{:08x}{:08x}{:08x}", word(0)?, word(1)?, word(2)?, word(3)?);
    Some(format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..]))
}

/// The four ints a UUID saves as, in SNBT.
pub fn uuid_snbt(uuid: &str) -> Option<String> {
    let value = u128::from_str_radix(&uuid.replace('-', ""), 16).ok()?;
    let word = |shift: u32| (value >> shift) as u32 as i32;
    Some(format!("[I;{},{},{},{}]", word(96), word(64), word(32), word(0)))
}
