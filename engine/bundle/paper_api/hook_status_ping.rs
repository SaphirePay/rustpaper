// Written by tools/gen_bundle.py.
fn paper_hook_status_ping(shared: &Shared) -> ::core::ops::ControlFlow<String> {
    match crate::paper_hub::status_ping(|| shared.status_json()) {
        Some(json) => ::core::ops::ControlFlow::Break(json),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
