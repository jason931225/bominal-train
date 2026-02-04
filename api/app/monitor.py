#!/usr/bin/env python3
"""
bominal Monitor CLI - Live system status viewer.

Usage:
    python -m app.monitor           # One-time snapshot
    python -m app.monitor --watch   # Live refresh every 3 seconds
    python -m app.monitor --help    # Show help

Shows:
    - Container health status
    - arq worker queue statistics
    - Recent train tasks (last 10)
    - Redis rate limiter status
    - Active database connections
"""

from __future__ import annotations

import argparse
import asyncio
import os
import subprocess
import sys
from datetime import datetime, timedelta, timezone

import redis.asyncio as aioredis
from sqlalchemy import func, select, text
from sqlalchemy.orm import selectinload

from app.core.config import get_settings
from app.db.models import Task, TaskAttempt, User
from app.db.session import SessionLocal

settings = get_settings()

# ANSI colors for terminal output
class Colors:
    HEADER = "\033[95m"
    BLUE = "\033[94m"
    CYAN = "\033[96m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    RESET = "\033[0m"


def color(text: str, c: str) -> str:
    """Wrap text in ANSI color codes."""
    return f"{c}{text}{Colors.RESET}"


def clear_screen() -> None:
    """Clear terminal screen."""
    os.system("clear" if os.name != "nt" else "cls")


def get_container_status() -> list[dict]:
    """Get Docker container status via docker compose ps."""
    try:
        result = subprocess.run(
            ["docker", "compose", "-f", "/opt/bominal/repo/infra/docker-compose.prod.yml", "ps", "--format", "json"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if result.returncode != 0:
            # Try local compose file
            result = subprocess.run(
                ["docker", "compose", "-f", "infra/docker-compose.yml", "ps", "--format", "json"],
                capture_output=True,
                text=True,
                timeout=5,
            )
        
        if result.returncode == 0 and result.stdout.strip():
            import json
            containers = []
            for line in result.stdout.strip().split("\n"):
                if line:
                    try:
                        containers.append(json.loads(line))
                    except json.JSONDecodeError:
                        pass
            return containers
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    return []


async def get_redis_stats(redis_client: aioredis.Redis) -> dict:
    """Get Redis queue and rate limiter statistics."""
    stats = {
        "connected": False,
        "queue_pending": 0,
        "queue_processing": 0,
        "queue_completed": 0,
        "queue_failed": 0,
        "rate_limiters": [],
        "memory_used": "N/A",
        "uptime_seconds": 0,
        "total_connections": 0,
    }
    
    try:
        # Test connection
        await redis_client.ping()
        stats["connected"] = True
        
        # arq queue stats (queue name is 'arq:queue')
        stats["queue_pending"] = await redis_client.zcard("arq:queue") or 0
        stats["queue_processing"] = await redis_client.zcard("arq:in-progress") or 0
        
        # arq result counts (approximate from keys)
        result_keys = []
        async for key in redis_client.scan_iter("arq:result:*"):
            result_keys.append(key)
        stats["queue_completed"] = len(result_keys)
        
        # Get rate limiter keys
        rate_limit_keys = []
        async for key in redis_client.scan_iter("train_rate:*"):
            rate_limit_keys.append(key)
        
        for key in rate_limit_keys[:5]:  # Show first 5
            ttl = await redis_client.ttl(key)
            tokens = await redis_client.get(key)
            stats["rate_limiters"].append({
                "key": key.decode() if isinstance(key, bytes) else key,
                "tokens": tokens.decode() if tokens else "0",
                "ttl": ttl,
            })
        
        # Server info
        info = await redis_client.info("memory")
        stats["memory_used"] = info.get("used_memory_human", "N/A")
        
        server_info = await redis_client.info("server")
        stats["uptime_seconds"] = server_info.get("uptime_in_seconds", 0)
        
        client_info = await redis_client.info("clients")
        stats["total_connections"] = client_info.get("connected_clients", 0)
        
    except Exception as e:
        stats["error"] = str(e)
    
    return stats


async def get_db_stats() -> dict:
    """Get database statistics."""
    stats = {
        "connected": False,
        "total_users": 0,
        "total_tasks": 0,
        "active_tasks": 0,
        "tasks_today": 0,
        "failed_today": 0,
        "completed_today": 0,
        "total_attempts_today": 0,
        "error_rate_today": 0.0,
        "recent_tasks": [],
    }
    
    try:
        async with SessionLocal() as db:
            stats["connected"] = True
            
            # User count
            result = await db.execute(select(func.count(User.id)))
            stats["total_users"] = result.scalar() or 0
            
            # Task counts
            result = await db.execute(select(func.count(Task.id)))
            stats["total_tasks"] = result.scalar() or 0
            
            # Active tasks (pending/running)
            result = await db.execute(
                select(func.count(Task.id)).where(
                    Task.state.in_(["pending", "running"]),
                    Task.hidden_at.is_(None),
                )
            )
            stats["active_tasks"] = result.scalar() or 0
            
            # Tasks created today
            today_start = datetime.now(timezone.utc).replace(hour=0, minute=0, second=0, microsecond=0)
            result = await db.execute(
                select(func.count(Task.id)).where(Task.created_at >= today_start)
            )
            stats["tasks_today"] = result.scalar() or 0
            
            # Completed today
            result = await db.execute(
                select(func.count(Task.id)).where(
                    Task.completed_at >= today_start
                )
            )
            stats["completed_today"] = result.scalar() or 0
            
            # Failed today
            result = await db.execute(
                select(func.count(Task.id)).where(
                    Task.failed_at >= today_start
                )
            )
            stats["failed_today"] = result.scalar() or 0
            
            # Today's attempt stats (for error rate)
            result = await db.execute(
                select(func.count(TaskAttempt.id)).where(
                    TaskAttempt.started_at >= today_start
                )
            )
            stats["total_attempts_today"] = result.scalar() or 0
            
            result = await db.execute(
                select(func.count(TaskAttempt.id)).where(
                    TaskAttempt.started_at >= today_start,
                    TaskAttempt.ok == False,  # noqa: E712
                )
            )
            failed_attempts = result.scalar() or 0
            
            if stats["total_attempts_today"] > 0:
                stats["error_rate_today"] = (failed_attempts / stats["total_attempts_today"]) * 100
            
            # Recent tasks with attempts (include hidden for full visibility)
            result = await db.execute(
                select(Task)
                .options(selectinload(Task.attempts))
                .order_by(Task.updated_at.desc())
                .limit(10)
            )
            tasks = result.scalars().all()
            
            for task in tasks:
                # Get latest attempt from eagerly loaded attempts
                attempts = sorted(task.attempts, key=lambda a: a.started_at, reverse=True) if task.attempts else []
                latest_attempt = attempts[0] if attempts else None
                
                stats["recent_tasks"].append({
                    "id": str(task.id)[:8],
                    "module": task.module,
                    "state": task.state,
                    "hidden": task.hidden_at is not None,
                    "created_at": task.created_at,
                    "updated_at": task.updated_at,
                    "attempts": len(attempts),
                    "last_error": latest_attempt.error_code if latest_attempt and not latest_attempt.ok else None,
                })
    
    except Exception as e:
        stats["error"] = str(e)
    
    return stats


def format_time_ago(dt: datetime) -> str:
    """Format datetime as 'X ago' string."""
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    
    now = datetime.now(timezone.utc)
    diff = now - dt
    
    if diff < timedelta(minutes=1):
        return f"{int(diff.total_seconds())}s ago"
    elif diff < timedelta(hours=1):
        return f"{int(diff.total_seconds() / 60)}m ago"
    elif diff < timedelta(days=1):
        return f"{int(diff.total_seconds() / 3600)}h ago"
    else:
        return f"{diff.days}d ago"


def state_color(state: str) -> str:
    """Get color for task state."""
    colors = {
        "pending": Colors.YELLOW,
        "running": Colors.BLUE,
        "completed": Colors.GREEN,
        "failed": Colors.RED,
        "cancelled": Colors.DIM,
        "paused": Colors.CYAN,
    }
    return colors.get(state, Colors.RESET)


def print_header(title: str) -> None:
    """Print section header."""
    print(f"\n{color('═' * 60, Colors.DIM)}")
    print(f"{color('  ' + title, Colors.BOLD + Colors.CYAN)}")
    print(f"{color('═' * 60, Colors.DIM)}")


def print_containers(containers: list[dict]) -> None:
    """Print container status table."""
    print_header("🐳 Container Status")
    
    if not containers:
        print(f"  {color('Unable to get container status', Colors.YELLOW)}")
        return
    
    for c in containers:
        name = c.get("Name", c.get("Service", "unknown"))
        state = c.get("State", c.get("Status", "unknown"))
        health = c.get("Health", "")
        
        # Determine status color
        if "healthy" in health.lower() or state == "running":
            status_color = Colors.GREEN
            icon = "✓"
        elif "unhealthy" in health.lower():
            status_color = Colors.RED
            icon = "✗"
        elif state == "exited":
            status_color = Colors.RED
            icon = "✗"
        else:
            status_color = Colors.YELLOW
            icon = "?"
        
        status_text = f"{state}" + (f" ({health})" if health else "")
        print(f"  {color(icon, status_color)} {name:20} {color(status_text, status_color)}")


def format_uptime(seconds: int) -> str:
    """Format uptime in human-readable form."""
    if seconds < 60:
        return f"{seconds}s"
    elif seconds < 3600:
        return f"{seconds // 60}m"
    elif seconds < 86400:
        return f"{seconds // 3600}h {(seconds % 3600) // 60}m"
    else:
        days = seconds // 86400
        hours = (seconds % 86400) // 3600
        return f"{days}d {hours}h"


def print_redis_stats(stats: dict) -> None:
    """Print Redis statistics."""
    print_header("📊 Queue & Rate Limiting (Redis)")
    
    if not stats["connected"]:
        print(f"  {color('✗ Redis disconnected', Colors.RED)}")
        if "error" in stats:
            print(f"    {color(stats['error'], Colors.DIM)}")
        return
    
    uptime_str = format_uptime(stats.get("uptime_seconds", 0))
    print(f"  {color('✓', Colors.GREEN)} Connected  |  Memory: {stats['memory_used']}  |  Uptime: {uptime_str}  |  Clients: {stats.get('total_connections', 0)}")
    print()
    print(f"  {color('arq Queue:', Colors.BOLD)}")
    print(f"    Pending:    {color(str(stats['queue_pending']), Colors.YELLOW if stats['queue_pending'] > 0 else Colors.DIM)}")
    print(f"    Processing: {color(str(stats['queue_processing']), Colors.BLUE if stats['queue_processing'] > 0 else Colors.DIM)}")
    print(f"    Completed:  {color(str(stats.get('queue_completed', 0)), Colors.GREEN if stats.get('queue_completed', 0) > 0 else Colors.DIM)}")
    
    if stats["rate_limiters"]:
        print()
        print(f"  {color('Rate Limiters:', Colors.BOLD)}")
        for rl in stats["rate_limiters"]:
            key_short = rl["key"].replace("train_rate:", "")[:30]
            print(f"    {key_short}: {rl['tokens']} tokens, TTL {rl['ttl']}s")


def print_db_stats(stats: dict) -> None:
    """Print database statistics."""
    print_header("🗄️  Database (PostgreSQL)")
    
    if not stats["connected"]:
        print(f"  {color('✗ Database disconnected', Colors.RED)}")
        if "error" in stats:
            print(f"    {color(stats['error'], Colors.DIM)}")
        return
    
    print(f"  {color('✓', Colors.GREEN)} Connected")
    print()
    print(f"  Users:        {stats['total_users']}")
    print(f"  Total Tasks:  {stats['total_tasks']}")
    print(f"  Active Tasks: {color(str(stats['active_tasks']), Colors.YELLOW if stats['active_tasks'] > 0 else Colors.DIM)}")
    
    print()
    print(f"  {color('Today:', Colors.BOLD)}")
    print(f"    Created:    {stats['tasks_today']}")
    print(f"    Completed:  {color(str(stats.get('completed_today', 0)), Colors.GREEN if stats.get('completed_today', 0) > 0 else Colors.DIM)}")
    print(f"    Failed:     {color(str(stats.get('failed_today', 0)), Colors.RED if stats.get('failed_today', 0) > 0 else Colors.DIM)}")
    
    error_rate = stats.get('error_rate_today', 0)
    attempts = stats.get('total_attempts_today', 0)
    if attempts > 0:
        error_color = Colors.RED if error_rate > 50 else Colors.YELLOW if error_rate > 20 else Colors.GREEN
        print(f"    Attempts:   {attempts} ({color(f'{error_rate:.1f}% errors', error_color)})")


def print_recent_tasks(tasks: list[dict]) -> None:
    """Print recent tasks table."""
    print_header("📋 Recent Tasks (Last 10)")
    
    if not tasks:
        print(f"  {color('No tasks found', Colors.DIM)}")
        return
    
    # Header
    print(f"  {'ID':8} {'Module':8} {'State':12} {'#':3} {'Updated':10} {'Error':16}")
    print(f"  {'-' * 8} {'-' * 8} {'-' * 12} {'-' * 3} {'-' * 10} {'-' * 16}")
    
    for t in tasks:
        state_display = t["state"]
        if t.get("hidden"):
            state_display = f"{t['state']}(H)"
        state_str = color(state_display, state_color(t["state"]))
        error_str = color(t["last_error"] or "-", Colors.RED if t["last_error"] else Colors.DIM)
        updated_str = format_time_ago(t["updated_at"])
        
        print(f"  {t['id']:8} {t['module']:8} {state_str:21} {t['attempts']:^3} {updated_str:10} {error_str}")


async def run_monitor(watch: bool = False, interval: int = 3) -> None:
    """Run the monitor display."""
    redis_client = aioredis.from_url(settings.redis_url, decode_responses=False)
    
    try:
        while True:
            if watch:
                clear_screen()
            
            # Header
            now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
            env = settings.app_env.upper()
            env_color = Colors.RED if env == "PRODUCTION" else Colors.GREEN
            
            print(f"\n{color('bominal Monitor', Colors.BOLD + Colors.CYAN)} | {color(env, env_color)} | {now}")
            
            # Gather all stats concurrently
            containers = get_container_status()
            redis_stats, db_stats = await asyncio.gather(
                get_redis_stats(redis_client),
                get_db_stats(),
            )
            
            # Print sections
            print_containers(containers)
            print_redis_stats(redis_stats)
            print_db_stats(db_stats)
            print_recent_tasks(db_stats.get("recent_tasks", []))
            
            print(f"\n{color('─' * 60, Colors.DIM)}")
            if watch:
                print(f"{color(f'Refreshing every {interval}s... Press Ctrl+C to exit', Colors.DIM)}")
            
            if not watch:
                break
            
            await asyncio.sleep(interval)
    
    except KeyboardInterrupt:
        print(f"\n{color('Monitor stopped.', Colors.DIM)}")
    
    finally:
        await redis_client.aclose()


def main() -> None:
    """CLI entry point."""
    parser = argparse.ArgumentParser(
        description="bominal Monitor - Live system status viewer",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python -m app.monitor           One-time snapshot
  python -m app.monitor --watch   Live refresh (Ctrl+C to stop)
  python -m app.monitor -w -i 5   Refresh every 5 seconds
        """,
    )
    parser.add_argument(
        "-w", "--watch",
        action="store_true",
        help="Continuously refresh the display",
    )
    parser.add_argument(
        "-i", "--interval",
        type=int,
        default=3,
        help="Refresh interval in seconds (default: 3)",
    )
    
    args = parser.parse_args()
    
    asyncio.run(run_monitor(watch=args.watch, interval=args.interval))


if __name__ == "__main__":
    main()
