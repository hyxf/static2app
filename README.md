# Static2App

Package a pre-built static website into a small standalone Rust executable. The executable embeds the site, serves it on a random `127.0.0.1` port, and opens the system browser.

The runtime uses `tiny_http` and `rust-embed`. It does not require Node.js, Python, Electron, or a separate web server on the user's machine. The embedded page can still make network requests of its own.

## GitHub Action

Build the frontend first, then package its output:

```yaml
- uses: actions/checkout@v4
- name: Build frontend
  run: npm ci && npm run build
- name: Package
  id: package
  uses: hyxf/static2app/action@v1
  with:
    dist-dir: dist
    binary-name: my-app
    # version: 1.2.3
- uses: actions/upload-artifact@v4
  with:
    name: my-app
    path: ${{ steps.package.outputs.dist-dir }}
```

The Action builds for the operating system and architecture of its runner and emits one compressed package plus `checksums.txt`. Run separate jobs on the desired native runners to produce multiple platforms. Rust macOS targets require the macOS SDK, so macOS binaries should be built on macOS runners.

Inputs:

| Name | Required | Default | Description |
|---|---:|---|---|
| `dist-dir` | Yes | — | Built static site directory containing `index.html` |
| `binary-name` | Yes | — | Output executable base name |
| `version` | No | Git ref name, then `dev` | Version embedded in `--version` output and artifact name |

The output `dist-dir` points to `static2app-src/dist/pkg`.

## Runtime behavior

- Binds to a random localhost port and opens the default browser.
- Serves GET and HEAD requests from embedded files.
- Falls back to `index.html` for extensionless SPA routes; missing asset paths return 404.
- Provides `--version`.

## Build locally

Place your built website in `static/`, then run:

```sh
cargo build --release
./target/release/static2app --version
```

Release settings favor smaller binaries (`opt-level = "z"`, LTO, one codegen unit, and stripped symbols). The website assets are part of the executable, so their size contributes directly to the final artifact.

## License

MIT. See [LICENSE](LICENSE).
