# http_server
 This is the simplest http server framework.

# Compilation
`cd http_server`<br>
`cargo build --release`

# Install
In Cargo.toml add next line after `[dependencies]`: <br>
`http_server = { path = "http_server" }`

# Usage 
Example: <br>
``````markdown
```rust
use http_server::{Server, Request, Response};
fn main() {
    let ip = String::from("127.0.0.1");
    let port = 80;
    let mut server = Server::listen(&ip, &80);
    
    println!("Start server on {}:{}", ip, port);

    server.route("/home", home);
    server.route("/test", test);
    server.route("/", root);
    server.route404(_404);
    server.start();
}

fn home(_: &mut Request) -> Response
{
    http_server::file("index.html")
}
fn test(_: &mut Request) -> Response
{
    http_server::file("test.html")
}
fn root(_: &mut Request) -> Response
{
    http_server::redirect("/home")
}

fn _404(_: &mut Request) -> Response
{
    http_server::file("404.html")
}
```
``````
