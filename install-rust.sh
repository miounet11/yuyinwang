#!/bin/bash

echo "🦀 Recording King v7.0 - Rust 安装脚本"
echo "========================================"
echo ""

# 检查是否已安装 Rust
if command -v rustc &> /dev/null; then
    echo "✅ Rust 已安装"
    rustc --version
    cargo --version
    echo ""
    echo "跳过安装，继续下一步..."
else
    echo "📥 开始安装 Rust..."
    echo ""

    # 安装 Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    # 加载环境变量
    source "$HOME/.cargo/env"

    echo ""
    echo "✅ Rust 安装完成"
    rustc --version
    cargo --version
fi

echo ""
echo "📦 安装 Tauri CLI..."
cargo install tauri-cli --version ^1.6

echo ""
echo "✅ 安装完成！"
echo ""
echo "下一步："
echo "  npm run tauri:dev"
echo ""
