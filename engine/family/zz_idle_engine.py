# What a server holds with nobody on it. Measured with tools/idle_mem.py against a build without the rows.
EDITS = [
    # The allocator's free pages go back after boot, whose parsing peaks far above what it keeps, and again at a pause.
    ('horizon-threads/src/lib.rs', """/// Give back to the OS the memory the allocator holds free, after something
""", """/// Let the OS take the pages of a process with nothing to do: they stay its own and come back
/// as they're touched. Windows; elsewhere what `release_free_memory` frees is all there is.
pub fn trim_working_set() {
    #[cfg(windows)]
    // SAFETY: the pseudo handle of this process, and the documented values for "as little as you can".
    unsafe {
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, SetProcessWorkingSetSize};
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

/// `mi_option_page_commit_on_demand` in mimalloc.h (index 40 in v3); the bindings don't name it.
const PAGE_COMMIT_ON_DEMAND: libmimalloc_sys::mi_option_t = 40;

/// Whether a new page commits its memory as it fills rather than all at once. A boot's parsing
/// leaves most pages holding a little of what it keeps, so a server boots with this on (262 MB
/// private became 199 MB on a restart, 382 MB became 248 MB on a new world) and turns it off to
/// play: every commit it spreads out is a system call, on the threads a tick waits for.
pub fn commit_on_demand(on: bool) {
    // SAFETY: sets one integer option, which mimalloc reads at each new page.
    unsafe {
        libmimalloc_sys::mi_option_set(PAGE_COMMIT_ON_DEMAND, i32::from(on).into());
    }
}

/// Give back to the OS the memory the allocator holds free, after something
"""),
    ('horizon-client/src/main.rs', """    let (home, side, thread) = if args.iter().any(|a| a == "--dedicated") {
""", """    let (home, side, thread) = if args.iter().any(|a| a == "--dedicated") {
        horizon_threads::commit_on_demand(true);
"""),
    ('horizon-client/src/game/dedicated.rs', """                "Done ({:.3}s)! For help, type \\"help\\"", started.elapsed().as_secs_f64()
            ),
        );
        self.save_game()?;
        self.log_saved_levels();
""", """                "Done ({:.3}s)! For help, type \\"help\\"", started.elapsed().as_secs_f64()
            ),
        );
        self.save_game()?;
        self.log_saved_levels();
        horizon_threads::commit_on_demand(false);
        horizon_threads::release_free_memory();
"""),
    ('horizon-client/src/game/dedicated.rs', """                        "Server empty for {} seconds, pausing", props
                        .pause_when_empty_seconds
                    ),
                );
                self.save_game()?;
""", """                        "Server empty for {} seconds, pausing", props
                        .pause_when_empty_seconds
                    ),
                );
                self.save_game()?;
                horizon_threads::release_free_memory();
                horizon_threads::trim_working_set();
"""),
    # A worker's heap is its own: `mi_collect` reaches only the thread calling it, so each worker empties its own.
    ('horizon-client-render/src/streaming.rs', """                                        mesh_scratch = MeshScratch::default();
                                        released = true;
""", """                                        mesh_scratch = MeshScratch::default();
                                        released = true;
                                        horizon_threads::release_free_memory();
"""),
]
