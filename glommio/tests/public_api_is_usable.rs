//! Every public seam added recently, exercised the way a dependent crate
//! would.
//!
//! Twice in two releases something was published that could not be used from
//! outside: `RxBuf` was `pub` in a private module and never re-exported, and
//! `OwnedRxBuf` was re-exported with a `pub(crate)` constructor and no
//! accessor. Neither was visible from inside the crate, where everything is
//! reachable and every constructor is in scope.
//!
//! An integration test compiles as a separate crate, so it sees exactly what
//! a consumer sees. This file is deliberately shallow -- it checks that each
//! seam can be *reached and driven*, not that it is correct, which is what
//! the unit tests are for.

use futures_lite::{
    io::{AsyncReadExt, AsyncWriteExt},
    stream::StreamExt,
};
use glommio::{
    channels::{broadcast, oneshot, watch},
    net::{TcpListener, TcpStream},
    LocalExecutor, LocalExecutorBuilder, Placement,
};
use std::io::IoSlice;

#[test]
fn shared_channels_can_be_built_and_moved_between_executors() {
    // The Storage seam: `shared` constructors must be callable, and what they
    // return must be `Send` from outside as well as inside.
    fn assert_send<T: Send>(_: &T) {}

    let (bcast_tx, mut bcast_rx) = broadcast::shared::<u32>(16);
    let (watch_tx, mut watch_rx) = watch::shared(0u32);
    let (once_tx, once_rx) = oneshot::shared::<u32>();
    assert_send(&bcast_tx);
    assert_send(&bcast_rx);
    assert_send(&watch_tx);
    assert_send(&watch_rx);
    assert_send(&once_tx);
    assert_send(&once_rx);

    let far = LocalExecutorBuilder::new(Placement::Unbound)
        .spawn(move || async move {
            let broadcast = bcast_rx.recv().await.unwrap();
            watch_rx.changed().await.unwrap();
            let watched = watch_rx.get();
            let once = once_rx.await.unwrap();
            (broadcast, watched, once)
        })
        .unwrap();

    bcast_tx.send(1).unwrap();
    watch_tx.send(2).unwrap();
    once_tx.send(3).unwrap();

    assert_eq!(far.join().unwrap(), (1, 2, 3));
}

#[test]
fn local_channels_still_work_through_their_public_surface() {
    LocalExecutor::default().run(async {
        let (sender, mut receiver) = broadcast::broadcast::<u32>(4);
        sender.send(7).unwrap();
        assert_eq!(receiver.next().await.unwrap().unwrap(), 7);

        let (sender, mut receiver) = watch::watch(0u32);
        sender.send(9).unwrap();
        receiver.changed().await.unwrap();
        assert_eq!(*receiver.borrow(), 9);

        let (sender, receiver) = oneshot::oneshot::<u32>();
        sender.send(11).unwrap();
        assert_eq!(receiver.await.unwrap(), 11);
    });
}

#[test]
fn a_vectored_write_is_reachable_from_outside() {
    LocalExecutor::default().run(async {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let reader = glommio::spawn_local(async move {
            let mut stream = listener.accept().await.unwrap();
            let mut got = Vec::new();
            let mut buf = [0u8; 32];
            loop {
                let read = stream.read(&mut buf).await.unwrap();
                if read == 0 {
                    break;
                }
                got.extend_from_slice(&buf[..read]);
            }
            got
        })
        .detach();

        let mut writer = TcpStream::connect(addr).await.unwrap();
        let written = writer
            .write_vectored(&[IoSlice::new(b"one"), IoSlice::new(b"two")])
            .await
            .unwrap();
        assert_eq!(written, 6, "the default writes only the first slice");
        writer.close().await.unwrap();

        assert_eq!(reader.await.unwrap(), b"onetwo".to_vec());
    });
}

#[test]
fn a_buffered_stream_is_reachable_from_outside() {
    LocalExecutor::default().run(async {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let reader = glommio::spawn_local(async move {
            let mut stream = listener.accept().await.unwrap().buffered();
            let mut buf = [0u8; 16];
            let read = stream.read(&mut buf).await.unwrap();
            buf[..read].to_vec()
        })
        .detach();

        let mut writer = TcpStream::connect(addr).await.unwrap();
        writer.write_all(b"buffered").await.unwrap();
        assert_eq!(reader.await.unwrap(), b"buffered".to_vec());
    });
}

#[test]
fn every_address_shape_still_compiles() {
    // The resolver replaced std's `ToSocketAddrs` with glommio's own, so the
    // shapes callers actually pass have to keep working from outside the
    // crate. This test is about compiling, not connecting.
    use std::net::{IpAddr, Ipv4Addr};

    LocalExecutor::default().run(async {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let port = addr.port();
        let owned = format!("127.0.0.1:{port}");

        // Each of these is a shape std::net::ToSocketAddrs accepts.
        assert!(TcpStream::connect(addr).await.is_ok());
        assert!(TcpStream::connect(owned.as_str()).await.is_ok());
        assert!(TcpStream::connect(owned.clone()).await.is_ok());
        assert!(TcpStream::connect(("127.0.0.1", port)).await.is_ok());
        assert!(TcpStream::connect((String::from("127.0.0.1"), port))
            .await
            .is_ok());
        assert!(TcpStream::connect((IpAddr::V4(Ipv4Addr::LOCALHOST), port))
            .await
            .is_ok());
        assert!(TcpStream::connect(&[addr][..]).await.is_ok());
        assert!(TcpStream::connect(vec![addr]).await.is_ok());
        // A borrow of a non-'static String: the shape that rules out simply
        // bounding the parameter Send + 'static and handing it to the pool.
        assert!(TcpStream::connect(&owned).await.is_ok());
    });
}
