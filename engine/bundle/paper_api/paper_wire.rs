// What the connection threads share with the protocol crate below the game, for Paper's connection classes:
// packets queued for a connection, the configuration tasks that hold one back, and what a connection said before it's
// a player.

/// What the connection threads share with the protocol crate below the game: outgoing packets
/// queued by listeners, and the configuration tasks that hold a connection back.
pub mod wire {
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeSet,
        sync::{Arc, RwLock},
    };
    type Answer = dyn Fn(&str, &[u8]) -> Option<Vec<u8>> + Send + Sync;
    type ConfigPacket = dyn Fn(i32, &[u8]) + Send + Sync;
    static ANSWER: RwLock<Option<Arc<Answer>>> = RwLock::new(None);
    static CONFIG: RwLock<Option<Arc<ConfigPacket>>> = RwLock::new(None);
    thread_local! {
        static CLIENT_LOGIN: Cell<bool> = const { Cell::new(false) };
        static OUT: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
        static TASKS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    }
    pub fn mark_client_login() {
        CLIENT_LOGIN.with(|c| c.set(true));
    }
    pub fn client_login() -> bool {
        CLIENT_LOGIN.with(Cell::get)
    }
    pub fn set_answer(answer: Arc<Answer>) {
        *ANSWER.write().unwrap_or_else(|e| e.into_inner()) = Some(answer);
    }
    pub fn answer(channel: &str, data: &[u8]) -> Option<Vec<u8>> {
        let answer = ANSWER.read().unwrap_or_else(|e| e.into_inner()).clone()?;
        answer(channel, data)
    }
    pub fn set_config(config: Arc<ConfigPacket>) {
        *CONFIG.write().unwrap_or_else(|e| e.into_inner()) = Some(config);
    }
    pub fn config_packet(id: i32, data: &[u8]) {
        let config = CONFIG.read().unwrap_or_else(|e| e.into_inner()).clone();
        if let Some(config) = config {
            config(id, data);
        }
    }
    pub fn push_out(body: Vec<u8>) {
        OUT.with(|o| o.borrow_mut().push(body));
    }
    pub fn take_out() -> Vec<Vec<u8>> {
        OUT.with(|o| std::mem::take(&mut *o.borrow_mut()))
    }
    pub fn task_add(key: &str) {
        TASKS.with(|t| t.borrow_mut().insert(key.to_owned()));
    }
    pub fn task_done(key: &str) {
        TASKS.with(|t| t.borrow_mut().remove(key));
    }
    pub fn tasks_pending() -> bool {
        TASKS.with(|t| !t.borrow().is_empty())
    }
}

/// A connection before it's a player, for Paper's connection classes: the first packets a client sends,
/// and what to do as configuration ends, which only the protocol crate's `send` can see.
pub mod connection {
    use std::cell::{Cell, RefCell};
    thread_local! {
        /// 0 nothing, 1 the handshake is next, 2 the login hello is.
        static EXPECTING: Cell<u8> = const { Cell::new(0) };
        static HANDSHAKE: RefCell<Option<(i32, String, u16, i32)>> = const { RefCell::new(None) };
        static HELLO: RefCell<Option<(String, u128)>> = const { RefCell::new(None) };
    }
    pub fn expect_handshake() {
        EXPECTING.set(1);
    }
    pub fn expecting() -> u8 {
        EXPECTING.get()
    }
    pub fn note_handshake(protocol: i32, host: String, port: u16, intent: i32) {
        EXPECTING.set(2);
        HANDSHAKE.with(|seen| *seen.borrow_mut() = Some((protocol, host, port, intent)));
    }
    pub fn note_hello(name: String, uuid: u128) {
        EXPECTING.set(0);
        HELLO.with(|seen| *seen.borrow_mut() = Some((name, uuid)));
    }
    pub fn handshake() -> Option<(i32, String, u16, i32)> {
        HANDSHAKE.with(|seen| seen.borrow().clone())
    }
    pub fn hello() -> Option<(String, u128)> {
        HELLO.with(|seen| seen.borrow().clone())
    }
}
