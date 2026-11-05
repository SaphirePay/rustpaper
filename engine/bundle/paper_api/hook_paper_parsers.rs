// Written by tools/gen_bundle.py.
fn paper_hook_paper_parsers(kind: &str) -> ::core::ops::ControlFlow<Option<(&'static str, Value)>> {
    let parser = match kind {
        "vec3" => "minecraft:vec3",
        "vec2" => "minecraft:vec2",
        "column_pos" => "minecraft:column_pos",
        "rotation" => "minecraft:rotation",
        "message" => "minecraft:message",
        _ => return ::core::ops::ControlFlow::Continue(()),
    };
    ::core::ops::ControlFlow::Break(Some((parser, Value::Null)))
}
