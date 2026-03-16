# emuNEX (server)
Remote Emulation & Library Management.

emuNEX is a service, think of it like Steam, but for emulators. This server will get client connections to download emulators and games. In the future it could handle features such as profiles/friends.

Find the source code for the client here: https://github.com/chloecinders/emuNEX-client

## (Legal) Disclaimer

emuNEX server itself provides no emulators or games. Games and emulators must be uploaded to the server with the permission of the respective copyright owners. emuNEX does not support any form of piracy.

## Depdencies

emuNEX server requires a Postgres database and any S3 storage (Personally I recommend Seaweedfs since its relatively easy to set up).

If you want to build emuNEX server yourself you will only need to install Rust.

## Installation

If you are using a Linux server you can just grab a binary together with the templates from the latest GitHub actions run: https://github.com/chloecinders/emuNEX-client/actions.
If you are using a Windows server instead (I hope not) you can build the server yourself.

## Configuration

See [Config.default.toml](Config.default.toml) for all config values.
To config the server place a Config.toml file in the same location as the binary.

```toml
server_domain = "http://localhost:8000" # Reachable domain of your server. Used to pass it onto the emuNEX client.
database_url = "postgres://user:password@host:port/database" # PostgresSQL Database url
repository = "chloecinders/emunex-server" # GitHub repository to pull updates from. Will always pull the most recent artifact binary.
github_token = "" # Used to pull the update from the above repository.
machine_id = 0 # Used for id generation

# S3 configuration, self explanatory.
[s3]
endpoint = "http://localhost:8333"
bucket = "emunex"
access_key = ""
secret_key = ""
region = ""
```

## Building From Source

Make sure Git and Rust is installed, preferrably through [rustup](https://rust-lang.org/tools/install/).
Once Rust is installed run the following commands:
```bash
git clone https://github.com/chloecinders/emuNEX-server.git
cd emuNEX-server
cargo build --release
```
The binary will be built to `./target/{toolchain}/release/emunex-server(.exe)`.

## Contributing

Contributions are open for everyone. Feel free to just make a PR. However we do reject "vibecoding". The majority of the code must be made by yourself and any AI generated code must be vetted to ensure code quality.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
