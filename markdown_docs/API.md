# 播客系统 API 文档

## 监控指标接口

### 1. 获取监控指标

- **路径**: `/metrics`
- **方法**: GET
- **功能**: 获取系统运行监控指标
- **响应**: Prometheus 格式的监控数据

### 2. 添加任务

- **路径**: `/add_task`
- **方法**: POST
- **功能**: 添加新的 RSS 爬取任务
- **请求体**:

  ```json
  {
    "rss_url": "string"
  }

## 播客查询接口

### 1. 搜索播客

- 路径: `/podcasts/search`
- 方法: GET
- 参数:
  - q: 搜索关键词
- 功能: 按标题搜索播客

### 2. 获取播客列表

- 路径: `/podcasts`
- 方法: GET
- 参数:
  - include_episodes: 是否包含剧集信息(可选)
- 功能: 获取播客列表

## 3. 分页获取播客

- 路径: `/podcasts/page/{page}/{per_page}`
- 方法: GET
- 参数:
  - page: 页码
  - per_page: 每页数量
- 功能: 分页获取播客列表

## 4. 按标题获取播客

- 路径: `/podcasts/by-title/{title}`
- 方法: GET
- 功能: 根据播客标题获取详细信息

## 5. 获取播客剧集

- 路径: `/podcasts/{id}/episodes/{page}/{per_page}`
- 方法: GET
- 参数:
  - id: 播客ID
  - page: 页码
  - per_page: 每页数量
- 功能: 分页获取指定播客的剧集列表
