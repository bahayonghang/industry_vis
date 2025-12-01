# Industry Vis Justfile
# https://github.com/casey/just

# 默认命令：显示帮助
default:
    @just --list

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

# 构建生产版本
build:
    npm run tauri:build

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
