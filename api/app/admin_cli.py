#!/usr/bin/env python3
"""
bominal Admin CLI - Administrative commands for user and task management.

Usage:
    python -m app.admin_cli user list                    # List all users
    python -m app.admin_cli user promote <email>         # Promote user to admin
    python -m app.admin_cli user demote <email>          # Demote admin to user
    python -m app.admin_cli user info <email>            # Show user details
    
    python -m app.admin_cli task list [--all]            # List tasks
    python -m app.admin_cli task cancel <id>             # Cancel a task
    python -m app.admin_cli task unhide <id>             # Unhide a task
    
    python -m app.admin_cli db stats                     # Database statistics
    python -m app.admin_cli db vacuum                    # Run VACUUM ANALYZE
    
    python -m app.admin_cli secret check                 # Check encryption status
    python -m app.admin_cli secret purge-payment --yes       # Purge all saved wallet card/CVV data
    python -m app.admin_cli secret purge-payment-cvv --yes   # Purge only cached CVV keys

Security:
    This CLI runs inside the API container with full database access.
    Use with caution in production.
"""

from __future__ import annotations

import argparse
import asyncio
import base64
import json
import os
import sys
from datetime import datetime, timedelta, timezone
from uuid import UUID

from sqlalchemy import String, delete, func, select, text, update

from app.core.config import get_settings
from app.core.crypto.service import get_envelope_crypto
from app.db.models import Role, Secret, Session, Task, TaskAttempt, User
from app.db.session import SessionLocal
from app.services.wallet import purge_all_saved_payment_data, purge_cached_payment_cvv_data

settings = get_settings()


# ANSI colors
class Colors:
    BOLD = "\033[1m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    CYAN = "\033[96m"
    DIM = "\033[2m"
    RESET = "\033[0m"


def color(text: str, c: str) -> str:
    return f"{c}{text}{Colors.RESET}"


def print_table(headers: list[str], rows: list[list[str]]) -> None:
    """Print a formatted table."""
    if not rows:
        print(color("  No data found", Colors.DIM))
        return
    
    # Calculate column widths
    widths = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            if i < len(widths):
                widths[i] = max(widths[i], len(str(cell)))
    
    # Print header
    header_line = "  " + "  ".join(h.ljust(widths[i]) for i, h in enumerate(headers))
    print(color(header_line, Colors.BOLD))
    print("  " + "  ".join("-" * w for w in widths))
    
    # Print rows
    for row in rows:
        line = "  " + "  ".join(str(cell).ljust(widths[i]) for i, cell in enumerate(row))
        print(line)


def _current_kek_keyring() -> dict[int, str]:
    """Return a normalized keyring that always includes the active KEK version."""
    keyring = {int(version): str(key) for version, key in (settings.master_keys_by_version or {}).items()}
    keyring.setdefault(int(settings.kek_version), settings.master_key)
    return keyring


def _serialize_keyring_for_env(keyring: dict[int, str]) -> str:
    payload = {str(version): keyring[version] for version in sorted(keyring)}
    return json.dumps(payload, separators=(",", ":"), ensure_ascii=True)


def _generate_master_key_b64() -> str:
    return base64.b64encode(os.urandom(32)).decode("utf-8")


def _parse_rotation_completed_at(timestamp: str) -> datetime:
    normalized = timestamp.strip()
    if normalized.endswith("Z"):
        normalized = f"{normalized[:-1]}+00:00"
    parsed = datetime.fromisoformat(normalized)
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


# ============================================================================
# User Commands
# ============================================================================

async def user_list() -> None:
    """List all users."""
    async with SessionLocal() as db:
        result = await db.execute(
            select(User, Role)
            .join(Role)
            .order_by(User.created_at.desc())
        )
        users = result.all()
        
        print(f"\n{color('Users', Colors.BOLD + Colors.CYAN)}\n")
        
        rows = []
        for user, role in users:
            rows.append([
                str(user.id)[:8],
                user.email[:30],
                user.display_name or "-",
                role.name,
                user.created_at.strftime("%Y-%m-%d"),
            ])
        
        print_table(["ID", "Email", "Display Name", "Role", "Created"], rows)
        print(f"\n  Total: {len(users)} users\n")


async def user_info(email: str) -> None:
    """Show detailed user information."""
    async with SessionLocal() as db:
        result = await db.execute(
            select(User, Role)
            .join(Role)
            .where(User.email == email)
        )
        row = result.first()
        
        if not row:
            print(color(f"\n  User not found: {email}\n", Colors.RED))
            return
        
        user, role = row
        
        # Get session count
        session_result = await db.execute(
            select(func.count(Session.id)).where(
                Session.user_id == user.id,
                Session.revoked_at.is_(None),
                Session.expires_at > datetime.now(timezone.utc),
            )
        )
        active_sessions = session_result.scalar() or 0
        
        # Get task count
        task_result = await db.execute(
            select(func.count(Task.id)).where(Task.user_id == user.id)
        )
        total_tasks = task_result.scalar() or 0
        
        # Get secret count
        secret_result = await db.execute(
            select(func.count(Secret.id)).where(Secret.user_id == user.id)
        )
        total_secrets = secret_result.scalar() or 0
        
        print(f"\n{color('User Details', Colors.BOLD + Colors.CYAN)}\n")
        print(f"  ID:              {user.id}")
        print(f"  Email:           {user.email}")
        print(f"  Display Name:    {user.display_name or '-'}")
        print(f"  Phone:           {user.phone_number or '-'}")
        print(f"  Role:            {color(role.name, Colors.GREEN if role.name == 'admin' else Colors.RESET)}")
        print(f"  Email Verified:  {'Yes' if user.email_verified_at else 'No'}")
        print(f"  Created:         {user.created_at.strftime('%Y-%m-%d %H:%M:%S UTC')}")
        print(f"  Updated:         {user.updated_at.strftime('%Y-%m-%d %H:%M:%S UTC')}")
        print(f"\n  Active Sessions: {active_sessions}")
        print(f"  Total Tasks:     {total_tasks}")
        print(f"  Stored Secrets:  {total_secrets}\n")


async def user_promote(email: str) -> None:
    """Promote a user to admin role."""
    async with SessionLocal() as db:
        # Get admin role
        role_result = await db.execute(select(Role).where(Role.name == "admin"))
        admin_role = role_result.scalar()
        
        if not admin_role:
            print(color("\n  Error: Admin role not found in database\n", Colors.RED))
            return
        
        # Get user
        user_result = await db.execute(select(User).where(User.email == email))
        user = user_result.scalar()
        
        if not user:
            print(color(f"\n  User not found: {email}\n", Colors.RED))
            return
        
        if user.role_id == admin_role.id:
            print(color(f"\n  User {email} is already an admin\n", Colors.YELLOW))
            return
        
        # Update role
        user.role_id = admin_role.id
        await db.commit()
        
        print(color(f"\n  ✓ User {email} promoted to admin\n", Colors.GREEN))


async def user_demote(email: str) -> None:
    """Demote an admin to regular user role."""
    async with SessionLocal() as db:
        # Get user role
        role_result = await db.execute(select(Role).where(Role.name == "user"))
        user_role = role_result.scalar()
        
        if not user_role:
            print(color("\n  Error: User role not found in database\n", Colors.RED))
            return
        
        # Get user
        user_result = await db.execute(select(User).where(User.email == email))
        user = user_result.scalar()
        
        if not user:
            print(color(f"\n  User not found: {email}\n", Colors.RED))
            return
        
        if user.role_id == user_role.id:
            print(color(f"\n  User {email} is already a regular user\n", Colors.YELLOW))
            return
        
        # Update role
        user.role_id = user_role.id
        await db.commit()
        
        print(color(f"\n  ✓ User {email} demoted to regular user\n", Colors.GREEN))


# ============================================================================
# Task Commands
# ============================================================================

async def task_list(show_all: bool = False) -> None:
    """List tasks."""
    async with SessionLocal() as db:
        query = select(Task, User).join(User).order_by(Task.updated_at.desc())
        
        if not show_all:
            query = query.where(Task.hidden_at.is_(None))
        
        query = query.limit(50)
        
        result = await db.execute(query)
        tasks = result.all()
        
        title = "All Tasks (including hidden)" if show_all else "Visible Tasks"
        print(f"\n{color(title, Colors.BOLD + Colors.CYAN)}\n")
        
        rows = []
        for task, user in tasks:
            state = task.state
            if task.hidden_at:
                state += "(H)"
            
            rows.append([
                str(task.id)[:8],
                user.email[:20],
                task.module,
                state,
                task.updated_at.strftime("%m-%d %H:%M"),
            ])
        
        print_table(["ID", "User", "Module", "State", "Updated"], rows)
        print(f"\n  Showing {len(tasks)} tasks" + (" (use --all to include hidden)" if not show_all else "") + "\n")


async def task_cancel(task_id: str) -> None:
    """Cancel a task."""
    async with SessionLocal() as db:
        try:
            uuid = UUID(task_id)
        except ValueError:
            # Try prefix match
            result = await db.execute(
                select(Task).where(Task.id.cast(String).startswith(task_id))
            )
            task = result.scalar()
            if not task:
                print(color(f"\n  Task not found: {task_id}\n", Colors.RED))
                return
        else:
            result = await db.execute(select(Task).where(Task.id == uuid))
            task = result.scalar()
        
        if not task:
            print(color(f"\n  Task not found: {task_id}\n", Colors.RED))
            return
        
        if task.state in ["completed", "failed", "cancelled"]:
            print(color(f"\n  Task {task_id} is already {task.state}\n", Colors.YELLOW))
            return
        
        task.state = "cancelled"
        task.cancelled_at = datetime.now(timezone.utc)
        await db.commit()
        
        print(color(f"\n  ✓ Task {task.id} cancelled\n", Colors.GREEN))


async def task_unhide(task_id: str) -> None:
    """Unhide a hidden task."""
    async with SessionLocal() as db:
        try:
            uuid = UUID(task_id)
        except ValueError:
            result = await db.execute(
                select(Task).where(Task.id.cast(String).startswith(task_id))
            )
            task = result.scalar()
            if not task:
                print(color(f"\n  Task not found: {task_id}\n", Colors.RED))
                return
        else:
            result = await db.execute(select(Task).where(Task.id == uuid))
            task = result.scalar()
        
        if not task:
            print(color(f"\n  Task not found: {task_id}\n", Colors.RED))
            return
        
        if not task.hidden_at:
            print(color(f"\n  Task {task_id} is not hidden\n", Colors.YELLOW))
            return
        
        task.hidden_at = None
        await db.commit()
        
        print(color(f"\n  ✓ Task {task.id} unhidden\n", Colors.GREEN))


# ============================================================================
# Database Commands
# ============================================================================

async def db_stats() -> None:
    """Show database statistics."""
    async with SessionLocal() as db:
        print(f"\n{color('Database Statistics', Colors.BOLD + Colors.CYAN)}\n")
        
        # Table counts
        tables = [
            ("Users", User),
            ("Sessions", Session),
            ("Tasks", Task),
            ("Task Attempts", TaskAttempt),
            ("Secrets", Secret),
        ]
        
        rows = []
        for name, model in tables:
            result = await db.execute(select(func.count()).select_from(model))
            count = result.scalar() or 0
            rows.append([name, str(count)])
        
        print_table(["Table", "Count"], rows)
        
        # Database size
        result = await db.execute(text("SELECT pg_database_size(current_database())"))
        db_size = result.scalar() or 0
        db_size_mb = db_size / (1024 * 1024)
        
        print(f"\n  Database Size: {db_size_mb:.2f} MB\n")
        
        # Active connections
        result = await db.execute(text(
            "SELECT count(*) FROM pg_stat_activity WHERE datname = current_database()"
        ))
        connections = result.scalar() or 0
        print(f"  Active Connections: {connections}\n")


async def db_vacuum() -> None:
    """Run VACUUM ANALYZE on the database."""
    print(f"\n{color('Running VACUUM ANALYZE...', Colors.CYAN)}")
    
    # Need to use raw connection for VACUUM
    from sqlalchemy import create_engine
    sync_engine = create_engine(settings.sync_database_url)
    
    with sync_engine.connect() as conn:
        conn.execution_options(isolation_level="AUTOCOMMIT")
        conn.execute(text("VACUUM ANALYZE"))
    
    print(color("  ✓ VACUUM ANALYZE completed\n", Colors.GREEN))


# ============================================================================
# Secret Commands
# ============================================================================

async def secret_check() -> None:
    """Check encryption status of secrets."""
    async with SessionLocal() as db:
        print(f"\n{color('Encryption Status', Colors.BOLD + Colors.CYAN)}\n")
        
        # Count secrets by KEK version
        result = await db.execute(
            select(Secret.kek_version, func.count(Secret.id))
            .group_by(Secret.kek_version)
            .order_by(Secret.kek_version)
        )
        versions = result.all()
        
        if not versions:
            print(color("  No encrypted secrets found\n", Colors.DIM))
            return
        
        rows = []
        for version, count in versions:
            rows.append([str(version), str(count)])
        
        print_table(["KEK Version", "Secret Count"], rows)
        
        current_version = settings.kek_version
        print(f"\n  Current KEK Version: {current_version}")
        
        # Check if any need rotation
        old_count = sum(count for version, count in versions if version < current_version)
        if old_count > 0:
            print(color(f"  ⚠ {old_count} secrets need rotation to current KEK\n", Colors.YELLOW))
        else:
            print(color("  ✓ All secrets using current KEK version\n", Colors.GREEN))


async def secret_purge_payment(*, yes: bool) -> None:
    """Purge all stored wallet payment-card data and cached CVV keys."""
    if not yes:
        print(color("\n  Refusing to purge payment data without --yes confirmation.\n", Colors.YELLOW))
        return

    async with SessionLocal() as db:
        summary = await purge_all_saved_payment_data(db)

    print(f"\n{color('Payment Data Purge', Colors.BOLD + Colors.CYAN)}\n")
    print(f"  Deleted DB payment_card secrets: {summary['db_payment_card_secrets_deleted']}")
    print(f"  Deleted Redis CVV keys (current): {summary['redis_cvv_keys_deleted_current']}")
    print(f"  Deleted Redis CVV keys (legacy):  {summary['redis_cvv_keys_deleted_legacy']}")
    print(f"  Deleted Redis CVV keys (total):   {summary['redis_cvv_keys_deleted_total']}\n")


async def secret_purge_payment_cvv(*, yes: bool) -> None:
    """Purge only cached CVV keys from Redis (does not delete saved card secrets)."""
    if not yes:
        print(color("\n  Refusing to purge cached CVV data without --yes confirmation.\n", Colors.YELLOW))
        return

    summary = await purge_cached_payment_cvv_data()

    print(f"\n{color('Payment CVV Cache Purge', Colors.BOLD + Colors.CYAN)}\n")
    print(f"  Deleted Redis CVV keys (current): {summary['redis_cvv_keys_deleted_current']}")
    print(f"  Deleted Redis CVV keys (legacy):  {summary['redis_cvv_keys_deleted_legacy']}")
    print(f"  Deleted Redis CVV keys (total):   {summary['redis_cvv_keys_deleted_total']}\n")


def secret_prepare_kek_rotation(*, new_version: int) -> None:
    """Generate a new KEK version payload and print env updates for primary switch."""
    if new_version < 1:
        print(color("\n  KEK version must be >= 1.\n", Colors.RED))
        return

    current_version = int(settings.kek_version)
    keyring = _current_kek_keyring()
    if new_version in keyring:
        print(color(f"\n  KEK version {new_version} already exists in keyring.\n", Colors.YELLOW))
        return

    new_master_key = _generate_master_key_b64()
    updated_keyring = dict(keyring)
    updated_keyring[new_version] = new_master_key

    print(f"\n{color('KEK Rotation Prepare', Colors.BOLD + Colors.CYAN)}\n")
    print(f"  Current primary KEK:   {current_version}")
    print(f"  New primary KEK:       {new_version}")
    print("  Old KEKs retained for unwrapping: yes\n")
    print(color("  Apply these env values for the next deploy:", Colors.BOLD))
    print(f"  KEK_VERSION={new_version}")
    print(f"  MASTER_KEY={new_master_key}")
    print(f"  MASTER_KEYS_BY_VERSION='{_serialize_keyring_for_env(updated_keyring)}'\n")
    print("  Next steps:")
    print("  1) Deploy API with the new KEK_VERSION payload above.")
    print("  2) Run background rewrap: `bominal-admin secret rotate-kek-background --yes`.")
    print("  3) After rewrap + retention window, retire old KEKs with `retire-kek`.\n")


async def secret_rotate_kek(*, yes: bool, dry_run: bool, limit: int | None) -> None:
    """Rewrap existing secrets to the currently configured KEK_VERSION."""
    if not dry_run and not yes:
        print(color("\n  Refusing to rotate secrets without --yes confirmation.\n", Colors.YELLOW))
        return

    crypto = get_envelope_crypto()
    target_version = settings.kek_version

    async with SessionLocal() as db:
        stmt = (
            select(Secret)
            .where(Secret.kek_version != target_version)
            .order_by(Secret.updated_at.asc(), Secret.id.asc())
        )
        if limit is not None and limit > 0:
            stmt = stmt.limit(limit)

        secrets_to_rotate = list((await db.execute(stmt)).scalars().all())
        scanned = len(secrets_to_rotate)
        rotated = 0
        skipped = 0
        failed = 0
        failures: list[tuple[str, str]] = []

        for secret in secrets_to_rotate:
            try:
                payload = crypto.decrypt_payload(
                    ciphertext=secret.ciphertext,
                    nonce=secret.nonce,
                    wrapped_dek=secret.wrapped_dek,
                    dek_nonce=secret.dek_nonce,
                    aad=secret.aad,
                    kek_version=secret.kek_version,
                    enforce_kek_version=True,
                )
                aad_text = f"{secret.kind}:{secret.user_id}"
                rewrapped = crypto.encrypt_payload(payload=payload, aad_text=aad_text)
            except Exception as exc:
                failed += 1
                failures.append((str(secret.id), type(exc).__name__))
                continue

            if dry_run:
                skipped += 1
                continue

            secret.ciphertext = rewrapped.ciphertext
            secret.nonce = rewrapped.nonce
            secret.wrapped_dek = rewrapped.wrapped_dek
            secret.dek_nonce = rewrapped.dek_nonce
            secret.aad = rewrapped.aad
            secret.kek_version = rewrapped.kek_version
            rotated += 1

        if rotated > 0 and not dry_run:
            await db.commit()

    mode = "DRY RUN" if dry_run else "EXECUTE"
    print(f"\n{color('Secret KEK Rotation', Colors.BOLD + Colors.CYAN)}")
    print(f"  Mode:            {mode}")
    print(f"  Target version:  {target_version}")
    print(f"  Scanned:         {scanned}")
    print(f"  Rotated:         {rotated}")
    print(f"  Skipped:         {skipped}")
    print(f"  Failed:          {failed}\n")
    if failures:
        print(color("  Failed secrets:", Colors.YELLOW))
        for secret_id, error_name in failures[:10]:
            print(f"    - {secret_id}: {error_name}")
        if len(failures) > 10:
            print(f"    ... and {len(failures) - 10} more")
        print()


async def secret_rotate_kek_background(*, yes: bool, batch_size: int, sleep_seconds: float) -> None:
    """Continuously rewrap secrets in batches until rotation completes or stalls."""
    if not yes:
        print(color("\n  Refusing background KEK rotation without --yes confirmation.\n", Colors.YELLOW))
        return
    if batch_size < 1:
        print(color("\n  Batch size must be >= 1.\n", Colors.RED))
        return
    if sleep_seconds < 0:
        print(color("\n  Sleep seconds must be >= 0.\n", Colors.RED))
        return

    print(f"\n{color('KEK Background Rotation', Colors.BOLD + Colors.CYAN)}")
    print(f"  Target version: {settings.kek_version}")
    print(f"  Batch size:     {batch_size}")
    print(f"  Sleep seconds:  {sleep_seconds}\n")

    runs = 0
    previous_remaining: int | None = None

    while True:
        runs += 1
        print(color(f"  Batch run #{runs}", Colors.BOLD))
        await secret_rotate_kek(yes=True, dry_run=False, limit=batch_size)

        async with SessionLocal() as db:
            remaining_result = await db.execute(
                select(func.count(Secret.id)).where(Secret.kek_version != settings.kek_version)
            )
            remaining = int(remaining_result.scalar() or 0)

        # Re-query aggregate counters from latest run output is not structured.
        # We infer progress from remaining secrets and batch strategy.
        # If remaining is zero, rotation is complete.
        if remaining == 0:
            completed_at = datetime.now(timezone.utc)
            retire_after = completed_at + timedelta(days=settings.kek_retirement_window_days)
            print(color("\n  Background KEK rotation completed.\n", Colors.GREEN))
            print(f"  Rotation completed at (UTC): {completed_at.isoformat()}")
            print(f"  Old KEK retirement not before: {retire_after.isoformat()}\n")
            return

        # No structured per-batch stats are exposed from secret_rotate_kek; if remaining is
        # less than batch size we make one more attempt, then require operator investigation.
        if remaining <= batch_size:
            print(color("  Remaining set is small; running one final pass.", Colors.DIM))
            await secret_rotate_kek(yes=True, dry_run=False, limit=batch_size)
            async with SessionLocal() as db:
                remaining_result = await db.execute(
                    select(func.count(Secret.id)).where(Secret.kek_version != settings.kek_version)
                )
                remaining = int(remaining_result.scalar() or 0)
            if remaining > 0:
                print(color("\n  Rotation stalled with remaining secrets; investigate decrypt failures.\n", Colors.RED))
                return
            completed_at = datetime.now(timezone.utc)
            retire_after = completed_at + timedelta(days=settings.kek_retirement_window_days)
            print(color("\n  Background KEK rotation completed.\n", Colors.GREEN))
            print(f"  Rotation completed at (UTC): {completed_at.isoformat()}")
            print(f"  Old KEK retirement not before: {retire_after.isoformat()}\n")
            return

        if previous_remaining is not None and remaining >= previous_remaining:
            print(color("\n  Rotation stalled without progress; investigate decrypt failures.\n", Colors.RED))
            return
        previous_remaining = remaining

        if sleep_seconds > 0:
            await asyncio.sleep(sleep_seconds)


async def secret_retire_kek(*, version: int, yes: bool, rotation_completed_at: str) -> None:
    """Retire an old KEK from active keyring after zero-usage + retention window."""
    if not yes:
        print(color("\n  Refusing KEK retirement without --yes confirmation.\n", Colors.YELLOW))
        return
    if version < 1:
        print(color("\n  KEK version must be >= 1.\n", Colors.RED))
        return
    if version == int(settings.kek_version):
        print(color("\n  Cannot retire the active KEK_VERSION.\n", Colors.RED))
        return

    keyring = _current_kek_keyring()
    if version not in keyring:
        print(color(f"\n  KEK version {version} is not present in current keyring.\n", Colors.YELLOW))
        return

    try:
        rotation_completed_utc = _parse_rotation_completed_at(rotation_completed_at)
    except Exception:
        print(color("\n  Invalid --rotation-completed-at value. Use ISO-8601 UTC timestamp.\n", Colors.RED))
        return

    retire_not_before = rotation_completed_utc + timedelta(days=settings.kek_retirement_window_days)
    now_utc = datetime.now(timezone.utc)
    if now_utc < retire_not_before:
        print(color("\n  Retention window has not elapsed; refusing KEK retirement.\n", Colors.YELLOW))
        print(f"  Rotation completed at (UTC): {rotation_completed_utc.isoformat()}")
        print(f"  Retire not before (UTC):     {retire_not_before.isoformat()}\n")
        return

    async with SessionLocal() as db:
        remaining_result = await db.execute(
            select(func.count(Secret.id)).where(Secret.kek_version == version)
        )
        remaining = int(remaining_result.scalar() or 0)
    if remaining > 0:
        print(color("\n  Refusing KEK retirement: secrets still reference this version.\n", Colors.RED))
        print(f"  Remaining secrets on KEK {version}: {remaining}\n")
        return

    updated_keyring = {k: v for k, v in keyring.items() if k != version}
    print(f"\n{color('KEK Retirement Ready', Colors.BOLD + Colors.CYAN)}\n")
    print(f"  Retired version:  {version}")
    print(f"  Active version:   {settings.kek_version}")
    print("  Remaining refs:   0\n")
    print(color("  Apply these env values for deploy:", Colors.BOLD))
    print(f"  KEK_VERSION={settings.kek_version}")
    print(f"  MASTER_KEYS_BY_VERSION='{_serialize_keyring_for_env(updated_keyring)}'\n")


# ============================================================================
# Main
# ============================================================================

def main() -> None:
    parser = argparse.ArgumentParser(
        description="bominal Admin CLI - Administrative commands",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # User commands
    user_parser = subparsers.add_parser("user", help="User management")
    user_sub = user_parser.add_subparsers(dest="action")
    
    user_sub.add_parser("list", help="List all users")
    
    user_info_parser = user_sub.add_parser("info", help="Show user details")
    user_info_parser.add_argument("email", help="User email")
    
    user_promote_parser = user_sub.add_parser("promote", help="Promote user to admin")
    user_promote_parser.add_argument("email", help="User email")
    
    user_demote_parser = user_sub.add_parser("demote", help="Demote admin to user")
    user_demote_parser.add_argument("email", help="User email")
    
    # Task commands
    task_parser = subparsers.add_parser("task", help="Task management")
    task_sub = task_parser.add_subparsers(dest="action")
    
    task_list_parser = task_sub.add_parser("list", help="List tasks")
    task_list_parser.add_argument("--all", "-a", action="store_true", help="Include hidden tasks")
    
    task_cancel_parser = task_sub.add_parser("cancel", help="Cancel a task")
    task_cancel_parser.add_argument("id", help="Task ID (full or prefix)")
    
    task_unhide_parser = task_sub.add_parser("unhide", help="Unhide a task")
    task_unhide_parser.add_argument("id", help="Task ID (full or prefix)")
    
    # Database commands
    db_parser = subparsers.add_parser("db", help="Database operations")
    db_sub = db_parser.add_subparsers(dest="action")
    
    db_sub.add_parser("stats", help="Show database statistics")
    db_sub.add_parser("vacuum", help="Run VACUUM ANALYZE")
    
    # Secret commands
    secret_parser = subparsers.add_parser("secret", help="Secret/encryption operations")
    secret_sub = secret_parser.add_subparsers(dest="action")
    
    secret_sub.add_parser("check", help="Check encryption status")
    secret_rotate_parser = secret_sub.add_parser(
        "rotate-kek",
        help="Rotate existing encrypted secrets to current KEK_VERSION",
    )
    secret_rotate_parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Scan/decrypt/rewrap in memory without writing DB changes",
    )
    secret_rotate_parser.add_argument(
        "--yes",
        action="store_true",
        help="Required confirmation flag for execute mode",
    )
    secret_rotate_parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Optional max number of secrets to process",
    )
    secret_prepare_parser = secret_sub.add_parser(
        "prepare-kek-rotation",
        help="Generate a new KEK version payload and promote it as primary for wrapping",
    )
    secret_prepare_parser.add_argument(
        "--new-version",
        type=int,
        required=True,
        help="New KEK version number to add to keyring and set active",
    )
    secret_rotate_bg_parser = secret_sub.add_parser(
        "rotate-kek-background",
        help="Run continuous batch rewrap until all secrets use current KEK_VERSION",
    )
    secret_rotate_bg_parser.add_argument(
        "--yes",
        action="store_true",
        help="Required confirmation flag for execute mode",
    )
    secret_rotate_bg_parser.add_argument(
        "--batch-size",
        type=int,
        default=200,
        help="Number of secrets to process per batch",
    )
    secret_rotate_bg_parser.add_argument(
        "--sleep-seconds",
        type=float,
        default=0.5,
        help="Pause between batch runs",
    )
    secret_retire_parser = secret_sub.add_parser(
        "retire-kek",
        help="Retire an old KEK version after rewrap completion and retention window",
    )
    secret_retire_parser.add_argument(
        "--version",
        type=int,
        required=True,
        help="KEK version to retire from keyring",
    )
    secret_retire_parser.add_argument(
        "--rotation-completed-at",
        type=str,
        required=True,
        help="UTC timestamp when rewrap completed (ISO-8601)",
    )
    secret_retire_parser.add_argument(
        "--yes",
        action="store_true",
        help="Required confirmation flag for retirement readiness output",
    )
    secret_purge_parser = secret_sub.add_parser(
        "purge-payment",
        help="Purge all saved payment card secrets and cached CVV data",
    )
    secret_purge_parser.add_argument(
        "--yes",
        action="store_true",
        help="Required confirmation flag to execute irreversible payment-data purge",
    )
    secret_purge_cvv_parser = secret_sub.add_parser(
        "purge-payment-cvv",
        help="Purge cached CVV data only (keeps saved payment card secrets)",
    )
    secret_purge_cvv_parser.add_argument(
        "--yes",
        action="store_true",
        help="Required confirmation flag to execute CVV-cache purge",
    )
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    # Route to appropriate handler
    if args.command == "user":
        if args.action == "list":
            asyncio.run(user_list())
        elif args.action == "info":
            asyncio.run(user_info(args.email))
        elif args.action == "promote":
            asyncio.run(user_promote(args.email))
        elif args.action == "demote":
            asyncio.run(user_demote(args.email))
        else:
            user_parser.print_help()
    
    elif args.command == "task":
        if args.action == "list":
            asyncio.run(task_list(show_all=args.all))
        elif args.action == "cancel":
            asyncio.run(task_cancel(args.id))
        elif args.action == "unhide":
            asyncio.run(task_unhide(args.id))
        else:
            task_parser.print_help()
    
    elif args.command == "db":
        if args.action == "stats":
            asyncio.run(db_stats())
        elif args.action == "vacuum":
            asyncio.run(db_vacuum())
        else:
            db_parser.print_help()
    
    elif args.command == "secret":
        if args.action == "check":
            asyncio.run(secret_check())
        elif args.action == "prepare-kek-rotation":
            secret_prepare_kek_rotation(new_version=args.new_version)
        elif args.action == "rotate-kek":
            asyncio.run(secret_rotate_kek(yes=args.yes, dry_run=args.dry_run, limit=args.limit))
        elif args.action == "rotate-kek-background":
            asyncio.run(
                secret_rotate_kek_background(
                    yes=args.yes,
                    batch_size=args.batch_size,
                    sleep_seconds=args.sleep_seconds,
                )
            )
        elif args.action == "retire-kek":
            asyncio.run(
                secret_retire_kek(
                    version=args.version,
                    yes=args.yes,
                    rotation_completed_at=args.rotation_completed_at,
                )
            )
        elif args.action == "purge-payment":
            asyncio.run(secret_purge_payment(yes=args.yes))
        elif args.action == "purge-payment-cvv":
            asyncio.run(secret_purge_payment_cvv(yes=args.yes))
        else:
            secret_parser.print_help()


if __name__ == "__main__":
    main()
