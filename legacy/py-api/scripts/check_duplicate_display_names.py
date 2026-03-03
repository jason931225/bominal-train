#!/usr/bin/env python3
from __future__ import annotations

import os
import sys

from sqlalchemy import MetaData, Table, create_engine, func, inspect, select
from sqlalchemy.engine import Engine


def resolve_sync_database_url() -> str | None:
    sync_url = os.getenv("SYNC_DATABASE_URL")
    if sync_url:
        return sync_url

    database_url = os.getenv("DATABASE_URL")
    if not database_url:
        return None

    if database_url.startswith("postgresql+asyncpg://"):
        return database_url.replace("postgresql+asyncpg://", "postgresql+psycopg://", 1)
    if database_url.startswith("sqlite+aiosqlite://"):
        return database_url.replace("sqlite+aiosqlite://", "sqlite://", 1)
    return database_url


def build_engine() -> Engine:
    database_url = resolve_sync_database_url()
    if not database_url:
        print("Missing SYNC_DATABASE_URL or DATABASE_URL.", file=sys.stderr)
        sys.exit(1)
    return create_engine(database_url, future=True)


def main() -> int:
    engine = build_engine()
    with engine.connect() as connection:
        inspector = MetaData()
        if not inspect(connection).has_table("users"):
            print("Display name duplicate check skipped: users table does not exist yet.")
            return 0

        users = Table("users", inspector, autoload_with=connection)
        if "display_name" not in users.c:
            print("Display name duplicate check skipped: users.display_name column does not exist yet.")
            return 0

        normalized_name = func.lower(func.trim(users.c.display_name))
        duplicate_rows = connection.execute(
            select(normalized_name.label("normalized_name"), func.count().label("duplicate_count"))
            .where(users.c.display_name.is_not(None))
            .where(func.trim(users.c.display_name) != "")
            .group_by(normalized_name)
            .having(func.count() > 1)
            .order_by(func.count().desc(), normalized_name.asc())
        ).all()

        if not duplicate_rows:
            print("Display name duplicate check passed: no duplicates found.")
            return 0

        print("Display name duplicate check failed. Duplicate normalized display names found:")
        for row in duplicate_rows:
            normalized = str(row.normalized_name)
            accounts = connection.execute(
                select(users.c.email, users.c.display_name)
                .where(func.lower(func.trim(users.c.display_name)) == normalized)
                .order_by(users.c.email.asc())
            ).all()
            formatted_accounts = ", ".join(f"{email} ({display_name})" for email, display_name in accounts)
            print(f" - {normalized}: {int(row.duplicate_count)} accounts -> {formatted_accounts}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
