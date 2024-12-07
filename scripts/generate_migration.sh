#!/bin/bash

# 检查参数
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <migration_name>"
    echo "Example: $0 create_podcasts_table"
    exit 1
fi

MIGRATION_NAME=$1

# 使用 diesel CLI 创建迁移
diesel migration generate $MIGRATION_NAME

# 获取最新创建的迁移目录
MIGRATION_DIR=$(ls -dt migrations/*_${MIGRATION_NAME} | head -n1)

# 创建 up.sql 模板
cat > "$MIGRATION_DIR/up.sql" << EOL
-- 迁移说明：
-- 1. 这个迁移的目的是什么？
-- 2. 它会修改哪些表？
-- 3. 是否有数据迁移？
-- 4. 是否需要注意性能问题？

-- 开始事务
BEGIN;

-- TODO: 在这里添加你的 SQL 语句
-- 例如：
-- CREATE TABLE your_table (
--     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
--     created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
--     updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
-- );

-- 提交事务
COMMIT;
EOL

# 创建 down.sql 模板
cat > "$MIGRATION_DIR/down.sql" << EOL
-- 回滚说明：
-- 1. 这个回滚会撤销哪些改动？
-- 2. 是否会丢失数据？
-- 3. 如何备份可能丢失的数据？

-- 开始事务
BEGIN;

-- TODO: 在这里添加回滚语句
-- 例如：
-- DROP TABLE IF EXISTS your_table;

-- 提交事务
COMMIT;
EOL

echo "Generated migration at $MIGRATION_DIR"
echo "Next steps:"
echo "1. Edit up.sql to add your migration SQL"
echo "2. Edit down.sql to add your rollback SQL"
echo "3. Test the migration: diesel migration run"
echo "4. Test the rollback: diesel migration redo"
