//! A mod loaded at run time. Change GREETING, build it, drop the library in `mods/`, and
//! `/fabricapi reload` plays the change: no game build and no restart.
use horizon_mod::{Answer, Host, Value, feedback, log, tell};

const GREETING: &str = "runtime mod says hello";

fn start(host: Host) {
    log("example_welcome started");
    host.command("runtimehello", 0);
    host.on("join", |event: &Value| {
        tell(event["uuid"].as_str().unwrap_or_default(), GREETING);
        Answer::Pass
    });
    host.on("command", |event: &Value| {
        if event["line"].as_str().unwrap_or_default().starts_with("runtimehello") {
            feedback(GREETING);
        }
        Answer::Pass
    });
    host.on("server_stopping", |_: &Value| {
        log("example_welcome heard the server stopping");
        Answer::Pass
    });
    // Sand refuses nothing and stone is fine: only a break at y = 200 is refused, for the proof.
    host.on("break_before", |event: &Value| {
        if event["pos"][1] == 200 { Answer::Fail } else { Answer::Pass }
    });
}

horizon_mod::mod_entry!(start, include_str!("../fabric.mod.json"));
