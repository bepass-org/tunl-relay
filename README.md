# tunl-relay

## Quick Start
```sh
$ ./tunl-relay --config config.toml
```

## Config
```toml
version = "v1"
bind = "0.0.0.0"
port = 6666

whitelist = [
    "173.245.48.0/20",
    "103.21.244.0/22",
    "103.22.200.0/22",
    "103.31.4.0/22",
    ...
]

# it blocks private networks by default
# but you can add other ip addresses (such as torrent trackers) to the list
blacklist = [
    "93.158.213.92/32",
]
```

**protocol version**: v1 refers to [bepass-relay protocol](https://github.com/bepass-org/bepass-relay/)
