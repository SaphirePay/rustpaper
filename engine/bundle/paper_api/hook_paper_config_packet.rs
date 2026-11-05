// Written by tools/gen_bundle.py.
fn paper_hook_paper_config_packet(id: i32, payload: &[u8]) {
    crate::paper_hub::config_packet_seen(id, payload);
}
