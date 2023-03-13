use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    tx: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(n: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));

        let mut threads = Vec::with_capacity(n);
        for _ in 0..n {
            let rx = Arc::clone(&rx);
            threads.push(thread::spawn(move || {
                println!("thread started");
                loop {
                    let msg = rx.lock().unwrap().recv();
                    println!("got msg");
                    match msg {
                        Ok(job) => job(),
                        Err(_) => break,
                    }
                }
            }));
        }

        ThreadPool { tx }
    }

    fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.send(Box::new(f)).unwrap();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let tp = ThreadPool::new(10);

    for stream in listener.incoming() {
        println!("ko?");
        tp.submit(move || {
            let mut stream = stream.unwrap();
            let mut br = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                br.read_line(&mut line).unwrap();
                // println!("line: {}", &line);
                match line.as_str() {
                    "ping\r\n" => {
                        stream.write("+PONG\r\n".as_bytes()).unwrap();
                    }
                    _ => continue,
                }
            }
        });
    }
    Ok(())
}
