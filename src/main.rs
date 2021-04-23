use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use tokio::sync::broadcast;

const PAYLOAD_SIZE: usize = 1024;

#[derive(Clone, Copy, Debug)]
struct Message {
    payload: [u8; PAYLOAD_SIZE],
    time: Instant,
}

impl Message {
    pub fn new(x: u8) -> Message {
        Message { payload: [x; PAYLOAD_SIZE], time: Instant::now() }
    }
}

struct ProcessingElement {
    tx: broadcast::Sender<Message>,
    rx: broadcast::Receiver<Message>,
}

#[tokio::main]
async fn main() {
    let iters = env::args().nth(1).unwrap().parse::<usize>().unwrap();
    let (ping_tx, ping_rx) = broadcast::channel(2);
    let (pong_tx, pong_rx) = broadcast::channel(2);
    let mut pe1 = ProcessingElement { tx: ping_tx, rx: pong_rx };
    let mut pe2 = ProcessingElement { tx: pong_tx, rx: ping_rx };

    let ping = tokio::spawn(async move {
        let mut times = Vec::with_capacity(iters);
        let start = Instant::now();
        for i in 0..iters {
            let value = i as u8;
            let message = Message::new(value);
            pe1.tx.send(message).unwrap();
            let response = pe1.rx.recv().await.unwrap();

            let rtt = Instant::now() - message.time;
            times.push(rtt);

            if response.payload[0] != value {
                panic!("Desync occurred.");
            }
        }
        let end = Instant::now();
        println!("{} iterations took {:?}", iters, end - start);

        let mut file = File::create("iter_times").unwrap();
        for t in &times {
            writeln!(file, "{:.3?}", t).unwrap();
        }
    });

    let pong = tokio::spawn(async move {
        for _ in 0..iters {
            let message = pe2.rx.recv().await.unwrap();
            let response = Message::new(message.payload[0]);
            pe2.tx.send(response).unwrap();
        }
    });


    ping.await.unwrap();
    pong.await.unwrap();
}
