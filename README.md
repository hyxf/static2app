# Static2App

> 🚀 GitHub Action：将前端静态网站打包为 **跨平台、零依赖的独立可执行文件**

**Static2App** 是一个 GitHub Action，用于在 CI 中将 **已构建的静态网站**
（React / Vue / Vite / 任意 HTML、CSS、JS）
打包为一个 **即开即用的本地桌面应用（二进制文件）**。

无需 Electron / Node / Python / JVM。

---

## ✨ 特性

* 🚀 **零依赖**：单一可执行文件，无需安装 Node.js、Python 或其他运行时
* 🌍 **跨平台**：
  * macOS（Intel 和 Apple Silicon）
  * Linux（amd64 / arm64）
  * Windows（amd64）
* 📦 **自动打包**：
  * macOS / Linux → `.tar.gz`
  * Windows → `.zip`
  * 自动生成 SHA256 校验和
* ⚡ **即开即用**：启动自动打开默认浏览器
* 🔒 **完全本地运行**：无网络、无数据上传
* 🎯 **CI 友好**：专为 GitHub Actions 设计
* 🔢 **版本支持**：内置版本信息和 `--version` 标志

---

## 🧠 工作原理

1. 前端项目先生成静态产物（如 `build/`、`dist/`）
2. Action 将静态文件嵌入 Go 二进制（`embed`）
3. 程序启动本地 HTTP 服务（随机端口 127.0.0.1）
4. 自动打开默认浏览器访问
5. 所有资源均来自本地内存

---

## 🚀 快速开始

### 示例：打包 React 应用

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

      - name: Create GitHub Release (optional)
        uses: softprops/action-gh-release@v1
        if: startsWith(github.ref, 'refs/tags/')
        with:
          files: ${{ steps.package.outputs.dist-dir }}/*
```

### 示例：打包 Vue / Vite 应用

```yaml
- name: Build frontend
  run: |
    npm install
    npm run build

- name: Package with Static2App
  uses: hyxf/static2app/action@v1
  with:
    dist-dir: dist        # Vite 默认输出目录
    binary-name: my-vue-app
```

---

## 🔧 Action Inputs

| 名称          | 必填 | 默认值 | 说明                                      |
| ------------- | ---- | ------ | ----------------------------------------- |
| `dist-dir`    | ✅    | -      | 前端构建产物目录（必须包含 `index.html`） |
| `binary-name` | ✅    | -      | 输出的可执行文件名称（不含扩展名）        |
| `version`     | ❌    | auto   | 手动指定版本号（优先于 Git tag）          |

### 📌 版本号优先级

```text
1. inputs.version（手动指定）
   ↓
2. Git tag（如 v1.2.3，自动去掉 v 前缀）
   ↓
3. "dev"（默认开发版本）
```

**示例：**
- Git tag `v1.2.3` → 版本号 `1.2.3`
- 手动设置 `version: 2.0.0-beta` → 版本号 `2.0.0-beta`

---

## 📦 构建产物

Action 会自动生成并打包以下文件：

```text
static2app-src/dist/pkg/
├── myapp-1.2.3-darwin-amd64.tar.gz
├── myapp-1.2.3-darwin-arm64.tar.gz
├── myapp-1.2.3-linux-amd64.tar.gz
├── myapp-1.2.3-linux-arm64.tar.gz
├── myapp-1.2.3-windows-amd64.zip
└── checksums.txt                        # SHA256 校验和
```

> 文件名可直接作为 GitHub Release Assets 使用。

---

## 🖥 支持的平台

| 平台    | 架构                      | 文件格式  |
| ------- | ------------------------- | --------- |
| macOS   | amd64 (Intel)             | .tar.gz   |
| macOS   | arm64 (Apple Silicon)     | .tar.gz   |
| Linux   | amd64                     | .tar.gz   |
| Linux   | arm64                     | .tar.gz   |
| Windows | amd64                     | .zip      |

---

## 🎯 使用场景

* ✅ 前端项目 → 桌面应用交付
* ✅ 离线文档 / 演示 Demo
* ✅ 内部系统分发（无需部署服务器）
* ✅ 给非技术用户使用的 Web 工具
* ✅ 本地开发工具（如配置编辑器、数据可视化）
* ✅ 教育培训材料（离线可用）

---

## 🔐 安全说明

* ✅ 仅监听 `127.0.0.1`（本地回环地址）
* ✅ 不访问公网
* ✅ 不收集任何用户数据
* ✅ 所有内容仅在本地运行
* ✅ 随机端口避免冲突

---

## 🛠 本地开发

克隆此仓库后，你可以在本地测试：

```bash
# 1. 准备你的静态文件
mkdir -p static
echo '<h1>Hello World</h1>' > static/index.html

# 2. 构建
go build -o myapp

# 3. 运行
./myapp

# 4. 查看版本
./myapp --version
```

---

## 📋 常见问题

### Q: 支持哪些前端框架？
A: 任何能生成静态 HTML/CSS/JS 的框架都支持：React, Vue, Angular, Svelte, Solid, Vite, Next.js (static export), Nuxt (static), 纯 HTML 等。

### Q: 生成的二进制文件有多大？
A: 取决于你的静态资源大小。基础二进制约 5-10 MB，加上你的资源文件。

### Q: 可以自定义端口吗？
A: 目前使用随机端口以避免冲突。如有需求可提 Issue。

### Q: 支持 API 调用吗？
A: 支持！你的前端代码可以调用外部 API，但建议将 API 端点做成可配置的。

### Q: 如何分发给用户？
A: 用户只需下载对应平台的压缩包，解压后直接运行可执行文件即可。

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

## 📜 License

MIT License - 详见 [LICENSE](LICENSE)

---

## 🔗 相关链接

- [GitHub Actions 文档](https://docs.github.com/actions)
- [Go embed 文档](https://pkg.go.dev/embed)
- [示例项目](./examples) (即将推出)

---

**Made with ❤️ by hyxf**
