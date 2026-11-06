# Static2App

Static2App 将**已经构建好的静态网站**打包成独立可执行文件。程序把网页资源嵌入二进制，启动本地 Rust HTTP 服务，并尝试打开默认浏览器。适用于 Vite、React、Vue、Svelte、Angular 静态导出或纯 HTML 项目。

运行器使用 Rust、`tiny_http` 和 `rust-embed`。Static2App 不负责构建前端，也不包含后端/API 服务。

## 1. 准备静态网站

先用前端项目现有命令构建。构建产物目录的根部必须有 `index.html`，并包含它引用的全部文件：

```text
dist/
├── index.html
├── assets/
│   ├── app.js
│   └── app.css
└── images/
    └── logo.png
```

`dist-dir` 要指向整个构建产物目录，不能只传 `index.html` 或 `assets/`。Action 也会复制隐藏文件。HTML 中的资源路径应能从站点根路径或相对路径访问；如果前端配置了部署子路径，打包前请调整 base/public path。

浏览器会对资源 URL 中的空格和特殊字符进行百分号编码，运行器会在查找嵌入文件前解码路径。服务识别常用网页、图片、字体、PDF、音频和视频 MIME 类型。

已有的 HTML 和资源会按 URL 路径提供。无扩展名的 SPA 路由会回退到 `index.html`；看起来像文件名的未知路径返回 404。

## 2. 配置 GitHub Release 工作流

完整的 React/Vite 示例位于 [examples/react-example.yml](examples/react-example.yml)。将它复制到你项目的 `.github/workflows/release.yml`，再按项目实际情况调整 Node 版本、前端构建命令、产物目录和可执行文件名。

注意：本仓库自己的 [.github/workflows/release.yml](.github/workflows/release.yml) 用于构建检查并更新 Static2App Action 的 `v1` 主版本标签；它不是使用者项目的应用发布工作流。要发布你自己的静态网站，请使用上面的 `examples/react-example.yml`。

示例工作流会：

1. 构建一次前端静态网站。
2. 将静态产物交给 Linux、macOS、Windows runner。
3. 每个 runner 为自身系统和架构构建一个可执行文件包。
4. 当推送 `v*` tag 时，把各平台压缩包和校验和发布到同一个 GitHub Release。

发布 Release 需要 `contents: write` 权限。如果仓库限制了 Actions 权限，请在仓库设置中允许工作流写入内容。

### 发布步骤

提交工作流后，推送一个版本 tag：

```sh
git add .github/workflows/release.yml
git commit -m "Add packaged app release workflow"
git push origin main
git tag v1.2.3
git push origin v1.2.3
```

推送 tag 会触发工作流。所有平台构建完成后，在仓库 **Releases** 页面下载附件。未设置 `version` 输入时，Action 使用 tag 名称并去掉开头的 `v`：`v1.2.3` 会生成版本 `1.2.3`，并写入程序的 `--version` 输出。

## Action 输入与输出

| 输入/输出 | 必填 | 默认值 | 说明 |
|---|---:|---|---|
| `dist-dir` | 是 | — | 静态网站构建目录，根部必须有 `index.html` |
| `binary-name` | 是 | — | 可执行文件及压缩包的基础名称，不含扩展名 |
| `version` | 否 | Git ref 名称；否则 `dev` | 写入 `--version`，也用于压缩包名称 |
| 输出 `dist-dir` | — | — | 包含压缩包和 `checksums.txt` 的目录 |

如果你的 workflow 已经构建了前端，可以直接添加打包步骤。下面的方式会把压缩包存为 workflow artifact；要发布正式 Release，请使用上面的完整工作流。

```yaml
- name: Build frontend
  run: npm ci && npm run build

- name: Package static site
  id: package
  uses: hyxf/static2app/action@v1
  with:
    dist-dir: dist
    binary-name: my-app
    # version: 1.2.3 # 可选；默认从 tag 推导

- uses: actions/upload-artifact@v4
  with:
    name: my-app-package
    path: ${{ steps.package.outputs.dist-dir }}/*
```

## Release 内容

每个 runner 生成一个压缩包和 SHA256 校验和。文件名包含应用名、版本和 Rust target triple，例如：

```text
my-app-1.2.3-x86_64-unknown-linux-gnu.tar.gz
my-app-1.2.3-aarch64-apple-darwin.tar.gz
my-app-1.2.3-x86_64-pc-windows-msvc.zip
checksums-linux.txt
checksums-macos.txt
checksums-windows.txt
```

实际架构由 runner 镜像决定。示例工作流生成一个 Linux、一个 macOS 和一个 Windows 包；如需更多架构，可在 [示例矩阵](examples/react-example.yml) 中增加 runner。macOS 交叉编译需要 Apple SDK，因此 macOS 包应在 macOS runner 上构建。

## 下载并运行

下载与用户操作系统和架构匹配的压缩包并解压：

```sh
# macOS 或 Linux
tar -xzf my-app-1.2.3-<target>.tar.gz
./my-app-1.2.3-<target> --help
./my-app-1.2.3-<target>

# Windows：在资源管理器中解压 .zip，再运行其中的 .exe
my-app-1.2.3-<target>.exe --help
```

默认由系统分配随机端口。传入 `--port 8080` 或 `--port=8080` 使用固定端口；`--port 0` 明确要求随机端口。`--help` 列出参数，`--version` 输出嵌入的版本号。

程序启动后会显示本机 URL；检测到活动网卡地址时还会显示局域网 URL。服务监听所有网络接口，**没有认证或 HTTPS**：任何能连接该端口的设备都可以访问嵌入的网站。防火墙和网络隔离规则仍然生效，请勿在不可信网络中提供敏感内容。

嵌入的网页本身可以发起网络请求。Static2App 不代理 API 请求，也不提供后端。

## 本地构建与版本号

本地开发时，将静态文件放入仓库的 `static/` 目录，再运行：

```sh
cargo build --release
./target/release/static2app --version
./target/release/static2app --port 8080
```

本地构建使用 [Cargo.toml](Cargo.toml) 中的 package 版本（本仓库当前为 `0.1.0`）。GitHub Action 使用 `version` 输入或推送的 tag。Release 配置偏向缩小可执行文件；前端资源大小会加到二进制体积中。

## 平台说明

- Action 每个 runner 构建一个本机 target，不会在一个 job 中自动交叉编译所有平台。
- Linux 二进制使用 runner 的 GNU target；较旧发行版可能缺少所需的 glibc 版本。
- Action 不会为 macOS 二进制签名或公证。Gatekeeper 可能警告或阻止从 Release 下载的程序。面向公众发布 macOS 程序时，请在 workflow 中加入 Apple Developer ID 签名与公证。
- Action 不会为 Windows 可执行文件代码签名。

## 常见问题

| 现象 | 检查项 |
|---|---|
| Action 提示找不到 `index.html` | `dist-dir` 应指向前端构建产物根目录，不是其父目录或 `assets/` 子目录。 |
| 页面打开但样式/图片缺失 | 检查构建后的 HTML 资源路径和前端 base/public path。 |
| SPA 客户端路由返回 404 | 使用无扩展名路由；看起来像文件名的路径会按资源处理。 |
| 局域网设备无法连接 | 使用启动日志中的 LAN URL，开放主机防火墙端口，并确认设备间网络可达。 |
| Release 缺少系统或架构 | 在打包矩阵中增加对应原生 runner，并确认其架构符合目标。 |
| macOS 阻止下载的程序 | 工作流没有签名/公证；面向最终用户分发时需加入 Developer ID 签名和公证。 |

## 许可证

MIT，详见 [LICENSE](LICENSE)。
