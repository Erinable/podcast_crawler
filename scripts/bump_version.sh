#!/bin/bash

# 检查参数
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <major|minor|patch>"
    exit 1
fi

# 获取当前版本
current_version=$(grep -m 1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $current_version"

# 分割版本号
IFS='.' read -r major minor patch <<< "$current_version"

# 更新版本号
case $1 in
    "major")
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    "minor")
        minor=$((minor + 1))
        patch=0
        ;;
    "patch")
        patch=$((patch + 1))
        ;;
    *)
        echo "Invalid version type. Use major, minor, or patch"
        exit 1
        ;;
esac

# 新版本号
new_version="$major.$minor.$patch"
echo "New version: $new_version"

# 更新 Cargo.toml
sed -i.bak "0,/^version = \".*\"/{s/^version = \".*\"/version = \"$new_version\"/}" Cargo.toml
rm Cargo.toml.bak

# 创建 git tag
git add Cargo.toml
git commit -m "chore(release): bump version to $new_version"
git tag -a "v$new_version" -m "Release version $new_version"

echo "Version bumped to $new_version and tag created"
echo "Run 'git push && git push --tags' to publish"
