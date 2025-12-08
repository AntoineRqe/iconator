# Icon web server

## Architecture

I've implemented a simple web server that can serve icons from a local folder.

The web server has a single endpoint `/icon/{type}/{name}` where `type` can be `file` or `folder`, and `name` is the name of the icon to serve.

```
+---------+   HTTP   +------------------+   route   +-------------------+
| Client  | -------> | Axum Router      | ------->  | get_icon handler  |
+---------+          | GET /icon/{...}  |           +---------+---------+
                                                        | validate name
                                                        | kind=file/folder
                                                        v
                                                +-----------------------+
                                                | FST lookup in lib.rs  |
                                                | -> icon number        |
                                                +-----------+-----------+
                                                            |
                                                            v
                                                +-----------------------+
                                                | read icons/{n}.svg    |
                                                | return image/svg+xml  |
                                                +-----------+-----------+
                                                            |
                                                            v
                                                       +---------+
                                                       | Client  |
                                                       +---------+
```

Note: I've decided to use a single endpoint for the same logic is used for both file and folder icons. The only difference is the `type` parameter in the URL.

## How to use

```bash
cargo build --release
cargo run --release
```

## Test

Send a request to the server to get an icon:

```bash
curl http://127.0.0.1:7878/icon/file/tslint.xml
curl http://127.0.0.1:7878/icon/folder/images
```

or use a browser to access the URL:

```
http://127.0.0.1:7878/icon/folder/images
```

## Limitation

### Localhost

For now, the server is only accessible from localhost and an hardcoded port (7878).

### Authentication

Currently, there is no authentication implemented. The server is open to anyone who can access it. On localhost, this is not a problem. But once it is deployed on internet, I need to implement some authentication mechanism (using jwt for example).

## Deployment

Firstly, we should package the server and its dependencies into a docker image.

Secondly, we can deploy the docker image on a any remote machine (VPS, cloud provider, etc...). We can use `docker-compose` to manage the deployment and configuration of the server (ip, port).

Depending on the deployment target, we can use a reverse proxy (nginx...) to handle the TLS termination and route the requests to the server.

## Scalability

For now, the lookup is in-memory, this works is we have a single server running. If we reach the limits of a single server, we can deploy multiple instances of the server behind a load balancer.

If the icons is immutable (which is unlikely to happen), each instance can have its own copy of svg files and mapping file.

But if icons are prone to evolve, we will need to store them in a distributed database, which can be access by each server instance (read only).

## Monitoring

We can use `prometheus` to monitor the server metrics (request count, request duration, error count, etc...). The `metrics` crate can be used to expose the metrics in a prometheus compatible format.

We will need measure the request duration (p50, p95, p99) to detect performance issues. We can also measure the request count and error count to detect errors.


## Stack

### Async Rust

I decided to use `tokio` as the async runtime for this project. It is currently the most popular async runtime, with an active community.
An other alternative could be `async-std`.

### Web server

I decided to use `axum` as the web server framework for this project. It is maintained by the same team as `tokio`, so optimzed to run on top of it. Other option could be `artix-web`, which could be more performant at the cost of complexity and rampup.

If we want to have full control of the web server, we can use `tokio` directly, start a listener, and spawn a task for each connection. But this is more complex and error-prone as we need to handle the HTTP protocol ourselves.

### Logging

I use `tracing` to log the server activity. It is a modern logging framework for Rust, with support for structured logging and async logging -> almost no overhead.

## Future implementation

### Argument passing

Use `clap` to parse argument from terminal and manually configure the server (port, host, etc...).

### Cache

we can use a local cache to store the last N requested icons -> better performance and less network congestion if icons stored in a remote server for example.

### Graceful shutdown

We can implement a graceful shutdown mechanism to handle the termination of the server. Tokio provides a `CancellationToken`.
