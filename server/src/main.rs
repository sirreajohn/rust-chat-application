use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const IP_ADDR: &str = "192.168.29.69:6000";
const MSG_SIZE: usize = 999;

fn sleep() {
    thread::sleep(std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(IP_ADDR).expect("Failed to bind TCP listener to given port.");
    server
        .set_nonblocking(true)
        .expect("cannot set server to non blocking mode.");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    println!("listening on {}", IP_ADDR);
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("client connected at, {}", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone socket"));

            thread::spawn(move || {
                loop {
                    let mut buffer = vec![0; MSG_SIZE];

                    match socket.read_exact(&mut buffer) {
                        Ok(_) => {
                            let msg = buffer
                                .into_iter()
                                .take_while(|&x| x != 0)
                                .collect::<Vec<_>>();

                            let msg = String::from_utf8(msg)
                                .expect("failed to convert string into UTF-8");

                            println!("{}, {}", addr, msg);
                            let msg = format!("{}: {}", addr, msg);
                            tx.send(msg).expect("failed to send msg");
                        }
                        Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                        Err(_) => {
                            println!("problem with the connection");
                            break;
                        }
                    }
                    sleep();
                }
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buffer = msg.clone().into_bytes();
                    buffer.resize(MSG_SIZE, 0);

                    client.write_all(&buffer).map(|_| client).ok()
                })
                .collect::<Vec<_>>()
        }

        sleep();
    }
}
