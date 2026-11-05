//! `org.bukkit.scheduler.BukkitScheduler`, driven by the server's tick.
use horizon_mod::{Answer, Host};
use std::sync::Mutex;

type Task = Box<dyn FnMut() + Send>;

pub(crate) struct Scheduled {
    pub(crate) id: u32,
    at: u64,
    period: Option<u64>,
    task: Task,
}

pub(crate) struct State {
    now: u64,
    next_id: u32,
    pub(crate) tasks: Vec<Scheduled>,
    pub(crate) cancelled: Vec<u32>,
}

pub(crate) static STATE: Mutex<State> = Mutex::new(State { now: 0, next_id: 1, tasks: Vec::new(), cancelled: Vec::new() });

fn add(delay: u64, period: Option<u64>, task: Task) -> u32 {
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let id = state.next_id;
    state.next_id += 1;
    let at = state.now + delay.max(1);
    state.tasks.push(Scheduled { id, at, period, task });
    id
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.runTask, org/bukkit/scheduler/BukkitRunnable.runTask, org/bukkit/scheduler/BukkitScheduler.scheduleSyncDelayedTask
pub fn run_task(task: impl FnMut() + Send + 'static) -> u32 {
    add(1, None, Box::new(task))
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskLater, org/bukkit/scheduler/BukkitRunnable.runTaskLater
pub fn run_task_later(delay: u64, task: impl FnMut() + Send + 'static) -> u32 {
    add(delay, None, Box::new(task))
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskTimer, org/bukkit/scheduler/BukkitRunnable.runTaskTimer, org/bukkit/scheduler/BukkitScheduler.scheduleSyncRepeatingTask
pub fn run_task_timer(delay: u64, period: u64, task: impl FnMut() + Send + 'static) -> u32 {
    add(delay, Some(period.max(1)), Box::new(task))
}

/// Tasks `run_task_asynchronously` started.
static ASYNC_TASKS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

pub(crate) fn is_async(id: u32) -> bool {
    ASYNC_TASKS.lock().unwrap_or_else(|e| e.into_inner()).contains(&id)
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.runTaskAsynchronously, org/bukkit/scheduler/BukkitRunnable.runTaskAsynchronously
pub fn run_task_asynchronously(task: impl FnOnce() + Send + 'static) -> crate::fam_core::BukkitTask {
    let id = {
        let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
        state.next_id += 1;
        state.next_id - 1
    };
    ASYNC_TASKS.lock().unwrap_or_else(|e| e.into_inner()).push(id);
    // 16 MB, as jrt's own threads: Java code converted to Rust goes deep, and with Rust's default 2 MB every call
    // of it made a new stack (`stacker`), ~0.3 ms a call.
    let spawned = std::thread::Builder::new().name("Craft Scheduler Thread".into()).stack_size(16 << 20).spawn(move || {
        ASYNC.with(|flag| flag.set(true));
        let _worker = crate::fam_core::worker_started();
        if !crate::fam_core::BukkitTask::of(id).is_cancelled() {
            task();
        }
    });
    drop(spawned);
    crate::fam_core::BukkitTask::of(id)
}

thread_local! {
    static ASYNC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether this thread came from `run_task_asynchronously`.
pub fn in_async_task() -> bool {
    ASYNC.with(std::cell::Cell::get)
}

/// Folia's calls without a `ScheduledTask`: `fam_paper` has the classes.
pub fn global_run(task: impl FnMut() + Send + 'static) -> u32 {
    run_task(task)
}

/// Folia's calls without a `ScheduledTask`: `fam_paper` has the classes.
pub fn global_run_delayed(delay: u64, task: impl FnMut() + Send + 'static) -> u32 {
    run_task_later(delay, task)
}

/// Folia's calls without a `ScheduledTask`: `fam_paper` has the classes.
pub fn global_run_at_fixed_rate(delay: u64, period: u64, task: impl FnMut() + Send + 'static) -> u32 {
    run_task_timer(delay, period, task)
}

/// Folia's calls without a `ScheduledTask`: `fam_paper` has the classes.
pub fn entity_run(delay: u64, task: impl FnMut() + Send + 'static) -> u32 {
    run_task_later(delay, task)
}

/// Folia's calls without a `ScheduledTask`: `fam_paper` has the classes.
pub fn async_run(task: impl FnOnce() + Send + 'static) {
    run_task_asynchronously(task);
}

/// bukkit: org/bukkit/scheduler/BukkitScheduler.cancelTask, org/bukkit/scheduler/BukkitTask.cancel
pub fn cancel_task(id: u32) {
    crate::fam_core::note_cancelled(id);
    STATE.lock().unwrap_or_else(|e| e.into_inner()).cancelled.push(id);
}

/// Ticks since the server started.
pub fn now() -> u64 {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).now
}

thread_local! {
    pub(crate) static RUNNING: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// bukkit: org/bukkit/scheduler/BukkitRunnable.cancel
///
/// From inside a task: `this.cancel()`.
pub fn cancel_self() {
    cancel_task(RUNNING.with(std::cell::Cell::get));
}

/// One server tick: what is due, in the order it was scheduled, outside the lock so that a task
/// may schedule another.
fn tick() {
    let mut due = {
        let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
        state.now += 1;
        let (now, cancelled) = (state.now, std::mem::take(&mut state.cancelled));
        state.tasks.retain(|task| !cancelled.contains(&task.id));
        let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut state.tasks).into_iter().partition(|task| task.at <= now);
        state.tasks = later;
        due
    };
    for scheduled in &mut due {
        RUNNING.with(|running| running.set(scheduled.id));
        (scheduled.task)();
    }
    let mut state = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let now = state.now;
    for mut scheduled in due {
        if let Some(period) = scheduled.period {
            scheduled.at = now + period;
            state.tasks.push(scheduled);
        }
    }
}

pub(crate) fn install(host: Host) {
    host.on("tick", |_| {
        tick();
        Answer::Pass
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn tasks_run_when_due_and_timers_repeat_until_cancelled() {
        let (later, timer) = (Arc::new(AtomicU32::new(0)), Arc::new(AtomicU32::new(0)));
        let count = later.clone();
        run_task_later(3, move || {
            count.fetch_add(1, Ordering::Relaxed);
        });
        let count = timer.clone();
        let id = run_task_timer(1, 2, move || {
            count.fetch_add(1, Ordering::Relaxed);
        });
        tick();
        tick();
        assert_eq!(later.load(Ordering::Relaxed), 0);
        tick();
        assert_eq!(later.load(Ordering::Relaxed), 1);
        assert_eq!(timer.load(Ordering::Relaxed), 2);
        cancel_task(id);
        tick();
        tick();
        tick();
        assert_eq!(timer.load(Ordering::Relaxed), 2);
        assert_eq!(later.load(Ordering::Relaxed), 1);
    }
}
