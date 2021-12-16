// Shamefule theft from Lily Mara/Code Tech as a baseline - https://www.youtube.com/watch?v=Iapc-qGTEBQ

use tokio::{
    io::{AsyncBufReadExt, BufReader, AsyncWriteExt},
    net::TcpListener,
    sync::broadcast,
};

use zaps_8583::Field;

#[tokio::main]
async fn main() {
    let listen_addr = "localhost:9090";
    let listener = TcpListener::bind(listen_addr)
        .await
        .expect(&format!("Unable to listen on {}", listen_addr));
    
    println!("Listener established for {}", listen_addr);

    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
    
        println!("Accepted connection from {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

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
                        let send = match line.clone().trim_end_matches("\n").parse::<Field>() {
                            Err(e) => {
                                eprintln!("{:?}", e);
                                line.clone()
                            },
                            Ok(field) => field.to_string(),
                        };
                        tx.send((send, addr)).unwrap();
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
