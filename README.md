# hello_ockam

Table of contents

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Differences between this repo & docs.ockam.io/Reference/Programming Libraries/Rust](#differences-between-this-repo--docsockamioreferenceprogramming-librariesrust)
- [Following Rust API guides below](#following-rust-api-guides-below)
- [Run the examples](#run-the-examples)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Differences between this repo & [docs.ockam.io/Reference/Programming Libraries/Rust](https://docs.ockam.io/reference/libraries/rust)

1. All the code examples have colorized output so that it is clear to see which node is outputting
   what to `stdout`.
   - Here's an example of secure channels example: ![05-secure-channel-ex](image.png)
   - Here's another example of routing over multiple hops:
     ![03-routing-over-multiple-hops-ex](image-1.png)
2. Instead of having multiple source files for certain examples (eg:
   [secure channels](https://github.com/nazmulidris/hello_ockam/blob/main/examples/05-secure-channel-over-two-transport-hops-responder.rs)),
   all the examples are self contained in a single source file using `tokio::spawn` to run multiple
   nodes concurrently, and shut down the entire example cleanly.
3. All short variable names are replaced w/ longer more readable ones.

## Following Rust API guides below

1. [Rust API](https://docs.ockam.io/reference/libraries/rust)
2. [Nodes and Workers](https://docs.ockam.io/reference/libraries/rust/routing)
3. [Routing and transports](https://docs.ockam.io/reference/libraries/rust/routing)
4. [Identities and Vaults](https://docs.ockam.io/reference/libraries/rust/vaults-and-identities)
5. [Secure Channels](https://docs.ockam.io/reference/libraries/rust/secure-channels)
6. [Credentials and Authorities](https://docs.ockam.io/reference/libraries/rust/credentials)

## Run the examples

```sh
cargo run --example 01-node
```

```sh
cargo run --example 02-worker
```

```sh
cargo run --example 03-routing
```

```sh
cargo run --example 03-routing-many-hops
```

```sh
cargo run --example 04-routing-over-transport
```

```sh
cargo run --example 04-routing-over-two-transport-hops
```

```sh
cargo run --example 05-identity
```

```sh
cargo run --example 05-secure-channel-over-two-transport-hops-responder
```
