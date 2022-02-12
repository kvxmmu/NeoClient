# NeoGrok client

Stupidly simple [NeoGrok](https://github.com/kvxmmu/neogrok) client

# Usage

`$ ./neogrok --help`

```
neogrok 1.2.0
nero
Simple NeoGrok(https://github.com/kvxmmu/neogrok) client implementation

USAGE:
    neogrok [OPTIONS] --remote <REMOTE> --local <LOCAL>

OPTIONS:
        --compression-level <COMPRESSION_LEVEL>
            Overwritten compression level (0 - for synchronization with server) [default: 0]

        --compression-profit <COMPRESSION_PROFIT>
            Minimal compression profit in percents(difference between original and compressed size
            in percent representation) [default: 5]

    -h, --help
            Print help information

    -l, --local <LOCAL>
            Local server address in domain[:port] format

    -m, --magic <MAGIC>
            Local server address

    -p, --port <PORT>
            Port to be requested [default: 0]

    -r, --remote <REMOTE>
            

    -t, --timeout <TIMEOUT>
            Local server connect timeout [default: 5]

    -V, --version
            Print version information
```

