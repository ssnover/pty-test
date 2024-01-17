use nix::{
    pty::{self, OpenptyResult},
    sys::termios::{tcgetattr, Termios},
};
use std::{
    os::{
        fd::{AsRawFd, FromRawFd, IntoRawFd},
        unix::fs::symlink,
    },
    path::PathBuf,
    time::Duration,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[tokio::main]
async fn main() {
    let symlink_path = PathBuf::from("/tmp/pty-test");
    let OpenptyResult { master, slave } = pty::openpty(None, None).unwrap();

    let mut host_file = File::from(std::fs::File::from(master));
    let device_path = nix::unistd::ttyname(slave.as_raw_fd()).unwrap();

    let _ = std::fs::remove_file(&symlink_path);
    symlink(&device_path, symlink_path).unwrap();

    let (mut reader, mut writer) = tokio::io::split(host_file);

    let (tx, rx) = async_channel::unbounded();
    tokio::join!(
        async {
            loop {
                let mut data = [0; 1];
                reader.read_exact(&mut data).await.unwrap();
                println!("Got data: {:02x}", data[0]);
                if data[0] == b'q' {
                    break;
                }
                tx.send(data[0]).await.unwrap();
            }
        },
        async {
            while let Ok(data) = rx.recv().await {
                println!("Received data");
                let data = [data, b'.'];
                if let Err(err) = writer.write(&data[..]).await {
                    eprintln!("Err on write: {err}");
                }
            }
        }
    );

    println!("Hello, world!");
}
