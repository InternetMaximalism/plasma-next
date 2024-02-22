# Halo2 Proof Generation Server

## How to Run

```sh
cargo run -r
```

## Environment Variables

- `SERVER_HOST`: Specifies the IP address or hostname where the server will bind [Default: "127.0.0.1"]
- `SERVER_PORT`: Determines the port number on which the server will listen for incoming connections [Default: "8081"]
- `SRS_PATH`: Specifies the path of the SRS file [Default: "srs.dat"]
- `LOG_LEVEL`: Specifies the log level for the server [Default: "info"]

### Setting the Variables

#### Linux/macOS

Use `export SERVER_PORT=<port>`.

#### Windows

Use `set SERVER_PORT=<port>` in the command prompt.

## Features

- `debug`: Do not generate actual proofs, instead generate dummy proofs.

```sh
cargo run -r --features debug
```
