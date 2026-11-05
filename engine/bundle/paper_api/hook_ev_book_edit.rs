// Written by tools/gen_bundle.py.
fn paper_hook_ev_book_edit(game: &mut crate::game::Game<'_>, host: &LanHost, conn: u32, p: &play::EditBook) -> ::core::ops::ControlFlow<()> {
    if crate::game::paper_bridge::hears("book_edit") {
        let payload = serde_json::json!({ "uuid": crate::game::paper_bridge::uuid_of(host, conn), "pages": p.pages, "title": p.title, "slot": p.slot });
        if crate::game::paper_bridge::on_refusable(game, "book_edit", payload) {
            return ::core::ops::ControlFlow::Break(());
        }
    }
    ::core::ops::ControlFlow::Continue(())
}
