#!/bin/sh
set -e
if [ -z "$DATABASE_URL" ]; then echo "DATABASE_URL is not set"; exit 1; fi
cmd="$@"
if [ -z "$cmd" ]; then cmd="./app_server"; fi
for i in $(seq 1 30); do if sqlx migrate run; then break; else echo "Waiting for database ($i/30)..."; sleep 2; fi; done
exec $cmd

