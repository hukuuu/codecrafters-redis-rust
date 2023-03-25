use redis_starter_rust::Frame;
use tokio::{
    io::copy,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

async fn process(mut stream: TcpStream) {
    loop {
        let (mut r, mut w) = stream.split();
        copy(&mut r, &mut w).await.unwrap();
    }
}
