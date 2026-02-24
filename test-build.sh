#!/bin/bash

echo "🔍 Recording King v7.0 - 构建测试"
echo "================================"

echo ""
echo "📊 代码统计："
echo "Rust 文件："
find src-tauri/src -name '*.rs' | wc -l
echo "Rust 代码行数："
find src-tauri/src -name '*.rs' -exec wc -l {} + | tail -1

echo ""
echo "TypeScript/React 文件："
find src -name '*.tsx' -o -name '*.ts' | wc -l
echo "前端代码行数："
find src -name '*.tsx' -o -name '*.ts' -exec wc -l {} + | tail -1

echo ""
echo "📦 检查依赖："
if [ -d "node_modules" ]; then
    echo "✅ Node 依赖已安装"
else
    echo "❌ Node 依赖未安装，运行: npm install"
fi

if [ -f "src-tauri/Cargo.toml" ]; then
    echo "✅ Cargo.toml 存在"
else
    echo "❌ Cargo.toml 不存在"
fi

echo ""
echo "🏗️  尝试编译检查："
echo "检查 Rust 语法..."
cd src-tauri && cargo check 2>&1 | head -20

echo ""
echo "✅ 测试完成"
echo ""
echo "运行开发模式：npm run tauri:dev"
echo "构建生产版本：npm run tauri:build"
