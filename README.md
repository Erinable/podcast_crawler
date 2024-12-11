# Podcast Crawler 🎙️

## Overview

Podcast Crawler is an advanced, async Rust-based podcast management and crawling
system designed for efficient podcast data retrieval, storage, and analysis.

## Features

- 🚀 Asynchronous Rust implementation
- 📦 Diesel ORM for PostgreSQL database interactions
- 🔍 Flexible podcast and episode crawling
- 📊 Advanced querying capabilities
- 🛡️ Robust error handling
- 📝 Comprehensive logging

## Technology Stack

- **Language**: Rust (Edition 2021)
- **Async Runtime**: Tokio
- **ORM**: Diesel
- **Web Framework**: Actix Web
- **Logging**: Tracing

## Prerequisites

- Rust 1.67+ (stable)
- PostgreSQL 12+
- Cargo
- diesel_cli

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/Erinable/podcast_crawler.git
cd podcast_crawler
```

### 2. Install Dependencies

```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 3. Database Setup

```bash
# Create databases
createdb podcast
createdb podcast_test

# Run migrations
diesel migration run
```

### 4. Configuration

Copy `.env.example` to `.env` and configure your settings:

```bash
cp .env.example .env
```

## Development Setup

### 1. Clone the Repository

```bash
git clone https://github.com/Erinable/podcast_crawler.git
cd podcast_crawler
```

### 2. Install Dependencies

```bash
cargo install diesel_cli --no-default-features --features postgres
```

### 3. Database Setup

```bash
# Create databases
createdb podcast
```

### 4. Environment Configuration

Copy `.env.example` to `.env` and configure your settings:

```bash
cp .env.example .env
```

### 5. Run Migrations

```bash
diesel migration run
```

### 6. Build and Run

```bash
# Development build
cargo run

# Release build
cargo run --release
```

### 7. Running Tests

```bash
cargo test
cargo clippy
```

## Makefile Tools 🛠️

The project includes a comprehensive Makefile with various utility commands:

### Development Commands

- `make run`: Run the project in development mode

  ```bash
  # Run in dev mode (default)
  make run

  # Run in release mode
  make run BUILD_TYPE=--release
  ```

### Log Analysis

- `make average`: Calculate average duration from the most recent log file

  ```bash
  make average
  ```

### Quality Checks

- `make pre-commit`: Run pre-commit checks to ensure code quality

  ```bash
  make pre-commit
  ```

### Maintenance Commands

- `make clean`: Clean project build artifacts
- `make test`: Run project tests
- `make doc`: Generate project documentation

### Pro Tips 💡

- Use `BUILD_TYPE=--release` for optimized performance
- Pre-commit checks help maintain code quality
- Log analysis provides insights into crawler performance

## Performance Optimization

- Uses native CPU target optimizations
- Async design for high concurrency
- Connection pooling
- Efficient database queries

## Security

- Environment-based configuration
- Secret detection in pre-commit hooks
- Dependency vulnerability scanning

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit with conventional commits
4. Run pre-commit hooks
5. Submit a pull request

## License

MIT License

## Roadmap

- [ ] Enhanced podcast discovery
- [ ] Machine learning recommendations
- [ ] Advanced analytics
- [ ] Multi-database support

## Metrics

![Build Status](https://img.shields.io/github/workflow/status/Erinable/podcast_crawler/Rust)
![Coverage](https://img.shields.io/codecov/c/github/Erinable/podcast_crawler)
![Downloads](https://img.shields.io/github/downloads/Erinable/podcast_crawler/total)

## Contact

Arrow Tunner - <Mr.han76@outlook.com>

Project Link: [https://github.com/Erinable/podcast_crawler](https://github.com/Erinable/podcast_crawler)
