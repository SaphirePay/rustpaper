// Written by tools/gen_bundle.py.
fn paper_hook_slash_usage(usage: &str) -> ::core::ops::ControlFlow<Result<Vec<(String, Value, bool)>, String>> {
    // A command word with a slash of its own (`/set`, WorldEdit's `//set` as typed): the rest read as any usage,
    // the slash put back on the word, as Paper's command tree has it.
    let Some(rest) = usage.strip_prefix('/') else { return ::core::ops::ControlFlow::Continue(()) };
    if rest.is_empty() || rest.starts_with(['/', ' ']) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(usage_nodes(rest).map(|mut nodes| {
        if let Some(first) = nodes.first_mut() {
            first.0.insert(0, '/');
        }
        nodes
    }))
}
