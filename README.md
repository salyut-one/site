# salyut-site

Web frontend for https://salyut.one. It serves the homepage, user sites, user
list, the read-only BBS view at `/bbs`, and system `pinky -lb` output at
`/now`.

## Build and test

```sh
make check
make build
```

For local development alongside `salyut-bbsd`:

```sh
cargo run
curl http://127.0.0.1:8082/users
curl http://127.0.0.1:8082/bbs
curl http://127.0.0.1:8082/now/~"$USER"
```

The listen address, BBS Unix socket, pinky binary, and pinky timeout are
configurable:

```text
salyut-site --listen 127.0.0.1:8082 \
  --bbs-socket /run/salyut-bbs/users/salyut.sock \
  --pinky /usr/bin/pinky --pinky-timeout-seconds 3
```

## Fedora 44

Build and install the release binary and systemd unit, then enable the service:

```sh
make check
make build
sudo make install
sudo useradd --system --gid salyut-bbs --home-dir /nonexistent \
  --shell /usr/sbin/nologin salyut-web
systemctl daemon-reload
systemctl enable --now salyut-site.service
```

The included unit runs as the unprivileged `salyut-web` account in the
`salyut-bbs` group, so the site uses the same daemon socket as the terminal
client. The BBS daemon independently rejects mutations from that identity.
Point the root site's reverse proxy at port 8082.
