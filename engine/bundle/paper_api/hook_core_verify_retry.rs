// Written by tools/gen_bundle.py.
fn paper_hook_core_verify_retry(name: &str, server_hash: &str, ip: Option<std::net::IpAddr>) -> ::core::ops::ControlFlow<Result<Option<JoinedProfile>, JoinError>> {
    thread_local! {
        static INSIDE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    if INSIDE.with(std::cell::Cell::get) {
        return ::core::ops::ControlFlow::Continue(());
    }
    INSIDE.with(|inside| inside.set(true));
    let mut result = has_joined(name, server_hash, ip);
    for pause in [250u64, 250, 500, 500, 750, 1000, 1000, 1500, 1500, 2000, 2000, 2000, 2000, 2000] {
        if !matches!(result, Err(JoinError::Unavailable)) {
            break;
        }
        log::warn!("Mojang's session service didn't answer {name}'s check; asking again");
        std::thread::sleep(std::time::Duration::from_millis(pause));
        result = has_joined(name, server_hash, ip);
    }
    INSIDE.with(|inside| inside.set(false));
    ::core::ops::ControlFlow::Break(result)
}
