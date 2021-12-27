// Shameful theft from Lily Mara/Code Tech as a baseline - https://www.youtube.com/watch?v=Iapc-qGTEBQ
use std::sync::Arc;

use tokio::{
    io::{AsyncBufReadExt, BufReader, AsyncWriteExt},
    net::TcpListener,
    sync::broadcast,
};

use zaps::{
    iso8583_spec_build,
    iso8583::spec::Spec,
    core::Parser,
};

pub fn iso8583_spec() -> Spec {
    iso8583_spec_build!(
        "0200":
            0: AsciiBitmap, 8;
            1: LLLVar, Alpha;
            8: Fixed, 15, Alphanum;
    )
}

pub async fn serve<K, T>(engine: T)
where
    T: 'static + Parser<K> + Send + Sync,
    <T as Parser<K>>::Err: std::fmt::Debug,
    K: std::fmt::Debug,
{
    let engine = Arc::new(engine);
    let listen_addr = "localhost:9090";
    let listener = TcpListener::bind(listen_addr)
        .await
        .unwrap_or_else(|e| panic!("Unable to listen on {}: {}", listen_addr, e));
    
    println!("Listener established for {}", listen_addr);

    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
    
        println!("Accepted connection from {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let thread_engine = engine.clone();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        line = line.trim_end_matches('\n').into();
                        if line.starts_with("iso8583:") {
                            line = line.trim_start_matches("iso8583:").into();
                            line = match thread_engine.tokenise(line[..].as_bytes()) {
                                Ok(tokens) => format!("{:?}", tokens),
                                Err(e) => format!("{:?}", e),
                            };
                        }
                        tx.send((line.clone() + "\n", addr)).unwrap();
                    }
                    result = rx.recv() => {
                        let (msg, recv_addr)  = result.unwrap();
                        if recv_addr != addr {
                            writer.write(recv_addr.to_string().as_bytes()).await.unwrap();
                            writer.write(" => ".as_bytes()).await.unwrap();
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
    // let res = tokio::spawn(async move {
    //     for _ in 0..10 {
    //         println!("hello world");
    //         std::thread::sleep(std::time::Duration::from_millis(1000));
    //     }
    // }).await;

    // println!("{:?}", res);
}
