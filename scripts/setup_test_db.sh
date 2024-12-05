#!/bin/bash

# Load environment variables
set -a
source .env
set +a

# Run migrations on test database
DATABASE_URL=$TEST_DATABASE_URL diesel migration run

echo "Test database setup complete!"
