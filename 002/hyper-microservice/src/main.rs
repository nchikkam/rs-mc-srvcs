use hyper::{Body, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

fn main() {
    let port = 8000;
    let address = ([127, 0, 0, 1], port).into();
    let builder = Server::bind(&address);

    let server = builder.serve(|| {
        service_fn_ok(|_| {
            Response::new(Body::from("Minimal Server - Microservice"))
        })
    });

    let server = server.map_err(drop);
    println!("Server Listening at port: {}", port);
    hyper::rt::run(server);
}

/*
Notes:
.into() is a Rust conversion method that uses the Into trait.

It tries to automatically convert the given value into another type, 
as long as the target type implements From for the given input type.

So .into() figures out what type you want based on the context (what type 
addr is expected to be)
*/