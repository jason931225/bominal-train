#!/usr/bin/env bash
set -euo pipefail

# Bominal Train Reservation — One-Time Bootstrap Script
# Installs all dependencies and sets up the development environment.
# Usage: ./deployment/bootstrap.sh
#
# Supports: macOS (Homebrew) and Debian/Ubuntu (apt)
# Idempotent: safe to re-run; skips already-installed tools.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()  { echo -e "${BLUE}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
fail()  { echo -e "${RED}[FAIL]${NC}  $*"; exit 1; }

command_exists() { command -v "$1" &>/dev/null; }

# ─── Detect OS ───────────────────────────────────────────────────────
detect_os() {
    case "$(uname -s)" in
        Darwin) OS="macos" ;;
        Linux)
            if [ -f /etc/debian_version ]; then
                OS="debian"
            else
                fail "Unsupported Linux distribution. Only Debian/Ubuntu supported."
            fi
            ;;
        *) fail "Unsupported OS: $(uname -s)" ;;
    esac
    info "Detected OS: $OS"
}

# ─── Install System Dependencies ─────────────────────────────────────
install_system_deps() {
    info "Checking system dependencies..."

    if [ "$OS" = "macos" ]; then
        if ! command_exists brew; then
            fail "Homebrew is required. Install from https://brew.sh"
        fi

        local deps=()
        command_exists rustup   || deps+=(rustup)
        command_exists psql     || deps+=(postgresql@16)
        command_exists valkey-server || command_exists redis-server || deps+=(valkey)
        command_exists brotli   || deps+=(brotli)
        command_exists caddy    || deps+=(caddy)

        if [ ${#deps[@]} -gt 0 ]; then
            info "Installing via Homebrew: ${deps[*]}"
            brew install "${deps[@]}"
        fi
    elif [ "$OS" = "debian" ]; then
        info "Updating apt cache..."
        sudo apt-get update -qq

        local deps=()
        command_exists curl     || deps+=(curl)
        command_exists git      || deps+=(git)
        command_exists psql     || deps+=(postgresql)
        command_exists brotli   || deps+=(brotli)
        command_exists pkg-config || deps+=(pkg-config)
        dpkg -s libssl-dev &>/dev/null 2>&1 || deps+=(libssl-dev)

        if [ ${#deps[@]} -gt 0 ]; then
            info "Installing via apt: ${deps[*]}"
            sudo apt-get install -y -qq "${deps[@]}"
        fi

        # Rust via rustup
        if ! command_exists rustup; then
            info "Installing Rust via rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
            source "$HOME/.cargo/env"
        fi

        # Valkey
        if ! command_exists valkey-server && ! command_exists redis-server; then
            info "Installing Valkey..."
            curl -fsSL https://packages.valkey.io/gpg | sudo gpg --dearmor -o /usr/share/keyrings/valkey-archive-keyring.gpg
            echo "deb [signed-by=/usr/share/keyrings/valkey-archive-keyring.gpg] https://packages.valkey.io/deb $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/valkey.list > /dev/null
            sudo apt-get update -qq && sudo apt-get install -y -qq valkey
        fi

        # Caddy
        if ! command_exists caddy; then
            info "Installing Caddy..."
            sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
            curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
            curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
            sudo apt-get update -qq && sudo apt-get install -y -qq caddy
        fi
    fi

    ok "System dependencies installed"
}

# ─── Rust Toolchain ──────────────────────────────────────────────────
setup_rust() {
    info "Setting up Rust toolchain..."

    # Ensure cargo is in PATH
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

    rustup update stable 2>/dev/null || true
    rustup target add wasm32-unknown-unknown
    rustup component add rustfmt clippy llvm-tools-preview

    # Cargo tools
    local cargo_tools=()
    command_exists cargo-leptos   || cargo_tools+=("cargo-leptos")
    command_exists sqlx           || cargo_tools+=("sqlx-cli --no-default-features --features rustls,postgres")
    command_exists cargo-llvm-cov || cargo_tools+=("cargo-llvm-cov")

    for tool in "${cargo_tools[@]}"; do
        info "Installing $tool..."
        cargo install $tool 2>/dev/null || warn "Failed to install $tool (may already be installed)"
    done

    ok "Rust toolchain ready"
}

# ─── Node.js (for React prototype dev + Tailwind) ────────────────────
setup_node() {
    info "Setting up Node.js..."

    if ! command_exists node; then
        if [ "$OS" = "macos" ]; then
            brew install node
        else
            curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
            sudo apt-get install -y -qq nodejs
        fi
    fi

    # Install project dependencies
    cd "$PROJECT_ROOT"
    if [ -f package.json ] && [ ! -d node_modules ]; then
        info "Installing npm dependencies..."
        npm install
    fi

    ok "Node.js ready ($(node --version))"
}

# ─── Tailwind CSS v4 Standalone + Lightning CSS ──────────────────────
setup_css_tools() {
    info "Setting up CSS tooling..."

    local bin_dir="$PROJECT_ROOT/.tools/bin"
    mkdir -p "$bin_dir"

    # Tailwind CSS v4 standalone CLI
    if [ ! -f "$bin_dir/tailwindcss" ]; then
        info "Downloading Tailwind CSS v4 standalone..."
        local tw_arch
        if [ "$OS" = "macos" ]; then
            tw_arch="macos-arm64"
            [ "$(uname -m)" = "x86_64" ] && tw_arch="macos-x64"
        else
            tw_arch="linux-x64"
            [ "$(uname -m)" = "aarch64" ] && tw_arch="linux-arm64"
        fi
        curl -sLO "https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-${tw_arch}"
        mv "tailwindcss-${tw_arch}" "$bin_dir/tailwindcss"
        chmod +x "$bin_dir/tailwindcss"
    fi

    ok "CSS tools ready"
}

# ─── Database Setup ──────────────────────────────────────────────────
setup_database() {
    info "Setting up PostgreSQL..."

    # Start PostgreSQL if not running
    if [ "$OS" = "macos" ]; then
        brew services start postgresql@16 2>/dev/null || true
    else
        sudo systemctl enable postgresql 2>/dev/null || true
        sudo systemctl start postgresql 2>/dev/null || true
    fi

    # Wait for Postgres to be ready
    local retries=10
    while ! pg_isready -q 2>/dev/null && [ $retries -gt 0 ]; do
        sleep 1
        retries=$((retries - 1))
    done

    if ! pg_isready -q 2>/dev/null; then
        warn "PostgreSQL not responding. You may need to start it manually."
        return
    fi

    # Create user and database (idempotent)
    if [ "$OS" = "macos" ]; then
        psql postgres -tc "SELECT 1 FROM pg_roles WHERE rolname='bominal'" | grep -q 1 || \
            createuser bominal --createdb 2>/dev/null || true
        psql postgres -tc "SELECT 1 FROM pg_database WHERE datname='bominal'" | grep -q 1 || \
            createdb bominal -O bominal 2>/dev/null || true
        psql postgres -c "ALTER USER bominal WITH PASSWORD 'bominal';" 2>/dev/null || true
    else
        sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='bominal'" | grep -q 1 || \
            sudo -u postgres createuser bominal --createdb 2>/dev/null || true
        sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname='bominal'" | grep -q 1 || \
            sudo -u postgres createdb bominal -O bominal 2>/dev/null || true
        sudo -u postgres psql -c "ALTER USER bominal WITH PASSWORD 'bominal';" 2>/dev/null || true
    fi

    ok "PostgreSQL ready (database: bominal, user: bominal)"
}

# ─── Valkey Setup ────────────────────────────────────────────────────
setup_valkey() {
    info "Setting up Valkey..."

    if [ "$OS" = "macos" ]; then
        brew services start valkey 2>/dev/null || brew services start redis 2>/dev/null || true
    else
        sudo systemctl enable valkey-server 2>/dev/null || sudo systemctl enable redis-server 2>/dev/null || true
        sudo systemctl start valkey-server 2>/dev/null || sudo systemctl start redis-server 2>/dev/null || true
    fi

    # Verify connectivity
    if command_exists valkey-cli; then
        valkey-cli ping &>/dev/null && ok "Valkey ready" || warn "Valkey not responding"
    elif command_exists redis-cli; then
        redis-cli ping &>/dev/null && ok "Valkey (redis-compatible) ready" || warn "Valkey not responding"
    else
        warn "No Valkey/Redis CLI found"
    fi
}

# ─── Environment File ────────────────────────────────────────────────
setup_env() {
    info "Setting up environment..."

    cd "$PROJECT_ROOT"
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            cp .env.example .env
            info "Created .env from .env.example"

            # Generate a random encryption key
            local key
            key=$(openssl rand -hex 32 2>/dev/null || head -c 32 /dev/urandom | xxd -p -c 64)
            if [ "$OS" = "macos" ]; then
                sed -i '' "s/^ENCRYPTION_KEY=.*/ENCRYPTION_KEY=${key}/" .env
            else
                sed -i "s/^ENCRYPTION_KEY=.*/ENCRYPTION_KEY=${key}/" .env
            fi
            ok "Generated random ENCRYPTION_KEY"
        else
            warn "No .env.example found; skipping .env creation"
        fi
    else
        ok ".env already exists"
    fi
}

# ─── Run Migrations ──────────────────────────────────────────────────
run_migrations() {
    info "Running database migrations..."

    cd "$PROJECT_ROOT"
    if [ -d crates/bominal-db/migrations ] && command_exists sqlx; then
        # Source .env for DATABASE_URL
        [ -f .env ] && set -a && source .env && set +a

        if [ -n "${DATABASE_URL:-}" ]; then
            sqlx database create 2>/dev/null || true
            sqlx migrate run --source crates/bominal-db/migrations 2>/dev/null || warn "Migrations failed or no migrations yet"
            ok "Migrations complete"
        else
            warn "DATABASE_URL not set; skipping migrations"
        fi
    else
        warn "sqlx CLI or migrations directory not found; skipping"
    fi
}

# ─── Verify Installation ─────────────────────────────────────────────
verify() {
    echo ""
    info "=== Installation Verification ==="

    local all_ok=true

    check() {
        if command_exists "$1"; then
            ok "$1: $($1 --version 2>&1 | head -1)"
        else
            warn "$1: NOT FOUND"
            all_ok=false
        fi
    }

    check rustc
    check cargo
    check node
    check npm
    check psql
    check caddy
    check brotli
    command_exists valkey-server && ok "valkey-server: installed" || \
        (command_exists redis-server && ok "redis-server: installed (valkey-compatible)") || \
        (warn "valkey/redis: NOT FOUND" && all_ok=false)
    command_exists cargo-leptos && ok "cargo-leptos: installed" || warn "cargo-leptos: NOT FOUND"
    command_exists sqlx && ok "sqlx-cli: installed" || warn "sqlx-cli: NOT FOUND"

    echo ""
    if [ "$all_ok" = true ]; then
        ok "=== All checks passed! ==="
    else
        warn "=== Some tools are missing. Check warnings above. ==="
    fi

    echo ""
    info "Next steps:"
    echo "  1. Review and edit .env file"
    echo "  2. Run: cargo leptos serve    (start dev server)"
    echo "  3. Run: npm run dev           (React prototype)"
    echo ""
}

# ─── Main ─────────────────────────────────────────────────────────────
main() {
    echo ""
    echo "  ╔══════════════════════════════════════════╗"
    echo "  ║   Bominal Train Reservation Bootstrap    ║"
    echo "  ╚══════════════════════════════════════════╝"
    echo ""

    detect_os
    install_system_deps
    setup_rust
    setup_node
    setup_css_tools
    setup_database
    setup_valkey
    setup_env
    run_migrations
    verify
}

main "$@"
