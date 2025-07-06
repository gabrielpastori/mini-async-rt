mod executor;
mod task;
mod reactor;
mod udp_socket;

use executor::{new_executor_spawner};
use udp_socket::UdpSocket;

fn main() {
    let (executor, spawner) = new_executor_spawner();
    spawner.spawn(async_main());
    std::mem::drop(spawner);

    executor.run();
}

async fn async_main() {
    let socket = UdpSocket::bind("127.0.0.1:8000").unwrap();

    let mut buf = [0; 10];
    let (amt, src) = socket.recv_from(&mut buf).await.unwrap();

    let buf = &mut buf[..amt];
    buf.reverse();
    socket.send_to(buf, src).await.unwrap();
}