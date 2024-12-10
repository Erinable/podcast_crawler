# API 文档

## 认证

目前处于开发阶段，暂未实现认证机制。后续会添加基于 JWT 的认证。

## 端点

### Podcast 相关接口

#### 获取播客列表

```http
GET /api/podcasts
```

查询参数：

- `page`: 页码（默认：1）
- `per_page`: 每页数量（默认：20）
- `sort`: 排序字段（可选：title, updated_at）
- `order`: 排序方向（asc/desc）

响应格式：

```json
{
  "data": [
    {
      "id": "uuid",
      "title": "播客标题",
      "description": "描述",
      "author": "作者",
      "website": "网站",
      "feed_url": "RSS feed URL",
      "image_url": "封面图片 URL",
      "created_at": "创建时间",
      "updated_at": "更新时间"
    }
  ],
  "meta": {
    "total": 100,
    "page": 1,
    "per_page": 20,
    "total_pages": 5
  }
}
```

#### 获取单个播客

```http
GET /api/podcasts/{id}
```

响应格式：

```json
{
  "data": {
    "id": "uuid",
    "title": "播客标题",
    "description": "描述",
    "author": "作者",
    "website": "网站",
    "feed_url": "RSS feed URL",
    "image_url": "封面图片 URL",
    "episodes": [
      {
        "id": "uuid",
        "title": "标题",
        "description": "描述",
        "audio_url": "音频 URL",
        "duration": "时长",
        "published_at": "发布时间"
      }
    ],
    "created_at": "创建时间",
    "updated_at": "更新时间"
  }
}
```

### 抓取任务相关接口

#### 创建抓取任务

```http
POST /api/crawl
```

请求体：

```json
{
  "feed_url": "要抓取的 RSS feed URL",
  "force_update": false
}
```

响应格式：

```json
{
  "data": {
    "task_id": "uuid",
    "status": "pending",
    "created_at": "创建时间"
  }
}
```

#### 获取任务状态

```http
GET /api/crawl/{task_id}
```

响应格式：

```json
{
  "data": {
    "task_id": "uuid",
    "status": "running|completed|failed",
    "progress": 80,
    "error": "错误信息（如果失败）",
    "created_at": "创建时间",
    "updated_at": "更新时间"
  }
}
```

## 错误处理

所有错误响应使用统一格式：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述",
    "details": {
      "字段": "具体错误"
    }
  }
}
```

常见错误码：

- `INVALID_REQUEST`: 请求格式错误
- `NOT_FOUND`: 资源不存在
- `VALIDATION_ERROR`: 数据验证失败
- `CRAWLER_ERROR`: 抓取过程错误
- `DATABASE_ERROR`: 数据库操作错误
- `INTERNAL_ERROR`: 内部服务器错误

## 限流策略

- 每个 IP 每分钟最多 60 个请求
- 抓取任务每个 IP 每小时最多 10 个
- 超出限制返回 429 状态码

## 版本控制

- 当前版本：v1
- API 版本通过 URL 前缀指定：`/api/v1/...`
- 重大更改会增加版本号
- 向后兼容的更改在当前版本进行
