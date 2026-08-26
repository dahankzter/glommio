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

#[test]
fn a_file_larger_than_the_pipe_arrives_intact() {
    // 65536 is the pipe capacity, so 200 KiB forces the loop to go round
    // several times. A single-chunk implementation truncates here.
    LocalExecutor::default().run(async {
        let payload: Vec<u8> = (0..200 * 1024).map(|i| (i % 251) as u8).collect();
        let path = seed("large", &payload);

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

        assert_eq!(sent, expected_len);
        let got = reader.await.unwrap();
        assert_eq!(got.len(), payload.len(), "every byte must arrive");
        assert_eq!(got, payload, "and in the right order");

        std::fs::remove_file(&path).ok();
    });
}

#[test]
fn asking_past_the_end_returns_short_rather_than_erroring() {
    LocalExecutor::default().run(async {
        let payload = b"only sixteen by".to_vec();
        let path = seed("short", &payload);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let reader = glommio::spawn_local(async move {
            let mut stream = listener.accept().await.unwrap();
            let mut got = Vec::new();
            stream.read_to_end(&mut got).await.unwrap();
            got
        })
        .detach();

        let mut writer = TcpStream::connect(addr).await.unwrap();
        let file = BufferedFile::open(&path).await.unwrap();
        // Ask for far more than the file holds.
        let sent = writer.send_file(&file, 0, 1024 * 1024).await.unwrap();
        file.close().await.unwrap();
        drop(writer);

        assert_eq!(sent, payload.len(), "a short file is short, not an error");
        assert_eq!(reader.await.unwrap(), payload);

        std::fs::remove_file(&path).ok();
    });
}

// This test is discriminated by mutating the initial `let mut pos = offset;`
// in `send_file` (change it to start at 0): that sends payload[0..2048]
// instead of payload[1024..3072], same length, wrong window. It is NOT
// discriminated by mutating the `pos += filled as u64;` advance inside the
// loop -- at 2048 bytes this transfer fits in a single splice, so the loop
// only runs once and that line's effect on later iterations never fires.
// The large-file test is what proves `pos` accumulates correctly.
#[test]
fn sending_from_an_offset_skips_the_prefix() {
    LocalExecutor::default().run(async {
        let payload: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
        let path = seed("offset", &payload);

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let reader = glommio::spawn_local(async move {
            let mut stream = listener.accept().await.unwrap();
            let mut got = Vec::new();
            stream.read_to_end(&mut got).await.unwrap();
            got
        })
        .detach();

        let mut writer = TcpStream::connect(addr).await.unwrap();
        let file = BufferedFile::open(&path).await.unwrap();
        let sent = writer.send_file(&file, 1024, 2048).await.unwrap();
        file.close().await.unwrap();
        drop(writer);

        assert_eq!(sent, 2048);
        assert_eq!(
            reader.await.unwrap(),
            payload[1024..3072].to_vec(),
            "the offset must select the right window, not just the right length"
        );

        std::fs::remove_file(&path).ok();
    });
}
