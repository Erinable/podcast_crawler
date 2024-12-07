# 部署指南

## 系统要求

### 硬件要求（最小配置）
- CPU: 2 核
- 内存: 4GB RAM
- 存储: 20GB SSD

### 软件要求
- OS: Ubuntu 20.04 LTS 或更高版本
- Rust 1.70+
- PostgreSQL 14+
- Redis 6+
- Docker (可选)
- Nginx (可选，用作反向代理)

## 环境准备

### 1. 系统配置
```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装基础依赖
sudo apt install -y build-essential pkg-config libssl-dev
```

### 2. 安装 Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. 安装 PostgreSQL
```bash
sudo apt install -y postgresql postgresql-contrib
sudo systemctl enable postgresql
sudo systemctl start postgresql
```

### 4. 安装 Redis
```bash
sudo apt install -y redis-server
sudo systemctl enable redis-server
sudo systemctl start redis-server
```

## 部署步骤

### 1. 代码部署
```bash
# 克隆代码
git clone https://github.com/your-username/podcast_crawler.git
cd podcast_crawler

# 编译发布版本
cargo build --release
```

### 2. 数据库配置
```bash
# 创建数据库用户和数据库
sudo -u postgres psql
postgres=# CREATE USER podcast_crawler WITH PASSWORD 'your_password';
postgres=# CREATE DATABASE podcast_crawler OWNER podcast_crawler;
postgres=# \q

# 运行数据库迁移
DATABASE_URL=postgres://podcast_crawler:your_password@localhost/podcast_crawler \
diesel migration run
```

### 3. 环境变量配置
```bash
# 复制环境变量模板
cp .env.example .env

# 编辑环境变量
nano .env
```

必要的环境变量：
- DATABASE_URL
- REDIS_URL
- SERVER_HOST
- SERVER_PORT
- LOG_LEVEL
- RUST_LOG

### 4. 系统服务配置

创建系统服务文件：
```bash
sudo nano /etc/systemd/system/podcast-crawler.service
```

服务配置内容：
```ini
[Unit]
Description=Podcast Crawler Service
After=network.target postgresql.service redis-server.service

[Service]
Type=simple
User=podcast_crawler
Group=podcast_crawler
WorkingDirectory=/path/to/podcast_crawler
EnvironmentFile=/path/to/podcast_crawler/.env
ExecStart=/path/to/podcast_crawler/target/release/podcast_crawler
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable podcast-crawler
sudo systemctl start podcast-crawler
```

## 监控配置

### 1. 日志配置
```bash
# 创建日志目录
mkdir -p /var/log/podcast_crawler
chown podcast_crawler:podcast_crawler /var/log/podcast_crawler

# 配置日志轮转
sudo nano /etc/logrotate.d/podcast-crawler
```

日志轮转配置：
```
/var/log/podcast_crawler/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 podcast_crawler podcast_crawler
}
```

### 2. 性能监控
- 使用 Prometheus 收集指标
- 配置 Grafana 仪表板
- 设置告警规则

## 备份策略

### 1. 数据库备份
```bash
# 创建备份脚本
nano /usr/local/bin/backup-podcast-crawler.sh
```

备份脚本内容：
```bash
#!/bin/bash
BACKUP_DIR="/var/backups/podcast_crawler"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# 数据库备份
pg_dump -U podcast_crawler podcast_crawler > $BACKUP_DIR/db_$TIMESTAMP.sql

# 压缩备份
gzip $BACKUP_DIR/db_$TIMESTAMP.sql

# 保留最近 7 天的备份
find $BACKUP_DIR -name "db_*.sql.gz" -mtime +7 -delete
```

设置定时任务：
```bash
chmod +x /usr/local/bin/backup-podcast-crawler.sh
echo "0 2 * * * /usr/local/bin/backup-podcast-crawler.sh" | sudo tee -a /etc/crontab
```

## 安全配置

### 1. 防火墙配置
```bash
# 配置 UFW
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable
```

### 2. SSL 配置（使用 Let's Encrypt）
```bash
# 安装 certbot
sudo apt install -y certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com
```

### 3. 系统安全
- 定期更新系统包
- 配置 fail2ban
- 禁用 root SSH 登录

## 故障恢复

### 1. 服务故障
```bash
# 检查服务状态
sudo systemctl status podcast-crawler

# 查看日志
sudo journalctl -u podcast-crawler -f

# 重启服务
sudo systemctl restart podcast-crawler
```

### 2. 数据库恢复
```bash
# 从备份恢复
gunzip -c /var/backups/podcast_crawler/db_TIMESTAMP.sql.gz | \
psql -U podcast_crawler podcast_crawler
```

## 扩展配置

### 1. 负载均衡（使用 Nginx）
```nginx
upstream podcast_crawler {
    server 127.0.0.1:8080;
    server 127.0.0.1:8081;
}

server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://podcast_crawler;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 2. 容器化部署
使用 Docker Compose 进行部署，配置文件见项目根目录的 `docker-compose.yml`。
