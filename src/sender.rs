use std::net::UdpSocket;

fn main() {
    print!("Starting sender . . . ");
    let socket = UdpSocket::bind("127.0.0.1:35555").unwrap();
    print!("Done.\n");

    let mut buf = [0; 100];
    println!("Sending data.");
    socket.send_to(&mut buf, "127.0.0.1:34555");

    drop(socket);
}
