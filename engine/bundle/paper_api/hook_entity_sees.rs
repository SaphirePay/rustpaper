// Written by tools/gen_bundle.py.
fn paper_hook_entity_sees(v: &Viewer, p: &Probe) -> ::core::ops::ControlFlow<bool> {
    if crate::paper_hub::hidden_from(v.entity_id, p.id) {
        return ::core::ops::ControlFlow::Break(false);
    }
    ::core::ops::ControlFlow::Continue(())
}
