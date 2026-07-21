# salyut-site

Web frontend for https://salyut.one. It serves the homepage, user sites, user
list, the read-only BBS view at `/bbs`, and system `pinky -lb` output at
`/now`. Each BBS board publishes RSS and Atom feeds at
`/bbs/boards/<slug>/rss.xml` and `/bbs/boards/<slug>/atom.xml`.

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

## Deploying

```sh
salyut-admin update
```

## License
[MIT](./LICENSE)
