"""harden supabase rls baseline and index hygiene

Revision ID: 20260225_0013
Revises: 20260223_0012
Create Date: 2026-02-25 00:00:00.000000
"""

from __future__ import annotations

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op


# revision identifiers, used by Alembic.
revision: str = "20260225_0013"
down_revision: Union[str, None] = "20260223_0012"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None

_RLS_POLICY_NAME = "bominal_deny_all"
_RLS_TABLES = (
    "alembic_version",
    "artifacts",
    "auth_challenges",
    "passkey_credentials",
    "password_reset_tokens",
    "roles",
    "secrets",
    "sessions",
    "task_attempts",
    "tasks",
    "users",
    "verification_tokens",
)

_DUPLICATE_INDEXES = (
    ("users", "ix_users_email", {"uq_users_email"}),
    ("roles", "ix_roles_name", {"uq_roles_name"}),
    ("sessions", "ix_sessions_token_hash", {"uq_sessions_token_hash"}),
    (
        "passkey_credentials",
        "ix_passkey_credentials_credential_id",
        {"passkey_credentials_credential_id_key"},
    ),
)


def _existing_tables(inspector: sa.Inspector) -> set[str]:
    return set(inspector.get_table_names())


def _existing_indexes(inspector: sa.Inspector, table_name: str) -> set[str]:
    return {index["name"] for index in inspector.get_indexes(table_name)}


def _existing_unique_constraints(inspector: sa.Inspector, table_name: str) -> set[str]:
    return {constraint["name"] for constraint in inspector.get_unique_constraints(table_name) if constraint.get("name")}


def _table_has_rls(bind: sa.Connection, table_name: str) -> bool:
    row = bind.execute(
        sa.text(
            """
            SELECT c.relrowsecurity
            FROM pg_class AS c
            JOIN pg_namespace AS n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relkind = 'r'
              AND c.relname = :table_name
            """
        ),
        {"table_name": table_name},
    ).scalar_one_or_none()
    return bool(row)


def _policy_exists(bind: sa.Connection, table_name: str, policy_name: str) -> bool:
    row = bind.execute(
        sa.text(
            """
            SELECT 1
            FROM pg_policies
            WHERE schemaname = 'public'
              AND tablename = :table_name
              AND policyname = :policy_name
            LIMIT 1
            """
        ),
        {"table_name": table_name, "policy_name": policy_name},
    ).scalar_one_or_none()
    return bool(row)


def _ensure_deny_all_policy_if_rls_enabled(bind: sa.Connection, table_name: str) -> None:
    if not _table_has_rls(bind, table_name):
        return
    if _policy_exists(bind, table_name, _RLS_POLICY_NAME):
        return
    op.execute(
        sa.text(
            f'CREATE POLICY "{_RLS_POLICY_NAME}" ON "public"."{table_name}" '
            'AS PERMISSIVE FOR ALL TO PUBLIC USING (false) WITH CHECK (false)'
        )
    )


def _drop_duplicate_unique_indexes(inspector: sa.Inspector, table_name: str, duplicate_index: str, peer_names: set[str]) -> None:
    indexes = _existing_indexes(inspector, table_name)
    if duplicate_index not in indexes:
        return

    constraints = _existing_unique_constraints(inspector, table_name)
    if indexes.intersection(peer_names) or constraints.intersection(peer_names):
        op.drop_index(duplicate_index, table_name=table_name)


def upgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    tables = _existing_tables(inspector)

    for table_name in _RLS_TABLES:
        if table_name in tables:
            _ensure_deny_all_policy_if_rls_enabled(bind, table_name)

    if "users" in tables:
        user_indexes = _existing_indexes(inspector, "users")
        if "ix_users_role_id" not in user_indexes:
            op.create_index("ix_users_role_id", "users", ["role_id"], unique=False)

    if "secrets" in tables:
        op.execute(
            """
            CREATE INDEX IF NOT EXISTS ix_secrets_user_kind_updated_desc
            ON secrets (user_id, kind, updated_at DESC)
            """
        )

    for table_name, duplicate_index, peer_names in _DUPLICATE_INDEXES:
        if table_name not in tables:
            continue
        _drop_duplicate_unique_indexes(inspector, table_name, duplicate_index, peer_names)
        inspector = sa.inspect(bind)


def downgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    tables = _existing_tables(inspector)

    if "secrets" in tables:
        op.execute("DROP INDEX IF EXISTS ix_secrets_user_kind_updated_desc")

    if "users" in tables:
        user_indexes = _existing_indexes(inspector, "users")
        if "ix_users_role_id" in user_indexes:
            op.drop_index("ix_users_role_id", table_name="users")

    if "users" in tables:
        op.execute("CREATE UNIQUE INDEX IF NOT EXISTS ix_users_email ON users (email)")
    if "roles" in tables:
        op.execute("CREATE UNIQUE INDEX IF NOT EXISTS ix_roles_name ON roles (name)")
    if "sessions" in tables:
        op.execute("CREATE UNIQUE INDEX IF NOT EXISTS ix_sessions_token_hash ON sessions (token_hash)")
    if "passkey_credentials" in tables:
        op.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS ix_passkey_credentials_credential_id ON passkey_credentials (credential_id)"
        )

    for table_name in _RLS_TABLES:
        if table_name not in tables:
            continue
        op.execute(sa.text(f'DROP POLICY IF EXISTS "{_RLS_POLICY_NAME}" ON "public"."{table_name}"'))
