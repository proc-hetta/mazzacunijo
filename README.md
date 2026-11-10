# mazzacunijo

Telegram bot by `/proc/hetta` that powers on machines on the LAN with Wake-on-LAN.
Only whitelisted Telegram users can use it, in private chats only. Anyone else is told
their Telegram ID, so they can ask to be added.

## Configuration

| Variable                  | Description                                                                 |
| ------------------------- | --------------------------------------------------------------------------- |
| `MAZZACUNIJO_BOT_TOKEN`   | Token from [@BotFather](https://t.me/BotFather)                              |
| `MAZZACUNIJO_ALLOWED_IDS` | Comma-separated Telegram user IDs, e.g. `123456789,987654321`               |
| `MAZZACUNIJO_SSH_HOST`    | Public host used in the SSH command, e.g. `example.org`                     |
| `MAZZACUNIJO_MACHINES`    | JSON list of `{"name", "mac", "ip", "username"}` objects, unique names up to 64 bytes |
| `MAZZACUNIJO_LOG`         | Optional [log filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html), defaults to `info` |

`ip` is the machine's IPv4 address, where the magic packet is sent. After a successful
wake the bot replies with `ssh -p220<last octet of ip> <username>@<MAZZACUNIJO_SSH_HOST>`.

```sh
MAZZACUNIJO_MACHINES='[{"name": "nas", "mac": "aa:bb:cc:dd:ee:ff", "ip": "192.168.1.40", "username": "admin"}]'
```

## Running

```sh
docker compose up -d --build
```

The container needs `network_mode: host`, because broadcast packets do not leave
Docker's bridge network. See [`compose.yaml`](compose.yaml).

## Releasing

Pushing a semver tag (e.g. `1.2.3`) builds a multi-arch image and pushes it to Docker Hub
as `$REGISTRY_USERNAME/mazzacunijo`.

## Banner

`assets/banner.jpg` is the `/proc/hetta` logo on a dark gradient. It is embedded in the
binary and used as the control-panel photo.
