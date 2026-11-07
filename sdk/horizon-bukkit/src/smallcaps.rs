//! Small capitals (`Teleport Pad` -> `ᴛᴇʟᴇᴘᴏʀᴛ ᴘᴀᴅ`), the lettering that makes menus and messages look finished.
//! Minecraft's own font draws them (they sit in its nonlatin_european.png), so they need no resource pack.
//!
//! - [`small`]: a plain string in small capitals.
//! - [`small_tags`]: the text inside `<small>...</small>` in small capitals, leaving MiniMessage tags (`<white>`),
//!   `{placeholders}` and `%placeholders%` as they are; `\<name>` (an escaped bracket, a command's argument) is
//!   text and is converted too. [`crate::adventure::mini_message`] applies it, so every MiniMessage text a
//!   plugin parses understands `<small>` (horizon-kit's `mini` does the same).
//!
//! ```ignore
//! horizon_bukkit::smallcaps::small("Link to a pad")                 // "ʟɪɴᴋ ᴛᴏ ᴀ ᴘᴀᴅ"
//! mini_message("<green><small>Pad {name} made</small>")             // ᴘᴀᴅ {name} ᴍᴀᴅᴇ, in green
//! ```
//!
//! `s` and `x` stay as they are (the font's own small `ꜱ` is drawn too wide and there is no small x); digits and
//! symbols are unchanged.

/// One letter in small capitals (anything else unchanged).
pub fn small_char(c: char) -> char {
    match c.to_ascii_lowercase() {
        'a' => 'ᴀ',
        'b' => 'ʙ',
        'c' => 'ᴄ',
        'd' => 'ᴅ',
        'e' => 'ᴇ',
        'f' => 'ꜰ',
        'g' => 'ɢ',
        'h' => 'ʜ',
        'i' => 'ɪ',
        'j' => 'ᴊ',
        'k' => 'ᴋ',
        'l' => 'ʟ',
        'm' => 'ᴍ',
        'n' => 'ɴ',
        'o' => 'ᴏ',
        'p' => 'ᴘ',
        'q' => 'ǫ',
        'r' => 'ʀ',
        't' => 'ᴛ',
        'u' => 'ᴜ',
        'v' => 'ᴠ',
        'w' => 'ᴡ',
        'y' => 'ʏ',
        'z' => 'ᴢ',
        's' => 's',
        'x' => 'x',
        _ => c,
    }
}

/// A plain string in small capitals.
pub fn small(text: &str) -> String {
    text.chars().map(small_char).collect()
}

/// [`small_tags`] without a copy when there is no `<small>` (what `mini` calls on every parse).
pub fn apply(text: &str) -> std::borrow::Cow<'_, str> {
    if text.contains("<small>") { std::borrow::Cow::Owned(small_tags(text)) } else { std::borrow::Cow::Borrowed(text) }
}

/// The text inside `<small>...</small>` in small capitals; tags and placeholders inside stay as they are.
pub fn small_tags(text: &str) -> String {
    if !text.contains("<small>") {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut rest = text;
    while let Some(start) = rest.find("<small>") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 7..];
        let (inner, tail) = match after.find("</small>") {
            Some(end) => (&after[..end], &after[end + 8..]),
            None => (after, ""),
        };
        convert(inner, &mut out);
        rest = tail;
    }
    out.push_str(rest);
    out
}

fn convert(inner: &str, out: &mut String) {
    // Inside a <tag>, a {placeholder} or a %placeholder%: copied as it is.
    let mut close: Option<char> = None;
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(end) = close {
            out.push(c);
            if c == end {
                close = None;
            }
            continue;
        }
        match c {
            // `\<name>`: an escaped bracket is text (a command's argument).
            '\\' if chars.peek() == Some(&'<') => {
                out.push('\\');
                out.push(chars.next().unwrap_or('<'));
            }
            '<' => {
                out.push(c);
                close = Some('>');
            }
            '{' => {
                out.push(c);
                close = Some('}');
            }
            '%' if chars.peek().is_some_and(|n| n.is_ascii_alphanumeric() || *n == '_') && inner_has_closing_percent(&chars) => {
                out.push(c);
                close = Some('%');
            }
            _ => out.push(small_char(c)),
        }
    }
}

/// `%name%` ahead (a placeholder), not a lone `50%`.
fn inner_has_closing_percent(chars: &std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    chars.clone().take_while(|c| !c.is_whitespace()).any(|c| c == '%')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain() {
        assert_eq!(small("Teleport Pad"), "ᴛᴇʟᴇᴘᴏʀᴛ ᴘᴀᴅ");
        assert_eq!(small("Wait 3s!"), "ᴡᴀɪᴛ 3s!");
    }

    #[test]
    fn tags_and_placeholders_stay() {
        assert_eq!(small_tags("<small><#4ade80>Pad <white>{name}</white> made</small> ok"), "<#4ade80>ᴘᴀᴅ <white>{name}</white> ᴍᴀᴅᴇ ok");
        assert_eq!(small_tags("<small>Hi %player_name%, 50% off</small>"), "ʜɪ %player_name%, 50% ᴏꜰꜰ");
        assert_eq!(small_tags("<small>/home \\<name></small>"), "/ʜᴏᴍᴇ \\<ɴᴀᴍᴇ>");
        assert_eq!(small_tags("no tag here"), "no tag here");
        assert_eq!(small_tags("<small>open"), "ᴏᴘᴇɴ");
    }

    #[test]
    fn mini_message_understands_small() {
        assert_eq!(crate::adventure::mini_message("<green><small>Pad made</small> ok").plain(), "ᴘᴀᴅ ᴍᴀᴅᴇ ok");
    }
}
