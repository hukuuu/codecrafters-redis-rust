mod lib;

use bytes::Bytes;
use lib::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Serving");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

async fn process(stream: TcpStream) {
    let mut connection = Connection::new(stream);
    loop {
        if let Some(frame) = connection.read_frame().await.unwrap() {
            process_frame(&mut connection, frame).await;
        }
    }
}

async fn process_frame(connection: &mut Connection, frame: Frame) {
    match frame {
        Frame::Simple(s) => {
            println!("simple, {}", s);
        }
        Frame::Error(e) => {
            println!("error {}", e);
        }
        Frame::Integer(n) => {
            println!("number {}", n);
        }
        Frame::Bulk(bytes) => {
            println!("bulk, {:?}", bytes);
            // if bytes.eq(&Bytes::from_static(b"ping")) {
            //     connection.write_frame(Frame::Bulk(bytes)).await.unwrap();
            // }
        }
        Frame::Array(frames) => {
            println!("ARRAY: {:?}", frames);

            match frames.len() {
                1 => {
                    connection
                        .write_frame(Frame::Bulk(Bytes::from_static(b"+PONG\r\n")))
                        .await
                        .unwrap();
                }
                2 => {
                    let f = frames.get(0).unwrap();
                    match f {
                        Frame::Bulk(bytes) => if bytes.eq(&Bytes::from_static(b"echo")) {
                            // connection.write_frame(frames.get(1).unwrap().clone()).await.unwrap()
                            connection.write_frame(Frame::Bulk(Bytes::from_static(b"+world\r\n"))).await.unwrap();
                            if let Frame::Bulk(bytes) = frames.get(1).unwrap() {
                                let mut v: Vec<u8> = vec![43];
                                v.append(&mut bytes.to_vec());
                                connection.write_frame(Frame::Bulk(Bytes::from(v))).await.unwrap();
                            }
                        },
                        _ => unimplemented!(),
                    }
                }
                _ => unimplemented!(),
            }
        } // Frame::Null => {
          //     println!("null");
          // }
    }
}
