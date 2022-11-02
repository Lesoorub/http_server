# http_server
 This is the simplest http server framework.

# Opportunities
 With this library, you can process input requests of any type (GET, POST, PUT, and e.tc .) and return any data in response to them: files, strings and byte arrays.

# Minuses
 Dynamic path processing has not been implemented yet, this imposes many restrictions.

# Compilation
Clone repository: `git clone https://github.com/Lesoorub/http_server`
Go to the repository: `cd http_server`<br>
Build library: `cargo build --release`

# Install
Since this is a library, you need to add a link to it in your `Cargo.toml` file:
Add next line after `[dependencies]`: <br>
`http_server = { path = "http_server" }`
In the path, you need to specify a relative or absolute path to the cloned library repository.

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
    http_server::str("<html><body>Test</body></html>")
}
fn root(_: &mut Request) -> Response
{
    http_server::redirect("/home")
}

fn _404(_: &mut Request) -> Response
{
    http_server::bytes(vec![ b'4', b'0', b'4' ])
}
```
``````
