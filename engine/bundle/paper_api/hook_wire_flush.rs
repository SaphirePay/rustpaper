// Written by tools/gen_bundle.py.
fn paper_hook_wire_flush(conn: &mut Connection) {
    for body in horizon_core::paper_wire::wire::take_out() {
        let _ = conn.send(&body);
    }
}
