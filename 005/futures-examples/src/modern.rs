use tokio::net::UdpSocket;
use tokio::sync::{mpsc, oneshot};
use tokio::task;
use tokio::time::{sleep, Duration};
use std::io;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    single().await?;
    multiple().await?;
    send_spawn().await?;
    println!("Starting UDP echo server...");
    alt_udp_echo().await?;
    Ok(())
}

// 1️⃣ One-shot channel example
async fn single() -> Result<()> {
    let (tx, rx) = oneshot::channel::<u8>();

    // Spawn a sender
    task::spawn(async move {
        tx.send(8).unwrap();
    });

    // Await receiver
    let value = rx.await?;
    println!("Received: {}", value);
    Ok(())
}

// 2️⃣ Multiple senders using mpsc
async fn multiple() -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<u8>(8);

    // Spawn senders
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx.clone();

    task::spawn(async move { tx1.send(1).await.unwrap(); });
    task::spawn(async move { tx2.send(2).await.unwrap(); });
    task::spawn(async move { tx3.send(3).await.unwrap(); });

    drop(tx); // Close channel to end receiver when done

    let mut sum = 0;
    while let Some(v) = rx.recv().await {
        sum += v;
    }

    println!("Calculated: {}", sum);
    Ok(())
}

// 3️⃣ Spawning multiple senders
async fn send_spawn() -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<u8>(8);

    // Spawn multiple senders
    for i in 1u8..=10 {
        let tx_clone = tx.clone();
        task::spawn(async move {
            tx_clone.send(i).await.unwrap();
            sleep(Duration::from_millis(50)).await;
        });
    }

    drop(tx); // important: close channel

    let mut total = 0;
    while let Some(v) = rx.recv().await {
        println!("Received: {}", v);
        total += v;
    }

    println!("Total: {}", total);
    Ok(())
}

// 4️⃣ UDP Echo server
async fn alt_udp_echo() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:12345").await?;
    println!("UDP Echo server listening on 0.0.0.0:12345");

    let mut buf = vec![0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let msg = String::from_utf8_lossy(&buf[..len]);
        println!("Received '{}' from {}", msg.trim(), addr);

        socket.send_to(msg.as_bytes(), addr).await?;
    }
}

/*
Needs:
tokio = { version = "1.43", features = ["full"] }
anyhow = "1.0"
*/