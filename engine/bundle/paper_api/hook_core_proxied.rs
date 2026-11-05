// Written by tools/gen_bundle.py.
fn paper_hook_core_proxied(game: &mut crate::game::Game<'_>, command: &str) -> ::core::ops::ControlFlow<bool> {
    match crate::game::paper_fam_core::proxied_line(game, command) {
        Some(ran) => ::core::ops::ControlFlow::Break(ran),
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
