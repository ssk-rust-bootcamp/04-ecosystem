use loom::{
    sync::{atomic::AtomicIsize, Arc},
    thread,
};

use loom::sync::atomic::Ordering::{Acquire, Relaxed, Release};

#[test]
#[should_panic]
fn buggy_concurrent_inc() {
    loom::model(|| {
        let num = Arc::new(AtomicIsize::new(0));

        let handlers: Vec<_> = (0..2)
            .map(|_| {
                let num = num.clone();

                thread::spawn(move || {
                    let curr = num.load(Acquire);
                    // This is a bug! this is not atomic!
                    num.store(curr + 1, Release);

                    // fix
                    // num.fetch_add(1, Relaxed);
                })
            })
            .collect();

        for h in handlers {
            h.join().unwrap();
        }
        assert_eq!(num.load(Relaxed), 2);
    });
}
