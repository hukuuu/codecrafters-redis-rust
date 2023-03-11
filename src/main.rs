use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener};


fn main() -> Result<(), Box<dyn Error>> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        println!("CONNECTION");
        let mut stream = stream.unwrap();
        let mut br = BufReader::new(stream.try_clone().unwrap());
        loop {
            let mut line = String::new();
            br.read_line(&mut line)?;
            dbg!(&line);
            match line.as_str() {
                "ping\r\n" => {
                    stream.write("+PONG\r\n".as_bytes())?;
                }
                _ => continue,
            }
        }
    }
    Ok(())
}
