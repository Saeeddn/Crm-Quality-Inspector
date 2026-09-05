#!/bin/bash
# Set up Postgres locally for the integration tests.
# - Rewrite pg_hba.conf so TCP auth works with a password (md5)
# - Start the cluster
# - Create the role + db the app expects

set -euo pipefail

PG_VERSION=$(ls /etc/postgresql/)
PG_CONF=/etc/postgresql/$PG_VERSION/main

echo "Detected Postgres version: $PG_VERSION"

# Allow password auth over TCP and local (md5 is fine for tests; scram
# also works but md5 is more forgiving across libpq versions).
sudo sed -i 's/^local\s\+all\s\+all\s\+peer/local all all md5/' $PG_CONF/pg_hba.conf
sudo sed -i 's/^host\s\+all\s\+all\s\+127.0.0.1\/32\s\+scram-sha-256/host all all 127.0.0.1\/32 md5/' $PG_CONF/pg_hba.conf
sudo sed -i 's/^host\s\+all\s\+all\s\+::1\/128\s\+scram-sha-256/host all all ::1\/128 md5/' $PG_CONF/pg_hba.conf

# Listen on TCP — some Ubuntu installs only listen on the Unix socket.
sudo sed -i "s/^#listen_addresses.*/listen_addresses = '127.0.0.1'/" $PG_CONF/postgresql.conf

sudo pg_ctlcluster "$PG_VERSION" main start
sleep 2

# Create role + db via the Unix socket as the postgres OS user.
sudo -u postgres psql -v ON_ERROR_STOP=1 \
  -c "CREATE ROLE \"$POSTGRES_USER\" LOGIN PASSWORD '$POSTGRES_PASSWORD';"
sudo -u postgres psql -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE \"$POSTGRES_DB\" OWNER \"$POSTGRES_USER\";"
sudo -u postgres psql -v ON_ERROR_STOP=1 \
  -c "ALTER ROLE \"$POSTGRES_USER\" CREATEDB;"

echo "Postgres ready: role=$POSTGRES_USER db=$POSTGRES_DB"