#!/bin/bash

# 获取最新的 git tag
latest_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
echo "Latest tag: $latest_tag"

# 获取当前日期
current_date=$(date +"%Y-%m-%d")

# 获取从上一个tag到现在的所有commit
commits=$(git log --pretty=format:"* %s (%h)" ${latest_tag}..HEAD)

# 如果没有新的commit，退出
if [ -z "$commits" ]; then
    echo "No new commits since last tag"
    exit 0
fi

# 准备新的changelog内容
changelog_content="## [Unreleased] - ${current_date}\n\n${commits}\n\n"

# 将新内容添加到CHANGELOG.md的顶部
if [ -f CHANGELOG.md ]; then
    # 保存原有内容
    original_content=$(cat CHANGELOG.md)
    # 写入新内容
    echo -e "# Changelog\n\n${changelog_content}${original_content}" > CHANGELOG.md
else
    # 如果文件不存在，创建新文件
    echo -e "# Changelog\n\n${changelog_content}" > CHANGELOG.md
fi

echo "CHANGELOG.md has been updated"
