use nix::{fcntl::OFlag, pty};
use std::{
    os::{
        fd::{AsRawFd, FromRawFd, IntoRawFd},
        unix::fs::symlink,
    },
    path::PathBuf,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[tokio::main]
async fn main() {
    let symlink_path = PathBuf::from("/tmp/pty-test");
    let pty_master = pty::posix_openpt(OFlag::O_RDWR | OFlag::O_NOCTTY).unwrap();
    pty::grantpt(&pty_master).unwrap();
    pty::unlockpt(&pty_master).unwrap();

    let slave_name = pty::ptsname_r(&pty_master).unwrap();

    let mut host_file = unsafe { File::from_raw_fd(pty_master.into_raw_fd()) };

    let _ = std::fs::remove_file(&symlink_path);
    symlink(&slave_name, symlink_path).unwrap();

    let (mut reader, mut writer) = tokio::io::split(host_file);

    // let (tx, rx) = async_channel::unbounded();
    // tokio::join!(
    //     async {
    //         loop {
    //             let mut data = [0; 1];
    //             reader.read_exact(&mut data).await.unwrap();
    //             println!("Got data: {:02x}", data[0]);
    //             if data[0] == b'q' {
    //                 break;
    //             }
    //             tx.send(data[0]).await.unwrap();
    //         }
    //     },
    //     async {
    //         while let Ok(data) = rx.recv().await {
    //             println!("Received data");
    //             let data = [data, b'.'];
    //             if let Err(err) = writer.write(&data[..]).await {
    //                 eprintln!("Err on write: {err}");
    //             }
    //         }
    //     }
    // );

    loop {
        let mut data = [0; 1];
        reader.read_exact(&mut data).await.unwrap();
        println!("Got data: {:02x}", data[0]);
        if data[0] == b'q' {
            break;
        }
        let data = [data[0], b'.'];
        writer.write(&data[..]).await.unwrap();
    }
}
