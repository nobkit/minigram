use core::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use embassy_executor::Executor;
use miniroute::{RouteHooks, route, router};

static SETUP_ORDER: AtomicU32 = AtomicU32::new(0);
static WORK_ORDER: AtomicU32 = AtomicU32::new(0);
static CLEANUP_ORDER: AtomicU32 = AtomicU32::new(0);
static SEQUENCE: AtomicU32 = AtomicU32::new(1);

fn next_seq() -> u32 {
    SEQUENCE.fetch_add(1, Ordering::SeqCst)
}

#[router]
enum TestRouter {
    #[to(TestRoute)]
    TestRoute,
}

#[route(router = TestRouter, hooks)]
enum TestRoute {
    #[task(lifecycle_worker)]
    Worker,
}

impl RouteHooks for TestRoute {
    async fn setup() {}
    async fn cleanup() {}
}

#[embassy_executor::task]
async fn lifecycle_worker(route: TestRoute) {
    route
        .task(async || {
            let _ = WORK_ORDER.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
            route.navigate(TestRouter::TestRoute);
        })
        .setup(async || {
            let _ = SETUP_ORDER.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
        })
        .cleanup(async || {
            let _ =
                CLEANUP_ORDER.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
        })
        .run()
        .await;
}

#[test]
fn test_setup_work_cleanup_ordering() {
    thread::spawn(|| {
        let executor = Box::leak(Box::new(Executor::new()));
        executor.run(|spawner| {
            TestRouter::spawn_routes(&spawner);
        });
    });

    thread::sleep(Duration::from_millis(50));

    TestRouter::start(TestRouter::TestRoute);

    thread::sleep(Duration::from_millis(500));

    let setup = SETUP_ORDER.load(Ordering::SeqCst);
    let work = WORK_ORDER.load(Ordering::SeqCst);
    let cleanup = CLEANUP_ORDER.load(Ordering::SeqCst);

    assert!(setup > 0, "setup callback should have run");
    assert!(work > 0, "work callback should have run");
    assert!(cleanup > 0, "cleanup callback should have run");

    assert!(
        setup < work,
        "setup ({setup}) must run before work ({work})"
    );
    assert!(
        work < cleanup,
        "work ({work}) must run before cleanup ({cleanup})"
    );
}
