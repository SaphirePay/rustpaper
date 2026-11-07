//! `net.kyori.adventure`: components, and MiniMessage, which most modern Paper plugins write their
//! messages in. A component here is the game's own text JSON.
use serde_json::{Map, Value, json};

/// bukkit: net/kyori/adventure/text/Component, net/kyori/adventure/text/TextComponent
#[derive(Clone, Debug, PartialEq)]
pub struct Component(pub Value);

impl Component {
    /// bukkit: net/kyori/adventure/text/Component.text
    pub fn text(text: &str) -> Self {
        Self(json!({ "text": text }))
    }
    /// bukkit: net/kyori/adventure/text/Component.empty
    pub fn empty() -> Self {
        Self::text("")
    }
    /// The words alone, as `PlainTextComponentSerializer` gives them.
    pub fn plain(&self) -> String {
        fn walk(value: &Value, out: &mut String) {
            if let Some(text) = value["text"].as_str() {
                out.push_str(text);
            }
            for child in value["extra"].as_array().into_iter().flatten() {
                walk(child, out);
            }
        }
        let mut out = String::new();
        walk(&self.0, &mut out);
        out
    }
}

impl Component {
    /// bukkit: net/kyori/adventure/text/serializer/legacy/LegacyComponentSerializer.serialize
    ///
    /// The text with its `\u{a7}` codes, as Bukkit's `String` calls hand it over.
    pub fn to_legacy(&self) -> String {
        fn code(key: &str) -> Option<char> {
            Some(match key {
                "bold" => 'l',
                "italic" => 'o',
                "underlined" => 'n',
                "strikethrough" => 'm',
                "obfuscated" => 'k',
                _ => return None,
            })
        }
        fn walk(value: &Value, out: &mut String) {
            if let Some(text) = value.as_str() {
                out.push_str(text);
                return;
            }
            if let Some(color) = value["color"].as_str() {
                if let Some(hex) = color.strip_prefix('#') {
                    out.push_str("\u{a7}x");
                    for digit in hex.chars() {
                        out.push('\u{a7}');
                        out.push(digit);
                    }
                } else if let Some(index) = NAMED.iter().position(|name| *name == color) {
                    out.push('\u{a7}');
                    out.extend(char::from_digit(index as u32, 16));
                }
            }
            for key in ["obfuscated", "bold", "strikethrough", "underlined", "italic"] {
                if value[key].as_bool() == Some(true) || value[key].as_i64() == Some(1) {
                    out.push('\u{a7}');
                    out.extend(code(key));
                }
            }
            if let Some(text) = value["text"].as_str() {
                out.push_str(text);
            }
            for child in value["extra"].as_array().into_iter().flatten() {
                walk(child, out);
            }
        }
        let mut out = String::new();
        walk(&self.0, &mut out);
        out
    }
}

const NAMED: [&str; 16] = [
    "black", "dark_blue", "dark_green", "dark_aqua", "dark_red", "dark_purple", "gold", "gray", "dark_gray", "blue", "green", "aqua", "red", "light_purple", "yellow", "white",
];

fn decoration(tag: &str) -> Option<&'static str> {
    Some(match tag {
        "bold" | "b" => "bold",
        "italic" | "i" | "em" => "italic",
        "underlined" | "u" => "underlined",
        "strikethrough" | "st" => "strikethrough",
        "obfuscated" | "obf" => "obfuscated",
        _ => return None,
    })
}

fn color(tag: &str) -> Option<String> {
    let name = tag.strip_prefix("color:").or(tag.strip_prefix("colour:")).or(tag.strip_prefix("c:")).unwrap_or(tag);
    let name = if name == "grey" { "gray" } else if name == "dark_grey" { "dark_gray" } else { name };
    if NAMED.contains(&name) {
        return Some(name.to_owned());
    }
    (name.len() == 7 && name.starts_with('#') && name[1..].bytes().all(|b| b.is_ascii_hexdigit())).then(|| name.to_owned())
}

fn unquote(text: &str) -> &str {
    text.strip_prefix('\'').and_then(|t| t.strip_suffix('\'')).or(text.strip_prefix('"').and_then(|t| t.strip_suffix('"'))).unwrap_or(text)
}

/// What a tag sets on the text after it: a key and its value.
fn style_of(tag: &str) -> Option<(String, Value)> {
    if let Some(key) = decoration(tag) {
        return Some((key.to_owned(), json!(true)));
    }
    if let Some(key) = tag.strip_prefix('!').and_then(decoration) {
        return Some((key.to_owned(), json!(false)));
    }
    // `<italic:false>`, `<bold:true>`: a decoration with its state written out.
    if let Some((name, state)) = tag.split_once(':') {
        if let (Some(key), Some(on)) = (decoration(name), match state { "true" => Some(true), "false" => Some(false), _ => None }) {
            return Some((key.to_owned(), json!(on)));
        }
    }
    if let Some(color) = color(tag) {
        return Some(("color".to_owned(), json!(color)));
    }
    if let Some(rest) = tag.strip_prefix("click:") {
        let (action, value) = rest.split_once(':')?;
        let value = unquote(value);
        let (key, value) = match action {
            "run_command" | "suggest_command" => ("command", value),
            "open_url" => ("url", value),
            "copy_to_clipboard" => ("value", value),
            _ => return None,
        };
        return Some(("click_event".to_owned(), json!({ "action": action, key: value })));
    }
    if let Some(rest) = tag.strip_prefix("hover:show_text:") {
        return Some(("hover_event".to_owned(), json!({ "action": "show_text", "value": mini_message(unquote(rest)).0 })));
    }
    if let Some(rest) = tag.strip_prefix("insert:").or(tag.strip_prefix("insertion:")) {
        return Some(("insertion".to_owned(), json!(unquote(rest))));
    }
    None
}

fn tag_name(tag: &str) -> &str {
    let tag = tag.strip_prefix('!').unwrap_or(tag);
    tag.split(':').next().unwrap_or(tag)
}

/// bukkit~: net/kyori/adventure/text/minimessage/MiniMessage.deserialize, net/kyori/adventure/text/minimessage/MiniMessage.miniMessage | colors, decorations, click, hover and insertion; gradient and rainbow draw their first color only; `<small>` is small capitals (Horizon's own tag)
pub fn mini_message(input: &str) -> Component {
    // `<small>text</small>`: small capitals (crate::smallcaps), before the tags are read.
    let input = &*crate::smallcaps::apply(input);
    let mut parts: Vec<Value> = Vec::new();
    // The open tags, each with what it set.
    let mut open: Vec<(String, String, Value)> = Vec::new();
    let mut text = String::new();
    let flush = |text: &mut String, open: &[(String, String, Value)], parts: &mut Vec<Value>| {
        if text.is_empty() {
            return;
        }
        let mut part = Map::new();
        part.insert("text".to_owned(), json!(std::mem::take(text)));
        for (_, key, value) in open {
            part.insert(key.clone(), value.clone());
        }
        parts.push(Value::Object(part));
    };
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && matches!(chars.get(i + 1), Some('<' | '\\')) {
            text.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if chars[i] != '<' {
            text.push(chars[i]);
            i += 1;
            continue;
        }
        // A tag runs to its `>`, and a quoted value may hold one.
        let mut end = i + 1;
        let mut quote = None;
        // A `<` before the `>` means this one was text: a usage line's `<true<red>|false>`.
        while end < chars.len() && (chars[end] != '>' || quote.is_some()) && (chars[end] != '<' || quote.is_some()) {
            match quote {
                Some(q) if chars[end] == q => quote = None,
                None if matches!(chars[end], '\'' | '"') => quote = Some(chars[end]),
                _ => {}
            }
            end += 1;
        }
        if end >= chars.len() || chars[end] == '<' {
            text.push('<');
            i += 1;
            continue;
        }
        let tag: String = chars[i + 1..end].iter().collect();
        i = end + 1;
        let lower = tag.to_ascii_lowercase();
        if lower == "reset" || lower == "r" {
            flush(&mut text, &open, &mut parts);
            open.clear();
        } else if lower == "newline" || lower == "br" {
            text.push('\n');
        } else if let Some(closing) = lower.strip_prefix('/') {
            let name = tag_name(closing);
            if let Some(at) = open.iter().rposition(|(opened, _, _)| tag_name(opened) == name || (color(closing).is_some() && color(opened).is_some())) {
                flush(&mut text, &open, &mut parts);
                open.remove(at);
            } else {
                text.push_str(&format!("<{tag}>"));
            }
        } else if lower.starts_with("gradient") || lower.starts_with("rainbow") {
            flush(&mut text, &open, &mut parts);
            let first = lower.split(':').nth(1).and_then(color).unwrap_or_else(|| "white".to_owned());
            open.push(("gradient".to_owned(), "color".to_owned(), json!(first)));
        } else if let Some((key, value)) = style_of(&lower).filter(|_| !lower.contains(':')).or_else(|| style_of(&tag)) {
            flush(&mut text, &open, &mut parts);
            open.push((lower.clone(), key, value));
        } else {
            // MiniMessage leaves what isn't a tag as it stands.
            text.push_str(&format!("<{tag}>"));
        }
    }
    flush(&mut text, &open, &mut parts);
    match parts.len() {
        0 => Component::empty(),
        1 => Component(parts.remove(0)),
        _ => Component(json!({ "text": "", "extra": parts })),
    }
}

/// `LegacyComponentSerializer`, as Paper reads a `sendMessage(String)`: section sign codes to a
/// component, where a color ends the decorations before it.
pub fn legacy(text: &str) -> Component {
    const COLORS: [(char, &str); 16] = [
        ('0', "black"), ('1', "dark_blue"), ('2', "dark_green"), ('3', "dark_aqua"), ('4', "dark_red"), ('5', "dark_purple"), ('6', "gold"), ('7', "gray"),
        ('8', "dark_gray"), ('9', "blue"), ('a', "green"), ('b', "aqua"), ('c', "red"), ('d', "light_purple"), ('e', "yellow"), ('f', "white"),
    ];
    let mut parts: Vec<Value> = Vec::new();
    let mut style = Map::new();
    let mut current = String::new();
    let flush = |current: &mut String, style: &Map<String, Value>, parts: &mut Vec<Value>| {
        if !current.is_empty() {
            let mut part = style.clone();
            part.insert("text".to_owned(), json!(std::mem::take(current)));
            parts.push(Value::Object(part));
        }
    };
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\u{a7}' {
            current.push(c);
            continue;
        }
        let Some(code) = chars.next().map(|code| code.to_ascii_lowercase()) else {
            break;
        };
        flush(&mut current, &style, &mut parts);
        // `§x§R§R§G§G§B§B`: a hex color, as CraftChatMessage reads it (titles, `sendMessage(String)`).
        if code == 'x' {
            let mut ahead = chars.clone();
            let hex: Option<String> = (0..6)
                .map(|_| match (ahead.next(), ahead.next()) {
                    (Some('\u{a7}'), Some(digit)) if digit.is_ascii_hexdigit() => Some(digit.to_ascii_uppercase()),
                    _ => None,
                })
                .collect();
            if let Some(hex) = hex {
                chars = ahead;
                style.clear();
                style.insert("color".to_owned(), json!(format!("#{hex}")));
                continue;
            }
        }
        if let Some((_, name)) = COLORS.iter().find(|(key, _)| *key == code) {
            style.clear();
            style.insert("color".to_owned(), json!(name));
        } else if code == 'r' {
            style.clear();
        } else if let Some(key) = match code {
            'l' => Some("bold"),
            'o' => Some("italic"),
            'n' => Some("underlined"),
            'm' => Some("strikethrough"),
            'k' => Some("obfuscated"),
            _ => None,
        } {
            style.insert(key.to_owned(), json!(true));
        }
    }
    flush(&mut current, &style, &mut parts);
    match parts.len() {
        0 => Component::empty(),
        1 => Component(parts.remove(0)),
        _ => Component(json!({ "text": "", "extra": parts })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_codes_become_a_component() {
        let text = legacy("\u{a7}7[\u{a7}6RPC\u{a7}7] \u{a7}l\u{a7}aHi \u{a7}lbold");
        assert_eq!(text.plain(), "[RPC] Hi bold");
        assert_eq!(text.0["extra"][1], json!({ "text": "RPC", "color": "gold" }));
        assert_eq!(text.0["extra"][3], json!({ "text": "Hi ", "color": "green" }), "a color ends the bold before it");
        assert_eq!(text.0["extra"][4]["bold"], json!(true));
    }

    #[test]
    fn legacy_hex_colors() {
        let text = legacy("\u{a7}x\u{a7}5\u{a7}d\u{a7}a\u{a7}d\u{a7}e\u{a7}2\u{a7}lA\u{a7}x\u{a7}1\u{a7}a\u{a7}b\u{a7}c\u{a7}9\u{a7}cB");
        assert_eq!(text.0["extra"][0], json!({ "text": "A", "color": "#5DADE2", "bold": true }));
        assert_eq!(text.0["extra"][1], json!({ "text": "B", "color": "#1ABC9C" }));
        assert_eq!(legacy("\u{a7}xno").plain(), "no", "a bare x is no hex");
    }

    #[test]
    fn nested_tags_stack_and_close() {
        let text = mini_message("<green>You have <yellow>{name}</yellow> left</green>");
        assert_eq!(text.0, json!({ "text": "", "extra": [
            { "text": "You have ", "color": "green" },
            { "text": "{name}", "color": "yellow" },
            { "text": " left", "color": "green" },
        ]}));
        assert_eq!(text.plain(), "You have {name} left");
    }

    #[test]
    fn decorations_hex_and_reset() {
        let text = mini_message("<bold><#ff0000>a<reset>b");
        assert_eq!(text.0["extra"][0], json!({ "text": "a", "bold": true, "color": "#ff0000" }));
        assert_eq!(text.0["extra"][1], json!({ "text": "b" }));
    }

    #[test]
    fn what_is_not_a_tag_stays_text() {
        assert_eq!(mini_message("a <b c").plain(), "a <b c");
        assert_eq!(mini_message("1 <notatag> 2").plain(), "1 <notatag> 2");
        assert_eq!(mini_message("\\<red>").plain(), "<red>");
    }

    #[test]
    fn click_and_hover_carry_their_values() {
        let text = mini_message("<click:run_command:'/home a'><hover:show_text:'<red>go'>x");
        assert_eq!(text.0["click_event"], json!({ "action": "run_command", "command": "/home a" }));
        assert_eq!(text.0["hover_event"]["value"], json!({ "text": "go", "color": "red" }));
    }
}
