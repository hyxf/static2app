# Static2App GitHub Action

Packages an already-built static website into a standalone Rust executable using `tiny_http` and `rust-embed`. This Action does not build the frontend.

```yaml
- uses: hyxf/static2app/action@v1
  id: package
  with:
    dist-dir: dist
    binary-name: my-app
    version: 1.0.0 # optional
```

Inputs are `dist-dir` (required; must contain `index.html`), `binary-name` (required), and `version` (optional; defaults to the Git ref name or `dev`). Output `dist-dir` points to the compressed package and `checksums.txt`.

The Action builds one native artifact for the runner OS and architecture. Use separate native runner jobs for each target platform. Build macOS binaries on macOS runners because cross-compilation requires an Apple SDK.
