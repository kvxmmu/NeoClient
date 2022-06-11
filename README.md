# NeoGrok Client

Simple, but useful [NeoGrok](https://git.nerodono.su/nero/neogrok) client written in Rust.


# Compilation

First, you need to install latest version of the Rust language. [rustup](https://rustup.rs/) is very useful for this purpose.

Next, compile NeoGrok client with the following command:

```bash
$ cargo build --release        # Compile NeoGrok client
$ mv target/release/neogrok .  # Extract binary from the target directory
$ rm -rf target                # Remove unnecessary stuff
```

# Simple usage

```bash
$ neogrok --remote remote-neogrok.su tcp localhost:80
```

Server will connect to the remote-neogrok.su and request server to bind available port. Here you go

# Advanced usage

For the advanced usage you can use `--help` flag, all flags are well-documented.

e.g.

```bash

$ neogrok --help      # list of all available root flags
$ neogrok tcp --help  # list of all available tcp flags

```

