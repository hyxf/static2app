package main

import (
	"context"
	"embed"
	"flag"
	"fmt"
	"io"
	"io/fs"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/pkg/browser"
)

//go:embed static/*
var distFS embed.FS

// Version is set via ldflags during build
var Version = "dev"

func main() {
	// 添加版本标志
	showVersion := flag.Bool("version", false, "Show version information")
	flag.Parse()

	if *showVersion {
		fmt.Printf("static2app version %s\n", Version)
		os.Exit(0)
	}

	// 获取 static 子目录
	distRoot, err := fs.Sub(distFS, "static")
	if err != nil {
		log.Fatal(err)
	}

	// 随机端口
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		log.Fatal(err)
	}

	port := ln.Addr().(*net.TCPAddr).Port
	addr := fmt.Sprintf("http://127.0.0.1:%d", port)

	// HTTP 服务器配置
	server := &http.Server{
		Handler:      spaHandler(distRoot),
		ReadTimeout:  10 * time.Second,
		WriteTimeout: 10 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	// 优雅关闭
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-quit
		log.Println("Shutting down server...")
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := server.Shutdown(ctx); err != nil {
			log.Printf("Server shutdown error: %v", err)
		}
	}()

	// 打开浏览器
	go func() {
		time.Sleep(100 * time.Millisecond)
		if err := browser.OpenURL(addr); err != nil {
			log.Printf("Failed to open browser: %v", err)
		}
	}()

	log.Printf("Server running at %s (version: %s)", addr, Version)
	if err := server.Serve(ln); err != nil && err != http.ErrServerClosed {
		log.Fatal(err)
	}
}

// spaHandler 处理 SPA 路由
func spaHandler(fsys fs.FS) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// 只允许 GET 和 HEAD
		if r.Method != http.MethodGet && r.Method != http.MethodHead {
			http.Error(w, "Method Not Allowed", http.StatusMethodNotAllowed)
			return
		}

		// 禁用缓存
		w.Header().Set("Cache-Control", "no-cache, no-store, must-revalidate")
		w.Header().Set("Pragma", "no-cache")
		w.Header().Set("Expires", "0")

		path := strings.TrimPrefix(r.URL.Path, "/")
		if path == "" {
			path = "index.html"
		}

		// 尝试打开文件
		file, err := fsys.Open(path)
		if err != nil {
			// 文件不存在，fallback 到 index.html（SPA 路由）
			serveIndexHTML(w, r, fsys)
			return
		}
		defer file.Close()

		// 获取文件信息
		stat, err := file.Stat()
		if err != nil {
			http.Error(w, "Internal Server Error", http.StatusInternalServerError)
			return
		}

		// 如果是目录，fallback 到 index.html
		if stat.IsDir() {
			serveIndexHTML(w, r, fsys)
			return
		}

		// 设置 Content-Type
		w.Header().Set("Content-Type", getContentType(path))

		// 发送文件内容
		http.ServeContent(w, r, stat.Name(), stat.ModTime(), file.(io.ReadSeeker))
	})
}

// serveIndexHTML 提供 index.html
func serveIndexHTML(w http.ResponseWriter, r *http.Request, fsys fs.FS) {
	file, err := fsys.Open("index.html")
	if err != nil {
		http.Error(w, "Not Found", http.StatusNotFound)
		return
	}
	defer file.Close()

	stat, err := file.Stat()
	if err != nil {
		http.Error(w, "Internal Server Error", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	http.ServeContent(w, r, stat.Name(), stat.ModTime(), file.(io.ReadSeeker))
}

// getContentType 根据文件扩展名返回 Content-Type
func getContentType(path string) string {
	switch {
	case strings.HasSuffix(path, ".css"):
		return "text/css; charset=utf-8"
	case strings.HasSuffix(path, ".js"):
		return "application/javascript; charset=utf-8"
	case strings.HasSuffix(path, ".json"):
		return "application/json; charset=utf-8"
	case strings.HasSuffix(path, ".png"):
		return "image/png"
	case strings.HasSuffix(path, ".jpg"), strings.HasSuffix(path, ".jpeg"):
		return "image/jpeg"
	case strings.HasSuffix(path, ".svg"):
		return "image/svg+xml"
	case strings.HasSuffix(path, ".woff"):
		return "font/woff"
	case strings.HasSuffix(path, ".woff2"):
		return "font/woff2"
	case strings.HasSuffix(path, ".ico"):
		return "image/x-icon"
	default:
		return "text/html; charset=utf-8"
	}
}
