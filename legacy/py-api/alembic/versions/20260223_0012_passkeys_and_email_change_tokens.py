"""add passkey credentials, auth challenges, and email-change token metadata

Revision ID: 20260223_0012
Revises: 20260222_0011
Create Date: 2026-02-23 00:00:00.000000
"""

from typing import Sequence, Union

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects import postgresql


# revision identifiers, used by Alembic.
revision: str = "20260223_0012"
down_revision: Union[str, None] = "20260222_0011"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def _existing_columns(inspector: sa.Inspector, table: str) -> set[str]:
    return {column["name"] for column in inspector.get_columns(table)}


def _existing_indexes(inspector: sa.Inspector, table: str) -> set[str]:
    return {index["name"] for index in inspector.get_indexes(table)}


def _existing_tables(inspector: sa.Inspector) -> set[str]:
    return set(inspector.get_table_names())


def upgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    tables = _existing_tables(inspector)

    if "verification_tokens" in tables:
        columns = _existing_columns(inspector, "verification_tokens")
        indexes = _existing_indexes(inspector, "verification_tokens")
        if "purpose" not in columns:
            op.add_column(
                "verification_tokens",
                sa.Column("purpose", sa.String(length=32), nullable=False, server_default="email_verify"),
            )
        if "target_email" not in columns:
            op.add_column(
                "verification_tokens",
                sa.Column("target_email", sa.String(length=320), nullable=True),
            )
        if "ix_verification_tokens_purpose" not in indexes:
            op.create_index("ix_verification_tokens_purpose", "verification_tokens", ["purpose"], unique=False)
        if "ix_verification_tokens_target_email" not in indexes:
            op.create_index("ix_verification_tokens_target_email", "verification_tokens", ["target_email"], unique=False)

    if "passkey_credentials" not in tables:
        op.create_table(
            "passkey_credentials",
            sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
            sa.Column("user_id", postgresql.UUID(as_uuid=True), nullable=False),
            sa.Column("credential_id", sa.String(length=512), nullable=False),
            sa.Column("public_key", sa.Text(), nullable=False),
            sa.Column("sign_count", sa.Integer(), nullable=False, server_default="0"),
            sa.Column("transports", sa.JSON(), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("updated_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("last_used_at", sa.DateTime(timezone=True), nullable=True),
            sa.ForeignKeyConstraint(["user_id"], ["users.id"]),
            sa.PrimaryKeyConstraint("id"),
            sa.UniqueConstraint("credential_id"),
        )
        op.create_index("ix_passkey_credentials_user_id", "passkey_credentials", ["user_id"], unique=False)

    if "auth_challenges" not in tables:
        op.create_table(
            "auth_challenges",
            sa.Column("id", postgresql.UUID(as_uuid=True), nullable=False),
            sa.Column("user_id", postgresql.UUID(as_uuid=True), nullable=True),
            sa.Column("email", sa.String(length=320), nullable=True),
            sa.Column("purpose", sa.String(length=32), nullable=False),
            sa.Column("challenge_hash", sa.String(length=64), nullable=False),
            sa.Column("challenge_b64url", sa.String(length=512), nullable=False),
            sa.Column("expires_at", sa.DateTime(timezone=True), nullable=False),
            sa.Column("used_at", sa.DateTime(timezone=True), nullable=True),
            sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
            sa.ForeignKeyConstraint(["user_id"], ["users.id"]),
            sa.PrimaryKeyConstraint("id"),
        )
        op.create_index("ix_auth_challenges_user_id", "auth_challenges", ["user_id"], unique=False)
        op.create_index("ix_auth_challenges_email", "auth_challenges", ["email"], unique=False)
        op.create_index("ix_auth_challenges_purpose", "auth_challenges", ["purpose"], unique=False)
        op.create_index("ix_auth_challenges_challenge_hash", "auth_challenges", ["challenge_hash"], unique=False)
        op.create_index("ix_auth_challenges_expires_at", "auth_challenges", ["expires_at"], unique=False)


def downgrade() -> None:
    bind = op.get_bind()
    inspector = sa.inspect(bind)
    tables = _existing_tables(inspector)

    if "auth_challenges" in tables:
        indexes = _existing_indexes(inspector, "auth_challenges")
        for name in (
            "ix_auth_challenges_expires_at",
            "ix_auth_challenges_challenge_hash",
            "ix_auth_challenges_purpose",
            "ix_auth_challenges_email",
            "ix_auth_challenges_user_id",
        ):
            if name in indexes:
                op.drop_index(name, table_name="auth_challenges")
        op.drop_table("auth_challenges")

    if "passkey_credentials" in tables:
        indexes = _existing_indexes(inspector, "passkey_credentials")
        for name in (
            "ix_passkey_credentials_user_id",
        ):
            if name in indexes:
                op.drop_index(name, table_name="passkey_credentials")
        op.drop_table("passkey_credentials")

    if "verification_tokens" in tables:
        columns = _existing_columns(inspector, "verification_tokens")
        indexes = _existing_indexes(inspector, "verification_tokens")
        if "ix_verification_tokens_target_email" in indexes:
            op.drop_index("ix_verification_tokens_target_email", table_name="verification_tokens")
        if "ix_verification_tokens_purpose" in indexes:
            op.drop_index("ix_verification_tokens_purpose", table_name="verification_tokens")
        if "target_email" in columns:
            op.drop_column("verification_tokens", "target_email")
        if "purpose" in columns:
            op.drop_column("verification_tokens", "purpose")
