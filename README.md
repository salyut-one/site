# salyut-site

The website for [salyut.one](https://salyut.one). It serves the home page at
`/` and builds `/users` from the normal local accounts in `/etc/passwd`, with a
link from each username to that user's `/~username` personal page. Users are
ordered by the birth time of their home directories, oldest first; accounts
whose home-directory birth time is unavailable appear last.

The HTTP server listens on `127.0.0.1:8082` by default and is intended to sit
behind the TLS reverse proxy for `salyut.one`. The reverse proxy should continue
to serve `/~username` from each user's `public_html`; this application owns the
site home page and user list, not the contents of users' home directories.

## Build and test

```sh
cargo test --locked
cargo build --release --locked
```

For local development with a fixture account database:

```sh
cargo run -- --passwd ./test-passwd
curl http://127.0.0.1:8082/users
```

The listen address, account database, normal-user UID range, and worker count
are configurable:

```text
salyut-site --listen 127.0.0.1:8082 --passwd /etc/passwd \
  --uid-min 1000 --uid-max 60000 --workers 4
```

## Fedora 44

Install the release binary and systemd unit, then enable the service:

```sh
install -m 0755 target/release/salyut-site /usr/local/bin/salyut-site
install -m 0644 etc/systemd/system/salyut-site.service \
  /etc/systemd/system/salyut-site.service
systemctl daemon-reload
systemctl enable --now salyut-site.service
```

The included unit reuses the unprivileged `salyut-web` account from
`salyut-bbs` and gives it read-only access to home-directory metadata. Point
the root site's reverse-proxy location at port 8082 while leaving its existing
`/~username` user-directory location in place.
