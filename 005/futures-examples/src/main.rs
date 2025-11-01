extern crate failure;
extern crate futures;
extern crate tokio;

use std::io;
use failure::Error;
use futures::{future, stream, Future, Stream, Sink, IntoFuture};
use futures::sync::{mpsc, oneshot};
use tokio::net::{UdpSocket, UdpFramed};
use tokio::codec::LinesCodec;

fn to_box<T>(fut :T) -> Box<dyn Future<Item=(), Error=()> + Send>
where
    T: IntoFuture,
    T::Future: Send + 'static,
    T::Item: 'static,
    T::Error: 'static,
{
    let fut = fut.into_future().map(drop).map_err(drop);
    Box::new(fut)
}

fn other<E>(err: E) -> io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::Other, err)
}

fn main() {
    single();
    multiple();
    send_spawn();
    println!("Start UDP echo");
    alt_udp_echo().unwrap();
}

fn single() {
    let (tx_sender, rx_future) = oneshot::channel::<u8>();
    let receiver = rx_future.map(|x| {
        println!("Received: {}", x);
    });
    let sender = tx_sender.send(8);
    let execute_all = future::join_all(vec![
        to_box(receiver),
        to_box(sender),
    ]).map(drop);
    tokio::run(execute_all);
}

fn multiple() {
    let (tx_sink, rx_stream) = mpsc::channel::<u8>(8);
    let receiver = rx_stream.fold(0, |acc, value| {
        future::ok(acc + value)
    }).map(|x| {
        println!("Calculated: {}", x);
    });
    let send_1 = tx_sink.clone().send(1);
    let send_2 = tx_sink.clone().send(2);
    let send_3 = tx_sink.clone().send(3);
    let execute_all = future::join_all(vec![
        to_box(receiver),
        to_box(send_1),
        to_box(send_2),
        to_box(send_3),
    ]).map(drop);
    drop(tx_sink);
    tokio::run(execute_all);
}

fn alt_udp_echo() -> Result<(), Error> {
    let from = "0.0.0.0:12345".parse()?;
    let socket = UdpSocket::bind(&from)?;
    let framed = UdpFramed::new(socket, LinesCodec::new());
    let (sink, stream) = framed.split();
    let (tx, rx) = mpsc::channel(16);
    let rx = rx.map_err(|_| other("can't take a message"))
        .fold(sink, |sink, frame| {
            sink.send(frame)
        });
    let process = stream.and_then(move |args| {
        tx.clone()
            .send(args)
            .map(drop)
            .map_err(other)
    }).collect();
    let execute_all = future::join_all(vec![
        to_box(rx),
        to_box(process),
    ]).map(drop);
    Ok(tokio::run(execute_all))
}

fn send_spawn() {
    let (tx_sink, rx_stream) = mpsc::channel::<u8>(8);
    let receiver = rx_stream.fold(0, |acc, value| {
        println!("Received: {}", value);
        future::ok(acc + value)
    }).map(drop);
    let spawner = stream::iter_ok::<_, ()>(1u8..11u8).map(move |x| {
        let fut = tx_sink.clone().send(x).map(drop).map_err(drop);
        tokio::spawn(fut);
    }).collect();
    let execute_all = future::join_all(vec![
        to_box(spawner),
        to_box(receiver),
    ]).map(drop);
    tokio::run(execute_all);
}

/*
Notes:
demonstration of asynchronous programming using Tokio 0.1 and Futures 0.1, 
showing how to use channels, streams, futures, and UDP sockets.
It’s quite an old-style Tokio example (from the pre–async/await era), but 
it still illustrates the key ideas of how async tasks communicate and 
execute concurrently

Crate setup and imports
extern crate failure;
extern crate futures;
extern crate tokio;


These are early (pre-2018 edition) crate imports. The program uses:
failure → for error handling,
futures → for composing asynchronous computations,
tokio → the runtime for executing them.

Functions:
fn to_box<T>(fut :T) -> Box<dyn Future<Item=(), Error=()> + Send>
Takes any future (T: IntoFuture) and converts it into a boxed, 
sendable future of type Future<Item=(), Error=()>.

Basically normalizes all futures so they can be run together in join_all.

fn other<E>(err: E) -> io::Error
Converts any error type into a generic io::Error with kind Other.

Used in the UDP echo example to make error types compatible.

fn main() {
    single();
    multiple();
    send_spawn();
    println!("Start UDP echo");
    alt_udp_echo().unwrap();
}
The program runs several examples in sequence:
single() — demonstrates a one-shot channel.
multiple() — demonstrates an mpsc channel.
send_spawn() — demonstrates spawning tasks that send messages.
alt_udp_echo() — runs a UDP echo server.



1. single() – one-shot communication
---------------------------------------------------------------
let (tx_sender, rx_future) = oneshot::channel::<u8>();

A one-time send/receive pair.
let receiver = rx_future.map(|x| println!("Received: {}", x));
let sender = tx_sender.send(8);
sender sends the number 8.
receiver prints it when received.
Then:
tokio::run(execute_all);
Runs both concurrently in the Tokio runtime.
O/P: received: 8


2. multiple() – mpsc (multi-producer, single-consumer) channel
---------------------------------------------------------------
let (tx_sink, rx_stream) = mpsc::channel::<u8>(8);


Creates a channel that can buffer up to 8 messages.

let receiver = rx_stream.fold(0, |acc, value| future::ok(acc + value))
    .map(|x| println!("Calculated: {}", x));


Receives all values, adds them up, and prints the total.

let send_1 = tx_sink.clone().send(1);
let send_2 = tx_sink.clone().send(2);
let send_3 = tx_sink.clone().send(3);


Sends 1, 2, and 3 into the channel.
O/P
Calculated: 6


3. send_spawn() – spawning send tasks
---------------------------------------------------------------
let (tx_sink, rx_stream) = mpsc::channel::<u8>(8);

Similar setup, but here multiple tasks are spawned to send numbers 1–10.
let spawner = stream::iter_ok::<_, ()>(1u8..11u8).map(move |x| {
    let fut = tx_sink.clone().send(x).map(drop).map_err(drop);
    tokio::spawn(fut);
}).collect();

Iterates over numbers 1–10.
For each, spawns a Tokio task that sends it into the channel.
let receiver = rx_stream.fold(0, |acc, value| {
    println!("Received: {}", value);
    future::ok(acc + value)
}).map(drop);

Receives and prints each number.

O/P
Received: 1
Received: 2
...
Received: 10




4. alt_udp_echo() – UDP echo server
---------------------------------------------------------------
let from = "0.0.0.0:12345".parse()?;
let socket = UdpSocket::bind(&from)?;

Binds a UDP socket on port 12345.
let framed = UdpFramed::new(socket, LinesCodec::new());
let (sink, stream) = framed.split();

Wraps the UDP socket in a line-based codec (treats each line as a frame).
Splits it into a sender and receiver half.
let (tx, rx) = mpsc::channel(16);
An internal message-passing channel.
let rx = rx.map_err(|_| other("can't take a message"))
    .fold(sink, |sink, frame| sink.send(frame));

Takes messages from rx and sends them back out over the UDP socket (the “echo”).
let process = stream.and_then(move |args| {
    tx.clone().send(args).map(drop).map_err(other)
}).collect();

Reads incoming UDP messages and forwards them into the channel.

In effect:
Any message you send to UDP port 12345 will be echoed back to you.

+ ---------------- + --------------------- + ------------------------------------------ +
| Function         | Demonstrates          | Description                                |
+ ---------------- + --------------------- + ------------------------------------------ +
| `single()`       | `oneshot` channel     | Sends a single message and prints it       |
| `multiple()`     | `mpsc` channel        | Sends multiple messages and sums them      |
| `send_spawn()`   | spawning tasks + mpsc | Spawns async senders sending numbers 1–10  |
| `alt_udp_echo()` | UDP networking        | A simple UDP echo server using `UdpFramed` |
+ ---------------- + --------------------- + ------------------------------------------ +
Notes

This code uses Tokio 0.1 and futures 0.1, which are obsolete.
Modern Rust uses async/await and tokio::spawn(async move { ... }) instead.

The patterns here (channels, task spawning, and UDP framing) still map conceptually 
to modern async code, just with much cleaner syntax today.
*/