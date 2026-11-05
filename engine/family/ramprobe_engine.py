# Measuring build only (HORIZON_RAM_PROBE=1, see fabric-port/tools/dim_patch.py): every allocation carries a
# header naming who made it: the kind of thread, or, inside a tick phase (`spans::time`), that phase. So
# `horizon-report` gives the live heap by thread and by tick phase beside the RSS.
LIB = 'horizon-client/src/'
EDITS = [
    ('horizon-threads/src/lib.rs', """pub use mimalloc::MiMalloc as Allocator;
""", """pub struct Allocator;

/// Live heap bytes by tag: 0..5 the kind of thread (worker, server, network, main, other), 8.. a tick phase.
pub static HEAP_BY_TAG: [std::sync::atomic::AtomicIsize; 72] = [const { std::sync::atomic::AtomicIsize::new(0) }; 72];
pub const HEAP_ROLES: [&str; 5] = ["worker", "server", "network", "main", "other"];
/// The tick phases' names, by slot (tag 8 + slot): registered without allocating.
static PHASES: [std::sync::atomic::AtomicPtr<u8>; 64] = [const { std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()) }; 64];
static PHASE_LENS: [std::sync::atomic::AtomicUsize; 64] = [const { std::sync::atomic::AtomicUsize::new(0) }; 64];

thread_local! {
    static HEAP_ROLE: std::cell::Cell<u8> = const { std::cell::Cell::new(255) };
    static HEAP_NAMING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static HEAP_PHASE: std::cell::Cell<u8> = const { std::cell::Cell::new(255) };
}

/// The slot of a phase name, registering it the first time (no allocation).
fn phase_slot(name: &'static str) -> u8 {
    for (slot, held) in PHASES.iter().enumerate() {
        let known = held.load(std::sync::atomic::Ordering::Acquire);
        if known == name.as_ptr() as *mut u8 {
            return slot as u8;
        }
        if known.is_null() {
            if held.compare_exchange(std::ptr::null_mut(), name.as_ptr() as *mut u8, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).is_ok() {
                PHASE_LENS[slot].store(name.len(), std::sync::atomic::Ordering::Release);
                return slot as u8;
            }
            if held.load(std::sync::atomic::Ordering::Acquire) == name.as_ptr() as *mut u8 {
                return slot as u8;
            }
        }
    }
    255
}

/// Inside `spans::time(name, ..)`: what's allocated is the phase's. Gives back the phase it replaced.
pub fn enter_phase(name: &'static str) -> u8 {
    let slot = phase_slot(name);
    HEAP_PHASE.try_with(|p| p.replace(slot)).unwrap_or(255)
}

pub fn leave_phase(previous: u8) {
    let _ = HEAP_PHASE.try_with(|p| p.set(previous));
}

fn heap_tag() -> u8 {
    let phase = HEAP_PHASE.try_with(std::cell::Cell::get).unwrap_or(255);
    if phase != 255 {
        return 8 + phase;
    }
    HEAP_ROLE
        .try_with(|r| {
            let known = r.get();
            if known != 255 {
                return known;
            }
            // Naming the thread may allocate: that allocation counts as "other".
            if HEAP_NAMING.with(|n| n.replace(true)) {
                return 4;
            }
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("").to_ascii_lowercase();
            let role = if name.starts_with("horizon-worker") {
                0
            } else if name.contains("server") {
                1
            } else if name.contains("net") || name.contains("conn") || name.contains("lan") || name.contains("io") {
                2
            } else if name == "main" {
                3
            } else {
                4
            };
            HEAP_NAMING.with(|n| n.set(false));
            r.set(role);
            role
        })
        .unwrap_or(4)
}

/// The live heap by thread kind and by tick phase, in MB (phases under 1 MB left out).
pub fn heap_by_role() -> Vec<(String, i64)> {
    let mb = |tag: usize| (HEAP_BY_TAG[tag].load(std::sync::atomic::Ordering::Relaxed) >> 20) as i64;
    let mut out: Vec<(String, i64)> = HEAP_ROLES.iter().enumerate().map(|(tag, name)| ((*name).to_owned(), mb(tag))).collect();
    for slot in 0..64 {
        let ptr = PHASES[slot].load(std::sync::atomic::Ordering::Acquire);
        if ptr.is_null() || mb(8 + slot) == 0 {
            continue;
        }
        let len = PHASE_LENS[slot].load(std::sync::atomic::Ordering::Acquire);
        // SAFETY: the pointer and length of a &'static str registered by `phase_slot`.
        let name = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
        out.push((format!("phase {name}"), mb(8 + slot)));
    }
    out
}

const HEAP_HEAD: usize = 16;

// SAFETY: forwards to mimalloc with room in front of each block for its tag; `dealloc` gets the same
// layout `alloc` did, so it finds the same front.
unsafe impl std::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let front = HEAP_HEAD.max(layout.align());
        let Ok(whole) = std::alloc::Layout::from_size_align(layout.size() + front, layout.align().max(HEAP_HEAD)) else {
            return std::ptr::null_mut();
        };
        let base = unsafe { mimalloc::MiMalloc.alloc(whole) };
        if base.is_null() {
            return base;
        }
        let tag = heap_tag();
        unsafe { *base = tag };
        HEAP_BY_TAG[(tag as usize).min(71)].fetch_add(layout.size() as isize, std::sync::atomic::Ordering::Relaxed);
        unsafe { base.add(front) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        let front = HEAP_HEAD.max(layout.align());
        let base = unsafe { ptr.sub(front) };
        let tag = unsafe { *base } as usize;
        HEAP_BY_TAG[tag.min(71)].fetch_sub(layout.size() as isize, std::sync::atomic::Ordering::Relaxed);
        let whole = unsafe { std::alloc::Layout::from_size_align_unchecked(layout.size() + front, layout.align().max(HEAP_HEAD)) };
        unsafe { mimalloc::MiMalloc.dealloc(base, whole) };
    }
}
"""),
    # Every tick phase tags what it allocates, timed or not.
    ('horizon-client-render/src/spans.rs', """pub fn time<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    if !ENABLED.load(Ordering::Relaxed) {
        return f();
    }
""", """pub fn time<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    let phase = horizon_threads::enter_phase(name);
    let out = time_inner(name, f);
    horizon_threads::leave_phase(phase);
    out
}
#[inline]
fn time_inner<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    if !ENABLED.load(Ordering::Relaxed) {
        return f();
    }
"""),
    (LIB + 'game/dedicated/report.rs', """            "memory": { "rss_mb": horizon_bench::rss_bytes(std::process::id()).map(|b| b >> 20) },
""", """            "memory": { "rss_mb": horizon_bench::rss_bytes(std::process::id()).map(|b| b >> 20), "heap_mb_by_thread": horizon_threads::heap_by_role() },
"""),
]
