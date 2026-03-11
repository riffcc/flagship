#!/usr/bin/env python3
"""
Riff.CC development stack launcher.

Starts Flagship frontend with hot-reload, backed by a Citadel mesh cluster.

Usage:
    ./riffstack.py up [N]            Start stack with N Citadel nodes (default: 5)
    ./riffstack.py down              Stop everything
    ./riffstack.py logs [service]    View logs
    ./riffstack.py ps                List containers
    ./riffstack.py restart           Restart frontend only

Options:
    --admin-keys=KEY1,KEY2,...  Comma-separated list of admin public keys
                                (ed25519p/... format)

Services:
    flagship-dev    Frontend (Vite) on http://localhost:9999
    citadel-lb      API load balancer on http://localhost:8085
    citadel-1..N    Mesh nodes

Examples:
    ./riffstack.py up 5              # Start with 5 Citadel nodes
    ./riffstack.py logs flagship-dev # View frontend logs
    ./riffstack.py restart           # Restart frontend after config change
    ./riffstack.py up 5 --admin-keys=ed25519p/abc123,ed25519p/def456
"""

import os
import subprocess
import sys
from pathlib import Path

import yaml


SCRIPT_DIR = Path(__file__).parent
CITADEL_DIR = Path.home() / 'projects' / 'citadel' / 'docker'


def load_env():
    """Load environment from .env.docker.dev if it exists."""
    env_file = SCRIPT_DIR / '.env.docker.dev'
    if env_file.exists():
        with open(env_file) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    os.environ.setdefault(key.strip(), value.strip())


def generate_compose() -> dict:
    """Generate docker-compose config for Flagship frontend."""
    home = os.path.expanduser('~')
    flagship_src = f'{home}/projects/flagship'

    return {
        'services': {
            'flagship-dev': {
                'build': {
                    'context': '.',
                    'dockerfile': 'Dockerfile.dev',
                },
                'container_name': 'flagship-dev',
                'working_dir': '/app',
                'command': 'sh -c "pnpm install || true; pnpm dev --host 0.0.0.0 --port 5175"',
                'ports': ['${FLAGSHIP_HOST:-0.0.0.0}:${FLAGSHIP_PORT:-9999}:5175'],
                'volumes': [
                    f'{flagship_src}:/app',
                    'flagship_node_modules:/app/node_modules',
                ],
                'env_file': ['.env.docker.dev'],
                'environment': [
                    'NODE_ENV=development',
                    'WEB=true',
                    'VITE_API_URL=${VITE_API_URL:-http://localhost:8085/api/v1}',
                    'VITE_ALLOWED_HOSTS=*',
                ],
                'ulimits': {
                    'nofile': {
                        'soft': 1048576,
                        'hard': 1048576,
                    },
                },
                'networks': ['citadel-mesh'],
                'restart': 'unless-stopped',
            },
        },
        'networks': {
            'citadel-mesh': {
                'external': True,
                'name': 'docker_citadel-mesh',
            },
        },
        'volumes': {
            'flagship_node_modules': None,
        },
    }


def citadel_cmd(args: list[str]):
    """Run a citadel.py command."""
    citadel_py = CITADEL_DIR / 'citadel.py'
    if not citadel_py.exists():
        print(f"Error: {citadel_py} not found")
        print("Make sure ~/projects/citadel/docker/citadel.py exists")
        sys.exit(1)

    subprocess.run(['python3', str(citadel_py)] + args, cwd=CITADEL_DIR, check=True)


def cmd_up(args: list[str]):
    """Start the full stack."""
    # Extract options
    docker_rust_build = '--docker-rust-build' in args
    admin_keys = None
    filtered_args = []

    i = 0
    while i < len(args):
        a = args[i]
        if a == '--docker-rust-build':
            pass  # Already captured above
        elif a.startswith('--admin-keys='):
            admin_keys = a.split('=', 1)[1]
        elif a == '--admin-keys' and i + 1 < len(args):
            admin_keys = args[i + 1]
            i += 1  # Skip the next arg (the value)
        elif not a.startswith('--'):
            filtered_args.append(a)
        i += 1
    args = filtered_args
    num_nodes = int(args[0]) if args else 5

    # Set ADMIN_PUBLIC_KEY env var if provided
    if admin_keys:
        os.environ['ADMIN_PUBLIC_KEY'] = admin_keys
        print(f"Admin keys: {admin_keys[:50]}..." if len(admin_keys) > 50 else f"Admin keys: {admin_keys}")

    print("=" * 60)
    print("Starting Riff.CC Development Stack")
    print("=" * 60)
    print()

    # Start Citadel cluster first
    print("[1/2] Starting Citadel cluster...")
    citadel_args = ['up', str(num_nodes)]
    if docker_rust_build:
        citadel_args.append('--docker-rust-build')
    citadel_cmd(citadel_args)
    print()

    # Check if Flagship is already running (hot-reloads, no need to rebuild)
    # Use exact match with ^...$ to avoid matching flagship-devpreview
    result = subprocess.run(
        ['docker', 'ps', '-q', '-f', 'name=^flagship-dev$', '-f', 'status=running'],
        capture_output=True, text=True
    )
    flagship_running = bool(result.stdout.strip())

    if flagship_running:
        print("[2/2] Flagship frontend already running (hot-reload enabled)")
    else:
        print("[2/2] Starting Flagship frontend...")
        compose = generate_compose()
        compose_file = SCRIPT_DIR / 'docker-compose.flagship.yml'

        with open(compose_file, 'w') as f:
            yaml.dump(compose, f, default_flow_style=False, sort_keys=False)

        subprocess.run([
            'docker', 'compose', '-f', str(compose_file),
            'up', '-d', '--build'
        ], check=True)

    print()
    print("=" * 60)
    print("Stack is running!")
    print("=" * 60)
    print()
    print("  Flagship:       http://localhost:9999")
    print("  Citadel API:    http://localhost:8085")
    print("  HAProxy stats:  http://localhost:8404/stats")
    print()
    print("Commands:")
    print("  ./riffstack.py logs              # All logs")
    print("  ./riffstack.py logs flagship-dev # Frontend logs")
    print("  ./riffstack.py ps                # List containers")
    print("  ./riffstack.py restart           # Restart frontend")
    print("  ./riffstack.py down              # Stop everything")


def cmd_down():
    """Stop everything."""
    print("Stopping Flagship...")
    compose_file = SCRIPT_DIR / 'docker-compose.flagship.yml'
    if compose_file.exists():
        subprocess.run([
            'docker', 'compose', '-f', str(compose_file), 'down'
        ])

    print("Stopping Citadel...")
    citadel_cmd(['down'])


def cmd_logs(args: list[str]):
    """View logs."""
    service = args[0] if args else None

    # If requesting flagship-dev, use flagship compose
    if service == 'flagship-dev':
        compose_file = SCRIPT_DIR / 'docker-compose.flagship.yml'
        subprocess.run([
            'docker', 'compose', '-f', str(compose_file), 'logs', '-f', 'flagship-dev'
        ])
    elif service:
        # Specific citadel service
        citadel_cmd(['logs', service])
    else:
        # All logs - run both in parallel
        import threading

        def citadel_logs():
            citadel_cmd(['logs'])

        t = threading.Thread(target=citadel_logs, daemon=True)
        t.start()

        compose_file = SCRIPT_DIR / 'docker-compose.flagship.yml'
        subprocess.run([
            'docker', 'compose', '-f', str(compose_file), 'logs', '-f'
        ])


def cmd_ps():
    """List all containers."""
    print("=== Flagship ===")
    compose_file = SCRIPT_DIR / 'docker-compose.flagship.yml'
    if compose_file.exists():
        subprocess.run([
            'docker', 'compose', '-f', str(compose_file), 'ps'
        ])
    print()
    print("=== Citadel ===")
    citadel_cmd(['ps'])


def cmd_restart():
    """Restart frontend only."""
    compose_file = SCRIPT_DIR / 'docker-compose.flagship.yml'
    subprocess.run([
        'docker', 'compose', '-f', str(compose_file), 'restart'
    ])


def main():
    load_env()

    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    cmd = sys.argv[1]
    args = sys.argv[2:]

    commands = {
        'up': lambda: cmd_up(args),
        'down': cmd_down,
        'logs': lambda: cmd_logs(args),
        'ps': cmd_ps,
        'restart': cmd_restart,
    }

    if cmd in commands:
        commands[cmd]()
    else:
        print(f"Unknown command: {cmd}")
        print(__doc__)
        sys.exit(1)


if __name__ == '__main__':
    main()
