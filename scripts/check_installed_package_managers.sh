#!/bin/bash

# 检测源的可用性
check_source() {
    local url=$1
    if [[ "$url" == git://* ]]; then
        # 使用 git 检查 git:// 协议
        git ls-remote "$url" &>/dev/null
        if [ $? -eq 0 ]; then
            echo "  可用"
        else
            echo "  不可用"
        fi
    else
        # 使用 curl 检查 http/https 协议
        if curl -Is "$url" | head -n 1 | grep -q "200"; then
            echo "  可用"
        else
            echo "  不可用"
        fi
    fi
}

# 检测 Cargo 镜像源
check_cargo_sources() {
    local global_cargo_config="${CARGO_HOME:-$HOME/.cargo}/config.toml"
    local project_cargo_config="./.cargo/config.toml"

    echo "检测 Cargo 镜像源配置："

    # 检查项目级配置
    if [ -f "$project_cargo_config" ]; then
        echo "项目级配置文件：$project_cargo_config"
        grep "registry" "$project_cargo_config" || echo "  未配置自定义镜像源"
    # 检查全局配置
    elif [ -f "$global_cargo_config" ]; then
        echo "全局配置文件：$global_cargo_config"
        grep "registry" "$global_cargo_config" || echo "  未配置自定义镜像源"
    else
        echo "未找到 Cargo 的配置文件，使用默认源 https://crates.io"
    fi

    # 读取配置并测试源的可用性
    if grep -q "registry" "$global_cargo_config" || grep -q "registry" "$project_cargo_config"; then
        echo "测试配置的镜像源..."
        grep "registry" "$global_cargo_config" "$project_cargo_config" | awk -F '=' '{print $2}' | xargs -I {} check_source {}
    else
        echo "使用默认源 https://crates.io"
        check_source "https://crates.io"
    fi
}

# 检测 NPM 镜像源
check_npm_sources() {
    echo "检测 NPM 镜像源配置："
    npm config get registry
    local npm_registry=$(npm config get registry)
    echo "NPM 镜像源: $npm_registry"
    check_source "$npm_registry"
}

# 检测 PIP 镜像源
check_pip_sources() {
    echo "检测 PIP 镜像源配置："
    local pip_config_file="$HOME/.pip/pip.conf"
    if [ -f "$pip_config_file" ]; then
        echo "PIP 配置文件：$pip_config_file"
        grep "index-url" "$pip_config_file" || echo "  未配置自定义镜像源"
    else
        echo "未找到 PIP 配置文件，使用默认源 https://pypi.org/simple"
    fi
    local pip_registry="https://pypi.org/simple"
    grep -q "index-url" "$pip_config_file" && pip_registry=$(grep "index-url" "$pip_config_file" | awk -F '=' '{print $2}' | xargs)
    echo "PIP 镜像源: $pip_registry"
    check_source "$pip_registry"
}

# 检测 Homebrew 镜像源
check_homebrew_sources() {
    echo "检测 Homebrew 镜像源配置："
    local brew_repo=$(brew --repo)
    local brew_source=$(git -C "$brew_repo" config --get remote.origin.url)
    echo "Homebrew 镜像源: $brew_source"
    check_source "$brew_source"
}

# 检测系统中的包管理工具及其镜像源的稳定性
echo "检测系统中的包管理工具及其镜像源的稳定性："
echo "======================================="

# 检查 Homebrew
if command -v brew &>/dev/null; then
    echo "Homebrew: 已安装"
    check_homebrew_sources
else
    echo "Homebrew: 未安装"
fi

# 检查 Cargo
if command -v cargo &>/dev/null; then
    echo "Cargo: 已安装"
    check_cargo_sources
else
    echo "Cargo: 未安装"
fi

# 检查 NPM
if command -v npm &>/dev/null; then
    echo "NPM: 已安装"
    check_npm_sources
else
    echo "NPM: 未安装"
fi

# 检查 PIP
if command -v pip &>/dev/null; then
    echo "Pip: 已安装"
    check_pip_sources
else
    echo "Pip: 未安装"
fi

echo "======================================="
