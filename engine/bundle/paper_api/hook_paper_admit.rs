// Written by tools/gen_bundle.py.
fn paper_hook_paper_admit(shared: &Shared, address: SocketAddr, uuid: u128, name: &str) -> ::core::ops::ControlFlow<Option<horizon_world::nbt::Tag>> {
    let Some(refusal) = shared.config.gate.as_ref().and_then(|gate| gate(address, uuid, name)) else {
        return ::core::ops::ControlFlow::Continue(());
    };
    match crate::paper_hub::validate_refused(uuid, name, &refusal) {
        None => ::core::ops::ControlFlow::Break(Some(refusal)),
        Some(Some(reason)) => ::core::ops::ControlFlow::Break(Some(reason)),
        Some(None) if shared.reserved.lock().unwrap().contains(&uuid) || shared.is_online(uuid) => ::core::ops::ControlFlow::Break(Some(translatable("multiplayer.disconnect.duplicate_login", &[]))),
        Some(None) => {
            shared.online.lock().unwrap().push((uuid, name.to_owned()));
            ::core::ops::ControlFlow::Break(None)
        }
    }
}
