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

Security:
    This CLI runs inside the API container with full database access.
    Use with caution in production.
"""

from __future__ import annotations

import argparse
import asyncio
import sys
from datetime import datetime, timezone
from uuid import UUID

from sqlalchemy import delete, func, select, text, update

from app.core.config import get_settings
from app.db.models import Role, Secret, Session, Task, TaskAttempt, User
from app.db.session import SessionLocal

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
                select(Task).where(Task.id.cast(str).startswith(task_id))
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
                select(Task).where(Task.id.cast(str).startswith(task_id))
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
        else:
            secret_parser.print_help()


if __name__ == "__main__":
    main()
