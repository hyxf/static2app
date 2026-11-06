# Static2App GitHub Action

此 Action 将**已经构建好的静态网站**嵌入 Rust 可执行文件。它不会安装前端依赖，也不会执行 `npm run build`；请先在调用 workflow 中完成前端构建。

## 快速接入

```yaml
- uses: actions/checkout@v4
- uses: actions/setup-node@v4
  with:
    node-version: 20

- name: Build frontend
  run: npm ci && npm run build

- name: Package website
  id: package
  uses: hyxf/static2app/action@v1
  with:
    dist-dir: dist
    binary-name: my-app
    # version: 1.2.3 # 可选

- uses: actions/upload-artifact@v4
  with:
    name: my-app-package
    path: ${{ steps.package.outputs.dist-dir }}/*
```

将 `dist` 改为前端实际构建目录。该目录根部必须包含 `index.html`，并包含页面所需的脚本、样式、图片、字体等全部静态文件。

## 输入

| 名称 | 必填 | 默认值 | 说明 |
|---|---:|---|---|
| `dist-dir` | 是 | — | 静态网站产物目录，根部必须存在 `index.html` |
| `binary-name` | 是 | — | 输出程序和压缩包名称的基础部分，不含扩展名 |
| `version` | 否 | Git ref 名称；否则 `dev` | 写入程序的 `--version` 输出，并用于压缩包名称 |

未显式设置 `version` 时，Action 使用 `github.ref_name`，并去掉开头的 `v`。例如 tag `v1.2.3` 会生成版本 `1.2.3`。本地直接运行 `cargo build` 则使用 `Cargo.toml` 中的 package 版本。

## 输出

`dist-dir` 输出值是调用 workflow workspace 下的 `static2app-src/dist/pkg`，其中包含一个平台压缩包和 `checksums.txt`。

| Runner 系统 | 包格式 | 示例名称 |
|---|---|---|
| Linux | `.tar.gz` | `my-app-1.2.3-x86_64-unknown-linux-gnu.tar.gz` |
| macOS | `.tar.gz` | `my-app-1.2.3-aarch64-apple-darwin.tar.gz` |
| Windows | `.zip` | `my-app-1.2.3-x86_64-pc-windows-msvc.zip` |

target triple 和架构取决于 runner。**每次 Action 运行只构建当前 runner 的一个平台**。多平台发布请创建多个原生 runner job；macOS 目标需要 Apple SDK，应在 macOS runner 上构建。

## 发布到 GitHub Release

单次 Action 调用会产生一个平台包。多平台构建、收集 checksums 并创建 Release 的完整 tag workflow，请复制 [examples/react-example.yml](../examples/react-example.yml)。它在推送 `v*` tag 后运行，需要 release job 具有 `contents: write` 权限。

仓库根目录的 `.github/workflows/release.yml` 是 Static2App 项目自身的 Action 发布维护流程，用于构建检查并更新 `v1` 主版本标签；它不替使用者的网站创建应用 Release。

若只需暂存单个平台产物，可用 [actions/upload-artifact](https://github.com/actions/upload-artifact)：

```yaml
- name: Package website
  id: package
  uses: hyxf/static2app/action@v1
  with:
    dist-dir: dist
    binary-name: my-app

- uses: actions/upload-artifact@v4
  with:
    name: my-app-package
    path: ${{ steps.package.outputs.dist-dir }}/*
```

## 运行行为与注意事项

- 可执行文件嵌入整份静态网站，启动本地 HTTP 服务并尝试打开默认浏览器。
- 默认随机端口；可传入 `--port 8080` 指定端口。`--help` 查看参数，`--version` 查看构建版本。
- 程序监听所有网络接口，并打印局域网访问地址。
- 服务没有认证或 HTTPS。能连接到端口的局域网设备均可访问页面；请按需设置防火墙，不要暴露敏感站点。
- Linux 二进制使用 runner 原生 GNU target，旧发行版可能需要较新的 glibc。
- Action 不会为 macOS 程序签名/公证，也不会为 Windows 程序代码签名。公开分发时，操作系统可能显示安全提示；macOS 面向公众发布需配置 Apple Developer ID 签名和公证。

更多前端路径、tag 发布、平台限制和排错说明见仓库根目录 [README](../README.md)。
