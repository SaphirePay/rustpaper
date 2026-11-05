# Server RAM (owner, 11-03: more players on less RAM). Each row's effect is measured with tools/perf_bench.py
# (1, 10, 25, 50, 100 bots, no plugins) against a build without the rows.
LIB = 'horizon-client/src/'
EDITS = [
    # Where the memory is: `horizon-report` now gives each level's chunk bytes beside the process's RSS.
    (LIB + 'game/dedicated/report.rs', """                "chunks": world.chunks.len(),
""", """                "chunks": world.chunks.len(),
                "chunk_heap_mb": world.chunks.values().map(|c| c.heap_bytes()).sum::<usize>() >> 20,
"""),
    # The process's own memory on Windows too: `status`, the panel and `horizon-report` showed none there.
    ('horizon-bench/src/hardware.rs', """/// Resident memory of a process, in bytes (macOS, Linux).
pub fn rss_bytes(pid: u32) -> Option<u64> {
""", """/// Resident memory of a process, in bytes (macOS, Linux; on Windows, this process's own).
pub fn rss_bytes(pid: u32) -> Option<u64> {
    #[cfg(windows)]
    {
        #[repr(C)]
        #[derive(Default)]
        struct Counters {
            size: u32,
            page_faults: u32,
            peak_working_set: usize,
            working_set: usize,
            quota_peak_paged: usize,
            quota_paged: usize,
            quota_peak_non_paged: usize,
            quota_non_paged: usize,
            page_file: usize,
            peak_page_file: usize,
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut core::ffi::c_void;
            fn K32GetProcessMemoryInfo(process: *mut core::ffi::c_void, counters: *mut Counters, size: u32) -> i32;
        }
        if pid != std::process::id() {
            return None;
        }
        let mut counters = Counters { size: size_of::<Counters>() as u32, ..Counters::default() };
        // SAFETY: the struct is PROCESS_MEMORY_COUNTERS, sized as the call asks, alive for the call.
        let read = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &raw mut counters, counters.size) != 0 };
        return read.then_some(counters.working_set as u64);
    }
"""),
    # The generation pipeline's sizes, for `horizon-report`: what explorers leave behind.
    ('horizon-client-render/src/streaming.rs', """pub type Proto = Box<dyn std::any::Any + Send>;
""", """pub type Proto = Box<dyn std::any::Any + Send>;
impl Streaming {
    /// protos held, light views, generating, finishing, decorated, written, lighting, deferred.
    pub fn pipeline_counts(&self) -> [usize; 8] {
        [
            self.protos.values().filter(|p| p.is_some()).count(),
            self.light_views.len(),
            self.generating.len(),
            self.finishing.len(),
            self.decorated.len(),
            self.written.len(),
            self.lighting.len(),
            self.deferred.len(),
        ]
    }
}
"""),
    (LIB + 'game/dedicated/report.rs', """        current["boats"] = json!(self.boats.len());
""", """        current["boats"] = json!(self.boats.len());
        current["pipeline"] = json!(self.streaming.pipeline_counts());
"""),
    # How big a mob is, part by part: ~1,000 mobs held about 190 MB on the tick thread at 50 players.
    (LIB + 'game/dedicated/report.rs', """            "process": process_limits(),
""", """            "process": process_limits(),
            "struct_bytes": {
                "mob": std::mem::size_of::<horizon_client::mobs::Mob>(),
                "extra": std::mem::size_of::<horizon_client::mobs::Extra>(),
                "ai": std::mem::size_of::<horizon_client::mobs::ai::Ai>(),
                "anim": std::mem::size_of::<horizon_client::mobs::anim::AnimState>(),
                "combat": std::mem::size_of::<horizon_client::mobs::combat::Tracker>(),
                "fluids": std::mem::size_of::<horizon_client::mobs::living::Fluids>(),
                "attrs": std::mem::size_of::<horizon_client::mobs::Attrs>(),
            },
"""),
    # And what each step of loading costs, in the log.
    (LIB + 'game.rs', """    log::info!("load core tables: {:.2}s", started.elapsed().as_secs_f32());
""", """    log::info!("load core tables: {:.2}s, {} MB", started.elapsed().as_secs_f32(), horizon_bench::rss_bytes(std::process::id()).unwrap_or(0) >> 20);
"""),
    (LIB + 'game.rs', """    let tables = Arc::new(MeshTables::new(&assets, &render, &facts, visibility)?);
""", """    let tables = Arc::new(MeshTables::new(&assets, &render, &facts, visibility)?);
    log::info!("mesh tables: {} MB", horizon_bench::rss_bytes(std::process::id()).unwrap_or(0) >> 20);
"""),
    (LIB + 'game.rs', """    log::info!("load pack tables: {:.2}s", started.elapsed().as_secs_f32());
""", """    log::info!("load pack tables: {:.2}s, {} MB", started.elapsed().as_secs_f32(), horizon_bench::rss_bytes(std::process::id()).unwrap_or(0) >> 20);
"""),
    (LIB + 'game.rs', """        log::info!("load items: {:.2}s", started.elapsed().as_secs_f32());
""", """        log::info!("load items: {:.2}s, {} MB", started.elapsed().as_secs_f32(), horizon_bench::rss_bytes(std::process::id()).unwrap_or(0) >> 20);
"""),
]
