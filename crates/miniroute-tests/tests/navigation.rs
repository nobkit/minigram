use core::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

use embassy_executor::Executor;
use miniroute::{route, router};

static SEQ: AtomicU32 = AtomicU32::new(1);
fn next_seq() -> u32 {
    SEQ.fetch_add(1, Ordering::SeqCst)
}

static R1_ACTIVATIONS: AtomicU32 = AtomicU32::new(0);

static R1_SETUP_1: AtomicU32 = AtomicU32::new(0);
static R1_WORK_1: AtomicU32 = AtomicU32::new(0);
static R1_CLEANUP_1: AtomicU32 = AtomicU32::new(0);

static R1_SETUP_2: AtomicU32 = AtomicU32::new(0);
static R1_WORK_2: AtomicU32 = AtomicU32::new(0);

static R2_SETUP: AtomicU32 = AtomicU32::new(0);
static R2_WORK: AtomicU32 = AtomicU32::new(0);
static R2_CLEANUP: AtomicU32 = AtomicU32::new(0);

#[router]
enum NavRouter {
    #[to(Route1)]
    Route1,
    #[to(Route2)]
    Route2,
}

#[route(router = NavRouter)]
enum Route1 {
    #[task(route1_worker)]
    Worker,
}

#[route(router = NavRouter)]
enum Route2 {
    #[task(route2_worker)]
    Worker,
}

#[embassy_executor::task]
async fn route1_worker(route: Route1) {
    route
        .task(async || {
            let activation = R1_ACTIVATIONS.load(Ordering::SeqCst);
            if activation == 1 {
                // First activation - record work, then navigate away.
                let _ =
                    R1_WORK_1.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
                route.navigate(NavRouter::Route2);
            } else if activation == 2 {
                // Second activation (after round-trip) - record, stay put.
                let _ =
                    R1_WORK_2.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
            }
        })
        .setup(async || {
            let prev = R1_ACTIVATIONS.fetch_add(1, Ordering::SeqCst);
            if prev == 0 {
                R1_SETUP_1.store(next_seq(), Ordering::SeqCst);
            } else if prev == 1 {
                R1_SETUP_2.store(next_seq(), Ordering::SeqCst);
            }
        })
        .cleanup(async || {
            let activation = R1_ACTIVATIONS.load(Ordering::SeqCst);
            if activation == 1 {
                let _ = R1_CLEANUP_1.compare_exchange(
                    0,
                    next_seq(),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
            }
        })
        .run()
        .await;
}

#[embassy_executor::task]
async fn route2_worker(route: Route2) {
    route
        .task(async || {
            let _ = R2_WORK.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
            // Immediately navigate back to Route1.
            route.navigate(NavRouter::Route1);
        })
        .setup(async || {
            let _ = R2_SETUP.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
        })
        .cleanup(async || {
            let _ = R2_CLEANUP.compare_exchange(0, next_seq(), Ordering::SeqCst, Ordering::SeqCst);
        })
        .run()
        .await;
}

/// Verifies a full Route1 -> Route2 -> Route1 rapid-fire navigation cycle.
///
/// This proves:
///  1. Workers receive the stop signal and run their cleanup before the next
///     route starts (no interleaving across routes).
///  2. The entire 8-event sequence is deterministic thanks to cooperative
///     scheduling (select ordering + yield_now).
///  3. A route can be re-activated after a round-trip.
#[test]
fn test_rapid_fire_navigation_1_2_1() {
    thread::spawn(|| {
        let executor = Box::leak(Box::new(Executor::new()));
        executor.run(|spawner| {
            NavRouter::spawn_routes(&spawner);
        });
    });

    // Let the executor poll so __main subscribers are registered.
    thread::sleep(Duration::from_millis(50));

    // Kick off: Route1 -> Route2 -> Route1 (workers navigate automatically).
    NavRouter::start(NavRouter::Route1);

    // Wait for the full 1 -> 2 -> 1 cycle to settle.
    thread::sleep(Duration::from_millis(1000));

    let r1_s1 = R1_SETUP_1.load(Ordering::SeqCst);
    let r1_w1 = R1_WORK_1.load(Ordering::SeqCst);
    let r1_c1 = R1_CLEANUP_1.load(Ordering::SeqCst);
    let r2_s = R2_SETUP.load(Ordering::SeqCst);
    let r2_w = R2_WORK.load(Ordering::SeqCst);
    let r2_c = R2_CLEANUP.load(Ordering::SeqCst);
    let r1_s2 = R1_SETUP_2.load(Ordering::SeqCst);
    let r1_w2 = R1_WORK_2.load(Ordering::SeqCst);

    assert!(r1_s1 > 0, "Route1 first setup should have run");
    assert!(r1_w1 > 0, "Route1 first work should have run");
    assert!(r1_c1 > 0, "Route1 first cleanup should have run");
    assert!(r2_s > 0, "Route2 setup should have run");
    assert!(r2_w > 0, "Route2 work should have run");
    assert!(r2_c > 0, "Route2 cleanup should have run");
    assert!(
        r1_s2 > 0,
        "Route1 second setup should have run (re-activation)"
    );
    assert!(
        r1_w2 > 0,
        "Route1 second work should have run (re-activation)"
    );

    // Expected: R1s1 < R1w1 < R1c1 < R2s < R2w < R2c < R1s2 < R1w2
    assert!(
        r1_s1 < r1_w1,
        "R1 setup1 ({r1_s1}) must precede R1 work1 ({r1_w1})"
    );
    assert!(
        r1_w1 < r1_c1,
        "R1 work1 ({r1_w1}) must precede R1 cleanup1 ({r1_c1})"
    );
    assert!(
        r1_c1 < r2_s,
        "R1 cleanup1 ({r1_c1}) must precede R2 setup ({r2_s}) — navigation boundary"
    );
    assert!(
        r2_s < r2_w,
        "R2 setup ({r2_s}) must precede R2 work ({r2_w})"
    );
    assert!(
        r2_w < r2_c,
        "R2 work ({r2_w}) must precede R2 cleanup ({r2_c})"
    );
    assert!(
        r2_c < r1_s2,
        "R2 cleanup ({r2_c}) must precede R1 setup2 ({r1_s2}) — return navigation boundary"
    );
    assert!(
        r1_s2 < r1_w2,
        "R1 setup2 ({r1_s2}) must precede R1 work2 ({r1_w2})"
    );

    assert_eq!(
        R1_ACTIVATIONS.load(Ordering::SeqCst),
        2,
        "Route1 should have activated exactly twice"
    );
}
