# GetIt! :truck:

GetIt is an easy-to-use resource getter. It is designed to be extremely minimal to the point that a simple example is all you need to learn it completely. To understand what it can do, below are a few examples of it's use.

First, add it to your `Cargo.toml`:

```toml
[dependencies]
    getit = "0.1"
```

## Examples

Get a file from your computer:

```rust
let response = getit::get("file:///home/user/file.txt").unwrap();
println!("{}", response);
```

or, even simpler because the default scheme is file:

```rust
let response = getit::get("home/user/file.txt").unwrap();
println!("{}", response);
```

Get a web page via HTTP:

```rust
let response = getit::get("http://www.example.com").unwrap();
println!("{}", response);
```

Get a web page via HTTPS:

```rust
let response = getit::get("https://www.example.com").unwrap();
println!("{}", response);
```

Get remote file via FTP:

```rust
let response = getit::get("ftp://user:pass@host/file.txt").unwrap();
println!("{}", response);
```

- While GetIt will attempt to establish an FTPS connection, it will fall back to bare FTP if the server does not support FTPS.
- Requires `ftp` feature - enabled by default

## Why GetIt?

GetIt is designed to be a simple and lightweight resource getter that can handle a variety of protocols, including file, HTTP, HTTPS, and FTP. It is intended to be easy to use and understand, with a minimal API that provides just the essential functionality.

It started as a personal project to load a configuration file either locally or from a remote server, but the author recognized that it could be useful to others as well. The goal is to provide a tool that is easy to integrate into other projects, without the overhead or mental load of a full-featured library.

## Non-Goals

GetIt is designed to be as simple as possible. It is not intended to be a full-featured HTTP client library. If you need a full-featured HTTP client library, check out [ureq](https://github.com/algesten/ureq) or [reqwest](https://github.com/seanmonstar/reqwest). Similarly, if you need a full-featured FTP client, go find that! This is not that. This is just a simple resource getter. Nothing more.
