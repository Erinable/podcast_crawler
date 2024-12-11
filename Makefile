# 默认目标
.DEFAULT_GOAL := help

# 项目配置
CARGO_TARGET_DIR := target
PROJECT_NAME := podcast_crawler

# 最大并行任务数 (根据机器性能调整)
MAKE_JOBS ?= 4

# 是否使用发布模式，默认是 dev 模式
RUN_RELEASE_FLAG ?=

# --------------------------------------
# 目标: 运行 pre-commit 检查
# --------------------------------------

pre-commit:
	@echo "Running pre-commit checks..."
	@pre-commit run --all-files || { echo "Pre-commit check failed"; exit 1; }

# --------------------------------------
# 目标: 运行 cargo 并显示执行情况
# 默认使用开发模式 make run
# 使用发布模式 make run BUILD_TYPE=--release
# --------------------------------------

run:
	@echo "Running $(PROJECT_NAME) and show processing info ..."
	@./scripts/task_analysis.sh run $(RUN_RELEASE_FLAG)

# --------------------------------------
# 目标: 从日志中统计平均执行时间
# --------------------------------------

average:
	@echo "Calculating average duration from logs..."
	@./scripts/task_analysis.sh average

# --------------------------------------
# 目标: 清理构建输出
# --------------------------------------

clean:
	@echo "Cleaning the project..."
	@cargo clean

# --------------------------------------
# 目标: 测试项目
# --------------------------------------

test: pre-commit
	@echo "Running tests..."
	@cargo test

# --------------------------------------
# 目标: 生成指定包的文档并打开浏览器
# --------------------------------------

doc: pre-commit
	@echo "Generating docs for $(PROJECT_NAME)..."
	@cargo doc --package $(PROJECT_NAME) --open

# --------------------------------------
# 目标: 编译发布版本
# --------------------------------------

build-release: pre-commit
	@echo "Building release version..."
	@cargo build --release

# --------------------------------------
# 目标: 并行构建多个包
# --------------------------------------

# 假设你的项目有多个包（例如 podcast_crawler 和其他包），可以并行构建它们
build-parallel: pre-commit
	@echo "Building all packages in parallel..."
	@make -j$(MAKE_JOBS) build-package

build-package:
	@cargo build --package $(PROJECT_NAME)

# --------------------------------------
# 目标: 安装依赖
# --------------------------------------

install-deps: pre-commit
	@echo "Installing dependencies..."
	@cargo install

# --------------------------------------
# 目标: 执行所有常用任务
# --------------------------------------

all: fmt test build-release run-with-log

# --------------------------------------
# 目标: 提供帮助信息
# --------------------------------------

help:
	@echo "\033[1;36m╔══════════════════════════════════════════════════╗\033[0m"
	@echo "\033[1;36m║\033[1;33m  🚀  $(PROJECT_NAME) Makefile Commands  🛠️\033[1;36m       ║\033[0m"
	@echo "\033[1;36m╠══════════════════════════════════════════════════╣\033[0m"
	@echo "\033[1;36m║\033[1;32m  🏃  Development Commands:\033[1;36m                       ║\033[0m"
	@echo "\033[1;36m║\033[0m    \033[1;34mmake run\033[0m            Run project (dev mode)    \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m    \033[1;34mmake run BUILD_TYPE=--release\033[0m                 \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m                        Run in release mode       \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m    \033[1;34mmake average\033[0m        Calculate log durations   \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m    \033[1;34mmake pre-commit\033[0m     Run code quality checks   \033[1;36m║\033[0m"
	@echo "\033[1;36m╟──────────────────────────────────────────────────╢\033[0m"
	@echo "\033[1;36m║\033[1;35m  🧹  Maintenance Commands:\033[1;36m                       ║\033[0m"
	@echo "\033[1;36m║\033[0m    \033[1;34mmake clean\033[0m          Clean build artifacts     \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m    \033[1;34mmake test\033[0m           Run project tests         \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m    \033[1;34mmake doc\033[0m            Generate documentation    \033[1;36m║\033[0m"
	@echo "\033[1;36m╟──────────────────────────────────────────────────╢\033[0m"
	@echo "\033[1;36m║\033[1;33m  💡  Pro Tips:\033[1;36m                                   ║\033[0m"
	@echo "\033[1;36m║\033[0m    • \033[1;31mUse BUILD_TYPE=--release\033[0m                    \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m      for optimized performance                   \033[1;36m║\033[0m"
	@echo "\033[1;36m║\033[0m    • Pre-commit ensures code quality             \033[1;36m║\033[0m"
	@echo "\033[1;36m╚══════════════════════════════════════════════════╝\033[0m"
