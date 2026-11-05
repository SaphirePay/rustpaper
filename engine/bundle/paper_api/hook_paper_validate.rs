// Written by tools/gen_bundle.py.
fn paper_hook_paper_validate(c: &mut Connection, uuid: u128, name: &str, properties: &Vec<login::Property>) -> ::core::ops::ControlFlow<Result<Option<Parts>, String>> {
    match crate::paper_hub::validate_login(uuid, name, properties) {
        Some(reason) => {
            login_disconnect(c, &reason);
            ::core::ops::ControlFlow::Break(Ok(None))
        }
        None => ::core::ops::ControlFlow::Continue(()),
    }
}
