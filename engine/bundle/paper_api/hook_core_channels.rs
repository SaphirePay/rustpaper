// Written by tools/gen_bundle.py.
fn paper_hook_core_channels(host: &LanHost, conn: u32, packet: &S) {
    crate::game::paper_fam_core::note_channels(host, conn, packet);
}
