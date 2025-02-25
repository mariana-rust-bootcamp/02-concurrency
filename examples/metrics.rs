use std::{thread, time::Duration};

use anyhow::Ok;
use concurrency::Metrics;
use rand::Rng;

const N: usize = 2;
const M: usize = 3;

fn main() {
    let metrics = Metrics::new();

    // 模拟并发任务
    // start N workers and M requests

    println!("{:?}", metrics.clone());

    // 对含有Arc<Mutex<>>的对象进行clone，只会增加引用计数，不会增加锁的所有权; 虽然是不同实例, 但是data是同一个对象
    for i in 0..N {
        task_worker(i, metrics.clone())
    }

    for _ in 0..M {
        request_worker(metrics.clone());
    }

    loop {
        thread::sleep(Duration::from_secs(2));
        println!("{:?}", metrics.snapshot());
    }
}

fn task_worker(idx: usize, matrics: Metrics) {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();

            thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
            matrics.inc(format!("call.thread.worker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok(())
    });
}

fn request_worker(metrics: Metrics) {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();

            thread::sleep(Duration::from_millis(rng.gen_range(50..800)));
            let page = rng.gen_range(1..5);
            metrics.inc(format!("req.page.{}", page))?;
        }
        #[allow(unreachable_code)]
        Ok(())
    });
}
