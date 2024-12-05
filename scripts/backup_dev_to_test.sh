#!/bin/bash

# Load environment variables
set -a
source .env
set +a

echo "Starting backup from dev to test database..."

# Function to get table counts
get_table_counts() {
    local db=$1
    echo "Table counts for $db:"
    for table in podcasts episodes podcast_rank episode_rank; do
        count=$(psql -h localhost -U podcast -d "$db" -t -c "SELECT COUNT(*) FROM $table;")
        echo "$table: $count"
    done
}

# Get development database counts before backup
echo "Getting development database statistics..."
echo "Development database counts:"
get_table_counts "podcast"

# Dump the development database
echo "Dumping development database..."
pg_dump -h localhost -U podcast podcast > dev_backup.sql

if [ $? -ne 0 ]; then
    echo "Error: Failed to dump development database"
    exit 1
fi

# Clean existing data from test database using postgres user
echo "Cleaning test database..."
psql -h localhost -U postgres podcast_test -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT ALL ON SCHEMA public TO podcast;"

if [ $? -ne 0 ]; then
    echo "Error: Failed to clean test database"
    rm dev_backup.sql
    exit 1
fi

# Restore the dump to test database
echo "Restoring data to test database..."
psql -h localhost -U podcast podcast_test < dev_backup.sql

if [ $? -ne 0 ]; then
    echo "Error: Failed to restore to test database"
    rm dev_backup.sql
    exit 1
fi

# Get test database counts after restore
echo -e "\nTest database counts:"
get_table_counts "podcast_test"

# Validate data integrity
echo -e "\nValidating data integrity..."

# Compare podcast titles
echo "Checking podcast titles..."
psql -h localhost -U podcast -d podcast_test -c "
    WITH dev_podcasts AS (
        SELECT id, title FROM dblink('dbname=podcast', 'SELECT id, title FROM podcasts')
        AS t1(id INTEGER, title TEXT)
    ),
    test_podcasts AS (
        SELECT id, title FROM podcasts
    )
    SELECT 'Podcasts with different titles' as check_type,
           COALESCE(d.id, t.id) as id,
           d.title as dev_title,
           t.title as test_title
    FROM dev_podcasts d
    FULL OUTER JOIN test_podcasts t ON d.id = t.id
    WHERE d.title IS DISTINCT FROM t.title
    LIMIT 5;"

# Compare episode counts per podcast
echo "Checking episode counts per podcast..."
psql -h localhost -U podcast -d podcast_test -c "
    WITH dev_counts AS (
        SELECT podcast_id, COUNT(*) as count FROM dblink('dbname=podcast', 'SELECT podcast_id FROM episodes')
        AS t1(podcast_id INTEGER)
        GROUP BY podcast_id
    ),
    test_counts AS (
        SELECT podcast_id, COUNT(*) as count FROM episodes GROUP BY podcast_id
    )
    SELECT 'Episode counts per podcast' as check_type,
           COALESCE(d.podcast_id, t.podcast_id) as podcast_id,
           d.count as dev_count,
           t.count as test_count
    FROM dev_counts d
    FULL OUTER JOIN test_counts t ON d.podcast_id = t.podcast_id
    WHERE d.count IS DISTINCT FROM t.count
    LIMIT 5;"

# Clean up
rm dev_backup.sql

echo -e "\nBackup and validation complete!"
