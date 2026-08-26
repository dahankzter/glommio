//! `TcpStream::send_file` driven the way a consumer drives it.

use futures_lite::io::AsyncReadExt;
use glommio::{
    io::BufferedFile,
    net::{TcpListener, TcpStream},
    LocalExecutor,
};

fn tmp_path(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("glommio-send-file-{}-{}", std::process::id(), name));
    p
}

/// Writes `contents` to a fresh temp file and returns its path.
fn seed(name: &str, contents: &[u8]) -> std::path::PathBuf {
    let path = tmp_path(name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn a_small_file_arrives_intact() {
    LocalExecutor::default().run(async {
        let payload = b"the quick brown fox jumps over the lazy dog".repeat(10);
        let path = seed("small", &payload);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let expected_len = payload.len();

        let reader = glommio::spawn_local(async move {
            let mut stream = listener.accept().await.unwrap();
            let mut got = Vec::new();
            stream.read_to_end(&mut got).await.unwrap();
            got
        })
        .detach();

        let mut writer = TcpStream::connect(addr).await.unwrap();
        let file = BufferedFile::open(&path).await.unwrap();
        let sent = writer.send_file(&file, 0, expected_len).await.unwrap();
        file.close().await.unwrap();
        drop(writer);

        assert_eq!(
            sent, expected_len,
            "send_file should report every byte sent"
        );
        let got = reader.await.unwrap();
        assert_eq!(got, payload, "the bytes on the wire must match the file");

        std::fs::remove_file(&path).ok();
    });
}
