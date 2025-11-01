use rand::Rng;
// main.rs
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};
use futures::future::FutureExt; // for .then()

#[tokio::main]
async fn main() {
    println!("--- Demo 1: Async Task ---");
    demo_async_task().await;

    println!("\n--- Demo 2: Chained Futures (Promise-like) ---");
    demo_chained_futures().await;

    println!("\n--- Demo 3: Reactor Pattern Simulation (Non-blocking TCP) ---");
    // Uncomment this line if you want to run the TCP listener demo
    // demo_reactor_tcp().await;

    println!("\n--- Demo 3: Simulated Reactor ---");
    demo_simulated_reactor().await;
}

/// Demo 1: Simple async task returning a future (like a promise)
async fn demo_async_task() {
    async fn async_task() -> i32 {
        sleep(Duration::from_secs(1)).await;
        42
    }

    let result = async_task().await;
    println!("Async task result: {}", result);
}

/// Demo 2: Chaining futures like Promise.then()
async fn demo_chained_futures() {
    async fn async_task() -> i32 {
        sleep(Duration::from_secs(1)).await;
        10
    }

    async fn double(x: i32) -> i32 {
        x * 2
    }

    // Use FutureExt to chain
    let fut = async_task().then(|val| double(val));
    let result = fut.await;
    println!("Chained future result: {}", result);
}

/// Demo 3: Reactor pattern simulation with TCP listener
/// Note: This will block waiting for connections, so usually run separately
async fn demo_reactor_tcp() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on 127.0.0.1:8080...");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection from {:?}", addr);

        // Spawn a new async task for each connection
        tokio::spawn(async move {
            println!("Handling connection from {:?}", socket.peer_addr());
            // Here you could read/write with the socket asynchronously
        });
    }
}


/// Demo 3: Simulated Reactor using async tasks and timers
async fn demo_simulated_reactor() {
    println!("Simulating events in a reactor...");

    // Simulate 5 "events"
    let mut handles = vec![];

    for i in 1..=5 {
        let handle = tokio::spawn(async move {
            let delay = rand::thread_rng().gen_range(500..2000);
            sleep(Duration::from_millis(delay)).await;
            println!("Event {} handled after {} ms", i, delay);
        });
        handles.push(handle);
    }

    // Wait for all simulated events to finish
    for handle in handles {
        handle.await.unwrap();
    }

    println!("All simulated events handled by reactor.");
}

/*
Notes:
Explanation

Demo 1 (demo_async_task)
Simple async function returning a value after sleep.
Equivalent to a Promise that resolves later.

Demo 2 (demo_chained_futures)
Chains async computations like JS .then().
Uses FutureExt::then() from the futures crate.
Demo 3 (demo_reactor_tcp)
Demonstrates the Reactor pattern: a non-blocking TCP server using Tokio.
Each connection is handled by a separate task, reactor handles waking tasks.
*/