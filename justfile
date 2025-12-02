# Industry Vis Justfile
# https://github.com/casey/just

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
    @echo "  just build      快速构建便携版 exe (日常开发)"
    @echo "  just release    构建完整安装包 setup.exe (正式发布)"
    @echo "  just build-web  仅构建前端"
    @echo ""
    @echo "测试命令:"
    @echo "  just test       运行所有测试"
    @echo "  just test-rust  仅运行 Rust 测试"
    @echo "  just check      检查代码 (lint + cargo check)"
    @echo ""
    @echo "文档命令:"
    @echo "  just docs       启动文档开发服务器"
    @echo "  just docs-build 构建文档"
    @echo ""
    @echo "其他命令:"
    @echo "  just install    安装所有依赖"
    @echo "  just clean      清理构建产物"
    @echo "  just fmt        格式化 Rust 代码"
    @echo "  just update     更新所有依赖"
    @echo ""

# 安装依赖
install:
    npm install
    cd docs && npm install

# 开发模式
dev:
    npm run tauri:dev

# 仅前端开发
dev-web:
    npm run dev

# 快速构建便携版（日常开发测试用，只生成 exe）
build:
    npm run build
    cd src-tauri && cargo build --release
    @echo ""
    @echo "构建完成: src-tauri/target/release/industry-vis.exe"

# 构建安装包（正式发布用，生成 setup.exe）
release:
    npm run tauri:build
    @echo ""
    @echo "安装包已生成: src-tauri/target/release/bundle/nsis/"

# 仅构建前端
build-web:
    npm run build

# 运行测试
test:
    npm run test
    cd src-tauri && cargo test

# 运行 Rust 测试
test-rust:
    cd src-tauri && cargo test

# 检查代码
check:
    npm run lint
    cd src-tauri && cargo check

# 格式化代码
fmt:
    cd src-tauri && cargo fmt

# 清理构建产物
clean:
    rm -rf dist
    rm -rf node_modules
    cd src-tauri && cargo clean

# 启动文档开发服务器
docs:
    cd docs && npm run dev

# 构建文档
docs-build:
    cd docs && npm run build

# 预览构建后的文档
docs-preview:
    cd docs && npm run preview

# 生成图标
icons:
    npx tauri icon app-icon.svg

# 更新依赖
update:
    npm update
    cd docs && npm update
    cd src-tauri && cargo update
