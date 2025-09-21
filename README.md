# Safe multi-threaded DNS lookup with First-Connect HTTP Client in Rust

## General
Simple command-line program that first performs a DNS lookup provided a hostanem, then tries to concurrently establish a HTTP GET request with each IP address obtained. The first established TCP connection is chosen to perform the GET request, with others closed. Rust's safe multi-threading techniques were used.

<img width="300" height="500" alt="image" src="https://github.com/user-attachments/assets/a5c34d0f-324a-45d7-a591-bcace7f50456" />


### Extra
This repo contains a short essay on [*Memory Management in Rust, compared to C*](https://github.com/RaphaelNekam/Advanced_Systems_AE/blob/9ae997c6c0cc37ba3c8a336361d6a0e05454ff66/Short_essay_MM_Rust.pdf)

## To run

1. Run ```cargo init``` if required (Cargo.toml present, so should not be required)
2. Run ```cargo run -- [hostname]``` to perform try and establish HTTP get requests to the provided hostname. In practice this could e.g. be ```cargo run -- www.google.com```
