use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const IP_ADDR: &str = "127.0.0.01:6000";
const MSG_SIZE: usize = 999;

fn main() {
    let mut client = TcpStream::connect(IP_ADDR).expect("trouble connecting to chat server");
    client
        .set_nonblocking(true)
        .expect("cannot set client non blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        loop {
            let mut buffer = vec![0; MSG_SIZE];
            match client.read_exact(&mut buffer) {
                Ok(_) => {
                    let msg = buffer
                        .into_iter()
                        .take_while(|&x| x != 0)
                        .collect::<Vec<_>>();

                    let msg = String::from_utf8(msg).expect("failed to convet receiving message");
                    println!("{}", msg);
                }

                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("connection lose due to an error");
                    break;
                }
            }

            match rx.try_recv() {
                Ok(msg) => {
                    let mut buffer = msg.clone().into_bytes();
                    buffer.resize(MSG_SIZE, 0);
                    client
                        .write_all(&buffer)
                        .expect("failed to write into client");
                    // println!("message sent: {}", msg);
                }

                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    println!("write a message");
    loop {
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .expect("failed to read input strings");
        let msg = buffer.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }

    println!("Bye man.")
}
