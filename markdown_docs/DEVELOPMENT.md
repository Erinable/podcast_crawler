# 开发指南

## 开发环境设置

1. 安装依赖
   - Rust (使用 rustup)
   - PostgreSQL
   - Redis (可选)
   - Docker (可选，用于容器化开发)

2. 环境配置
   - 复制 `.env.example` 到 `.env`
   - 配置数据库连接
   - 配置其他必要的环境变量

3. 数据库设置

   ```bash
   cargo install diesel_cli --no-default-features --features postgres
   diesel setup
   diesel migration run
   ```

4. 运行测试

   ```bash
   cargo test
   ```

5. 运行开发服务器

   ```bash
   cargo run
   ```

## 代码规范

1. 代码格式
   - 使用 `cargo fmt` 格式化代码
   - 使用 `cargo clippy` 进行代码检查

2. 提交规范
   - 遵循 Conventional Commits
   - 使用提供的 git commit 模板
   - 每次提交前会自动运行 pre-commit hooks

3. 错误处理
   - 使用自定义的错误类型（详见 `src/infrastructure/error/`）
   - 使用 `try_with_log!` 和 `try_with_warn!` 宏
   - 确保错误信息清晰可追踪

4. 日志规范
   - 使用适当的日志级别（error, warn, info, debug, trace）
   - 包含必要的上下文信息
   - 避免敏感信息泄露
   - 使用结构化日志格式（JSON）

## 测试规范

1. 单元测试
   - 每个模块都应有对应的测试
   - 使用 `#[cfg(test)]` 标记测试模块
   - 测试覆盖率要求 >= 80%
   - 运行单元测试：`cargo test --lib`

2. 集成测试
   - 主要功能流程的端到端测试
   - API 接口测试
   - 数据库交互测试
   - 运行集成测试：`cargo test --test '*'`

## 性能基准测试

1. 使用 `criterion` 进行基准测试

   ```bash
   cargo bench
   ```

2. 基准测试位于 `benches/` 目录
3. 定期运行基准测试并记录结果

## 发布流程

1. 更新版本号（Cargo.toml）
2. CHANGELOG.md 会通过 pre-commit hook 自动更新
3. 创建发布标签：

   ```bash
   git tag -a v0.1.0 -m "Release version 0.1.0"
   git push origin v0.1.0
   ```

## 故障排除

1. 常见问题
   - 数据库连接问题：检查 DATABASE_URL 环境变量
   - 编译错误：运行 `cargo clean` 后重试
   - 测试失败：确保测试数据库已正确设置

2. 调试技巧
   - 使用 `RUST_LOG=debug cargo run` 查看详细日志
   - 使用 `RUST_BACKTRACE=1` 查看完整堆栈跟踪
   - 使用 rust-lldb 或 rust-gdb 进行调试

3. 日志分析
   - 日志位于 `logs/` 目录
   - 使用 `jq` 分析 JSON 格式日志
   - 错误日志包含错误码和上下文信息
