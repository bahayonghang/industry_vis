# Industry Vis Justfile
# https://github.com/casey/just
# Package manager: bun (https://bun.sh)

# 默认命令：显示帮助
default:
    @just help

# 显示帮助信息
help:
    @echo ""
    @echo "Industry Vis - 工业数据可视化系统"
    @echo "=================================="
    @echo ""
    @echo "开发命令:"
    @echo "  just dev        启动开发模式 (Tauri + Vite)"
    @echo "  just dev-web    仅启动前端开发服务器"
    @echo ""
    @echo "构建命令:"
    @echo "  just build        快速构建便携版 exe (日常开发)"
    @echo "  just release      构建轻量安装包 (不含 WebView2，约 40MB)"
    @echo "  just release-full 构建完整安装包 (含 WebView2，约 190MB)"
    @echo "  just build-web    仅构建前端"
    @echo ""
    @echo "测试与 CI 命令:"
    @echo "  just ci         完整 CI 检查 (与 GitHub Actions 一致)"
    @echo "  just test       运行所有测试"
    @echo "  just test-rust  仅运行 Rust 测试"
    @echo "  just check      检查代码 (lint + cargo check)"
    @echo "  just lint       代码风格检查"
    @echo ""
    @echo "文档命令:"
    @echo "  just docs       启动文档开发服务器"
    @echo "  just docs-build 构建文档"
    @echo ""
    @echo "其他命令:"
    @echo "  just install      安装所有依赖"
    @echo "  just sync-version 同步版本号 (package.json → Cargo.toml + tauri.conf.json)"
    @echo "  just clean        清理构建产物"
    @echo "  just kill-dev     终止残留的开发服务器进程"
    @echo "  just fmt          格式化 Rust 代码"
    @echo "  just update       更新所有依赖"
    @echo ""

# 安装依赖
install:
    bun install
    cd docs && bun install

# 开发模式
dev:
    bun run tauri:dev

# 仅前端开发
dev-web:
    bun run dev

# 同步版本号（从 package.json 同步到 Cargo.toml 和 tauri.conf.json）
sync-version:
    bun run sync-version

# 快速构建便携版（日常开发测试用，只生成 exe）
build: sync-version
    bun run build
    cd src-tauri && cargo build --release
    @echo ""
    @echo "构建完成: src-tauri/target/release/industry-vis.exe"

# 构建安装包（正式发布用，生成 setup.exe，不含 WebView2 Runtime）
release: sync-version
    bun run tauri:build
    @echo ""
    @echo "安装包已生成: src-tauri/target/release/bundle/nsis/"
    @echo "注意: 此版本不包含 WebView2 Runtime"
    @echo "若启动报错，请用户手动安装: https://developer.microsoft.com/microsoft-edge/webview2/"

# 构建完整安装包（包含 WebView2 Runtime，体积约 150MB+）
release-full: sync-version
    bun run build
    cd src-tauri && cargo tauri build --config '{"bundle":{"windows":{"webviewInstallMode":{"type":"offlineInstaller","silent":true}}}}'
    @echo ""
    @echo "完整安装包已生成: src-tauri/target/release/bundle/nsis/"
    @echo "此版本包含 WebView2 Runtime，可离线安装"

# 仅构建前端
build-web:
    bun run build

# =====================
# CI / 测试 / 检查命令
# =====================

# 完整 CI 检查（与 GitHub Actions release.yml 一致）
ci: ci-setup ci-lint ci-test ci-build
    @echo ""
    @echo "=========================================="
    @echo "✅ CI 检查全部通过！"
    @echo "=========================================="

# CI: 安装依赖
ci-setup:
    @echo ""
    @echo "📦 [1/4] 安装依赖..."
    @echo "=========================================="
    bun install

# CI: 代码风格检查
ci-lint: lint
    @echo ""
    @echo "✅ 代码风格检查通过"

# CI: 运行测试
ci-test:
    @echo ""
    @echo "🧪 [2/4] 运行测试..."
    @echo "=========================================="
    @echo "→ Rust 测试..."
    cd src-tauri && cargo test
    @echo ""
    @echo "✅ 测试通过"

# CI: 构建验证
ci-build:
    @echo ""
    @echo "🔨 [3/4] 构建验证..."
    @echo "=========================================="
    bun run sync-version
    bun run build
    cd src-tauri && cargo build --release
    @echo ""
    @echo "✅ 构建成功"

# 代码风格检查（lint + fmt check + clippy）
lint:
    @echo ""
    @echo "🔍 [检查] 代码风格..."
    @echo "=========================================="
    @echo "→ 前端 ESLint..."
    bun run lint
    @echo "→ Rust 格式检查..."
    cd src-tauri && cargo fmt --check
    @echo "→ Rust Clippy..."
    cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
    @echo "→ Rust cargo check..."
    cd src-tauri && cargo check

# 运行测试
test:
    bun run test
    cd src-tauri && cargo test

# 运行 Rust 测试
test-rust:
    cd src-tauri && cargo test

# 检查代码（快速检查，不含 clippy）
check:
    bun run lint
    cd src-tauri && cargo check

# 格式化代码
fmt:
    cd src-tauri && cargo fmt
    @echo "Rust 代码格式化完成"

# 清理构建产物
clean:
    rm -rf dist
    rm -rf node_modules
    cd src-tauri && cargo clean

# 启动文档开发服务器
docs:
    cd docs && bun run dev

# 构建文档
docs-build:
    cd docs && bun run build

# 预览构建后的文档
docs-preview:
    cd docs && bun run preview

# 生成图标
icons:
    bunx tauri icon app-icon.svg

# 终止残留的开发服务器进程 (Windows)
[windows]
kill-dev:
    @echo "正在检查并终止残留的开发进程..."
    -powershell -Command "Get-NetTCPConnection -LocalPort 5173 -ErrorAction SilentlyContinue | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue }"
    -powershell -Command "Get-Process -Name 'industry-vis' -ErrorAction SilentlyContinue | Stop-Process -Force"
    @echo "清理完成"

# 终止残留的开发服务器进程 (Unix)
[unix]
kill-dev:
    @echo "正在检查并终止残留的开发进程..."
    -lsof -ti:5173 | xargs -r kill -9
    -pkill -f 'industry-vis' || true
    @echo "清理完成"

# 更新依赖
update:
    bun update
    cd docs && bun update
    cd src-tauri && cargo update
