use anyhow::{anyhow, Ok, Result};
use std::{sync::mpsc, thread, time::Duration};

const NUM_PRODUCERS: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    for i in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx));
    }
    drop(tx);

    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consumer: {:?}", msg);
        }
        println!("consumer exit");
        // 这里可以返回任何实现了Send trait的类型
        42
    });

    // join()可以拿到线程返回的值
    let secret = consumer
        .join()
        .map_err(|err| anyhow!("Thread join error: {:?}", err))?;

    println!("secret: {}", secret);

    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    // 给定死循环+线程sleep让线程不主动退出
    loop {
        // rand0.9不支持usize 使用0.8.5
        let value = rand::random::<usize>(); // 随机数1-2^size
        tx.send(Msg::new(idx, value))?;
        let sleep_time = rand::random::<u8>() as u64 * 10; // 随机数1-127 * 10
        thread::sleep(Duration::from_millis(sleep_time)); // 参数接收u64, 所以需要显式强制类型转换u8 -> u64
        if rand::random::<u8>() % 5 == 0 {
            println!("producer {} exit", idx);
            break;
        }
    }
    // more things to do
    Ok(())
}

impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}
