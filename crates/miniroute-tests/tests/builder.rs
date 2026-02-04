use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use embassy_executor::Executor;
use miniroute::{route, router};

static TASK_ONLY_WORK: AtomicU32 = AtomicU32::new(0);

static WITH_SETUP_WORK: AtomicU32 = AtomicU32::new(0);
static WITH_SETUP_SETUP: AtomicBool = AtomicBool::new(false);

static WITH_CLEANUP_WORK: AtomicU32 = AtomicU32::new(0);
static WITH_CLEANUP_CLEANUP: AtomicBool = AtomicBool::new(false);

static FULL_WORK: AtomicU32 = AtomicU32::new(0);
static FULL_SETUP: AtomicBool = AtomicBool::new(false);
static FULL_CLEANUP: AtomicBool = AtomicBool::new(false);

#[router]
enum BuilderRouter {
    #[to(BuilderRoute)]
    BuilderRoute,
}

#[route(router = BuilderRouter)]
enum BuilderRoute {
    #[task(task_only_worker)]
    TaskOnly,
    #[task(with_setup_worker)]
    WithSetup,
    #[task(with_cleanup_worker)]
    WithCleanup,
    #[task(full_worker)]
    Full,
}

#[embassy_executor::task]
async fn task_only_worker(route: BuilderRoute) {
    route
        .task(async || {
            TASK_ONLY_WORK.fetch_add(1, Ordering::SeqCst);
            route.navigate(BuilderRouter::BuilderRoute);
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn with_setup_worker(route: BuilderRoute) {
    route
        .task(async || {
            WITH_SETUP_WORK.fetch_add(1, Ordering::SeqCst);
            route.navigate(BuilderRouter::BuilderRoute);
        })
        .setup(async || {
            WITH_SETUP_SETUP.store(true, Ordering::SeqCst);
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn with_cleanup_worker(route: BuilderRoute) {
    route
        .task(async || {
            WITH_CLEANUP_WORK.fetch_add(1, Ordering::SeqCst);
            route.navigate(BuilderRouter::BuilderRoute);
        })
        .cleanup(async || {
            WITH_CLEANUP_CLEANUP.store(true, Ordering::SeqCst);
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn full_worker(route: BuilderRoute) {
    route
        .task(async || {
            FULL_WORK.fetch_add(1, Ordering::SeqCst);
            route.navigate(BuilderRouter::BuilderRoute);
        })
        .setup(async || {
            FULL_SETUP.store(true, Ordering::SeqCst);
        })
        .cleanup(async || {
            FULL_CLEANUP.store(true, Ordering::SeqCst);
        })
        .run()
        .await;
}

#[test]
fn test_all_builder_combinations() {
    thread::spawn(|| {
        let executor = Box::leak(Box::new(Executor::new()));
        executor.run(|spawner| {
            BuilderRouter::spawn_routes(&spawner);
        });
    });

    thread::sleep(Duration::from_millis(50));

    BuilderRouter::start(BuilderRouter::BuilderRoute);

    thread::sleep(Duration::from_millis(500));

    assert!(
        TASK_ONLY_WORK.load(Ordering::SeqCst) > 0,
        "task-only worker should have run"
    );

    assert!(
        WITH_SETUP_WORK.load(Ordering::SeqCst) > 0,
        "with-setup worker should have run"
    );
    assert!(
        WITH_SETUP_SETUP.load(Ordering::SeqCst),
        "with-setup setup callback should have fired"
    );

    assert!(
        WITH_CLEANUP_WORK.load(Ordering::SeqCst) > 0,
        "with-cleanup worker should have run"
    );
    assert!(
        WITH_CLEANUP_CLEANUP.load(Ordering::SeqCst),
        "with-cleanup cleanup callback should have fired"
    );

    assert!(
        FULL_WORK.load(Ordering::SeqCst) > 0,
        "full worker should have run"
    );
    assert!(
        FULL_SETUP.load(Ordering::SeqCst),
        "full setup callback should have fired"
    );
    assert!(
        FULL_CLEANUP.load(Ordering::SeqCst),
        "full cleanup callback should have fired"
    );
}
