# static2app-action

Package pre-built static web assets into a single Go binary with an embedded web UI.

This action does **not** build your frontend.  
It assumes your static assets are already built (e.g., via `npm run build`).

## Usage

```yaml
- name: Package as standalone app
  uses: hyxf/static2app/action@v1
  with:
    dist-dir: build          # Your pre-built static files
    binary-name: my-app      # Output binary name
    version: 1.0.0           # Optional: override version
```

## Inputs

| Name | Required | Description |
|------|----------|-------------|
| `dist-dir` | ✅ | Directory containing pre-built static assets (must include `index.html`) |
| `binary-name` | ✅ | Base name of the output binary (without extension) |
| `version` | ❌ | Version string to embed (defaults to git tag or "dev") |

## Outputs

| Name | Description |
|------|-------------|
| `dist-dir` | Directory containing packaged artifacts (`static2app-src/dist/pkg`) |

## Full Example

```yaml
name: Build Desktop App

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Build frontend
        run: |
          npm install
          npm run build

      - name: Package with Static2App
        id: package
        uses: hyxf/static2app/action@v1
        with:
          dist-dir: build
          binary-name: myapp

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: myapp-binaries
          path: ${{ steps.package.outputs.dist-dir }}

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        if: startsWith(github.ref, 'refs/tags/')
        with:
          files: ${{ steps.package.outputs.dist-dir }}/*
```

## What Gets Built?

The action produces cross-platform binaries for:

- macOS (Intel & Apple Silicon)
- Linux (amd64 & arm64)
- Windows (amd64)

Each binary is packaged as:
- `.tar.gz` for macOS/Linux
- `.zip` for Windows

Plus a `checksums.txt` file for verification.

## License

MIT
