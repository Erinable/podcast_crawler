# 开发工作流程指南

本文档详细说明了项目的开发工作流程，包括日常开发、代码审查、测试和发布等环节。

## 目录

1. [开发环境设置](#开发环境设置)
2. [日常开发流程](#日常开发流程)
3. [代码提交规范](#代码提交规范)
4. [自动化工具使用](#自动化工具使用)
5. [测试流程](#测试流程)
6. [发布流程](#发布流程)
7. [问题跟踪](#问题跟踪)

## 开发环境设置

### 1. 初始化开发环境
```bash
# 1. 克隆仓库
git clone https://github.com/your-username/podcast_crawler.git
cd podcast_crawler

# 2. 安装依赖
cargo build

# 3. 配置开发环境
cp .env.example .env
# 编辑 .env 文件设置必要的环境变量

# 4. 安装开发工具
pip install pre-commit
pre-commit install
```

### 2. 配置 Git
```bash
# 设置提交模板
git config --local commit.template .gitmessage
```

## 日常开发流程

### 1. 功能开发
```bash
# 1. 创建功能分支
git checkout -b feature/your-feature-name

# 2. 使用自动化工具生成代码框架
# 生成新模块
./scripts/generate_module.sh src/api your_module

# 生成爬虫模块
./scripts/generate_crawler.sh your_crawler

# 生成数据库迁移
./scripts/generate_migration.sh your_migration

# 3. 开发功能
cargo check    # 检查代码
cargo test    # 运行测试
cargo fmt     # 格式化代码
cargo clippy  # 代码质量检查
```

### 2. 代码提交
```bash
# 1. 检查变更
git status
git diff

# 2. 暂存变更
git add .

# 3. 提交代码（会触发 pre-commit hooks）
git commit
# 使用模板填写提交信息，格式如下：
# feat(scope): add new feature
# fix(scope): fix some bug
# docs(scope): update documentation
```

### 3. 代码审查
1. 推送分支到远程仓库
   ```bash
   git push origin feature/your-feature-name
   ```
2. 创建 Pull Request
3. 等待代码审查
4. 根据反馈修改代码
5. 合并到主分支

## 代码提交规范

### 提交类型
- feat: 新功能
- fix: 错误修复
- docs: 文档更新
- style: 代码格式调整
- refactor: 代码重构
- perf: 性能优化
- test: 测试相关
- build: 构建系统相关
- ci: CI 配置相关
- chore: 其他修改

### 提交信息格式
```
<类型>(<范围>): <简短描述>

<详细描述>

<关闭的问题>
```

示例：
```
feat(crawler): 添加苹果播客支持

- 实现苹果播客 RSS 解析
- 添加播客元数据提取
- 支持音频文件下载

Closes #123
```

## 自动化工具使用

### 1. 模块生成
```bash
# 生成 API 模块
./scripts/generate_module.sh src/api podcast
# 生成爬虫模块
./scripts/generate_crawler.sh apple_podcasts
# 生成数据库迁移
./scripts/generate_migration.sh add_podcast_categories
```

### 2. 版本管理
```bash
# 更新版本号
./scripts/bump_version.sh minor  # major, minor, 或 patch

# 推送新版本
git push && git push --tags
```

### 3. 变更日志
- 变更日志会通过 pre-commit hook 自动更新
- 也可以手动运行：
  ```bash
  ./scripts/update_changelog.sh
  ```

## 测试流程

### 1. 单元测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_name

# 运行特定模块的测试
cargo test --package module_name
```

### 2. 集成测试
```bash
# 运行集成测试
cargo test --test '*'
```

### 3. 基准测试
```bash
cargo bench
```

## 发布流程

### 1. 准备发布
```bash
# 1. 确保主分支是最新的
git checkout main
git pull

# 2. 运行所有测试
cargo test

# 3. 更新版本号
./scripts/bump_version.sh minor
```

### 2. 创建发布
- 推送代码和标签后，GitHub Actions 会自动：
  1. 创建 GitHub Release
  2. 构建二进制文件
  3. 生成变更日志
  4. 上传构建产物

### 3. 发布后检查
1. 验证 GitHub Release 是否创建成功
2. 检查二进制文件是否可以下载
3. 验证变更日志是否正确
4. 确认所有 CI 检查都通过

## 问题跟踪

### 1. 问题报告
创建 Issue 时请包含：
- 问题描述
- 复现步骤
- 期望行为
- 实际行为
- 环境信息

### 2. 问题修复流程
1. 创建修复分支
   ```bash
   git checkout -b fix/issue-number
   ```
2. 修复问题
3. 添加测试
4. 提交代码
5. 创建 Pull Request

### 3. 问题关闭
- 在提交信息中使用 "Closes #issue-number" 自动关闭问题
- 确保所有测试通过
- 获得代码审查批准

## 最佳实践

### 1. 代码质量
- 遵循 Rust 代码规范
- 使用 clippy 进行代码质量检查
- 保持代码简洁和可读性
- 添加适当的注释和文档

### 2. 测试覆盖
- 新功能必须有测试
- 修复 bug 时添加回归测试
- 保持测试简单和可维护

### 3. 性能考虑
- 使用异步操作处理 I/O
- 适当使用缓存
- 注意资源使用效率
- 定期进行性能测试

### 4. 安全实践
- 不提交敏感信息
- 定期更新依赖
- 使用安全的 API
- 进行安全审计
