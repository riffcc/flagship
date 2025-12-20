#!/usr/bin/env python3
"""
Dynamic Citadel cluster launcher.

Usage:
    ./cluster.py up [N] [--docker-rust-build]  - Start N nodes (default: 5)
    ./cluster.py down                           - Stop the cluster
    ./cluster.py logs [svc]                     - View logs (optional: specific service)
    ./cluster.py ps                             - List running containers

Options:
    --docker-rust-build   Build Rust in Docker instead of cross-compiling on host.
                          Slower but doesn't require cross-compile toolchain.

Default: Cross-compile on host (auto-detects Apple Silicon vs Linux)

Environment:
    Reads from .env.docker.dev if present. Copy .env.docker.dev.example to get started.
"""

import os
import platform
import subprocess
import sys
from pathlib import Path
import yaml


def get_cross_compile_target() -> str | None:
    """
    Detect if cross-compilation is needed based on host platform.
    Returns the target triple if cross-compile needed, None for native build.

    - Apple Silicon (macOS ARM64): needs cross-compile to x86_64-unknown-linux-gnu
    - Linux x86_64: native build (no cross-compile)
    - Linux ARM64: native build (no cross-compile)
    """
    system = platform.system()
    machine = platform.machine()

    if system == 'Darwin' and machine == 'arm64':
        # Apple Silicon - cross-compile to Linux x86_64 for Docker
        return 'x86_64-unknown-linux-gnu'
    elif system == 'Darwin' and machine == 'x86_64':
        # Intel Mac - cross-compile to Linux x86_64
        return 'x86_64-unknown-linux-gnu'
    else:
        # Linux or other - native build
        return None


def get_binary_path() -> str:
    """Get the path to the lens-node binary based on cross-compile target."""
    target = get_cross_compile_target()
    if target:
        return f'/citadel/target/{target}/release/lens-node'
    else:
        return '/citadel/target/release/lens-node'


def load_env():
    """Load environment from .env.docker.dev if it exists."""
    env_file = Path(__file__).parent / '.env.docker.dev'
    if env_file.exists():
        with open(env_file) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    os.environ.setdefault(key.strip(), value.strip())

def ensure_citadel_node_image():
    """Build the citadel-node base image if it doesn't exist."""
    result = subprocess.run(
        ['docker', 'images', '-q', 'citadel-node:latest'],
        capture_output=True, text=True
    )
    if not result.stdout.strip():
        print("Building citadel-node base image...")
        subprocess.run([
            'docker', 'build', '-t', 'citadel-node:latest',
            '-f', 'Dockerfile.citadel-node', '.'
        ], check=True)


def get_peers_for_node(node_num: int) -> str:
    """
    Get CITADEL_PEERS for a specific node.

    Each node connects to up to 3 predecessors to distribute connection load.
    This prevents bottlenecking on the first few nodes.

    Examples:
      Node 1: "" (genesis, no peers)
      Node 2: "citadel-1:9000"
      Node 3: "citadel-1:9000,citadel-2:9000"
      Node 4: "citadel-1:9000,citadel-2:9000,citadel-3:9000"
      Node 5: "citadel-2:9000,citadel-3:9000,citadel-4:9000"
      Node 6: "citadel-3:9000,citadel-4:9000,citadel-5:9000"
    """
    if node_num <= 1:
        return ""  # Genesis node has no peers

    # Connect to up to 3 predecessors
    # Start from max(1, node_num - 3) to get the 3 nodes before this one
    start = max(1, node_num - 3)
    end = node_num  # exclusive (don't connect to self)

    peers = [f'citadel-{i}:9000' for i in range(start, end)]
    return ','.join(peers)


def generate_compose(num_nodes: int, docker_rust_build: bool = True) -> dict:
    """Generate docker-compose config for N citadel nodes."""

    # Get binary path (auto-detects cross-compile for macOS)
    binary_path = get_binary_path()

    # Base citadel node config (used as template)
    # Uses pre-built image with inotify-tools to avoid apt-get at runtime
    # Note: CITADEL_PEERS is set per-node in the loop below
    citadel_node_base = {
        'image': 'citadel-node:latest',
        'command': f'''bash -c "
            while [ ! -f {binary_path} ]; do sleep 2; done &&
            while true; do
                {binary_path} &
                PID=$$!
                inotifywait -e close_write {binary_path}
                kill $$PID 2>/dev/null || true
                sleep 1
            done
        "''',
        'environment': {
            'ADMIN_PUBLIC_KEY': '${ADMIN_PUBLIC_KEY:-}',
            'RUST_LOG': '${RUST_LOG:-info}',
        },
        'networks': ['citadel-mesh'],
    }
    if docker_rust_build:
        citadel_node_base['depends_on'] = ['citadel-builder']

    # Resolve home directory for bind mounts
    home = os.path.expanduser('~')
    flagship_src = f'{home}/projects/flagship'
    citadel_src = f'{home}/projects/citadel'

    # Build services dict
    services = {
        'flagship-dev': {
            'build': {
                'context': '.',
                'dockerfile': 'Dockerfile.dev',
            },
            'working_dir': '/app',
            'command': 'sh -c "pnpm install || true; pnpm dev --host 0.0.0.0 --port 5175"',
            'ports': ['${FLAGSHIP_HOST:-0.0.0.0}:${FLAGSHIP_PORT:-9999}:5175'],
            'volumes': [f'{flagship_src}:/app', 'flagship_node_modules:/app/node_modules'],
            'env_file': ['.env'],
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
            'depends_on': ['citadel-lb'],
            'networks': ['citadel-mesh'],
        },
        'citadel-lb': {
            'image': 'haproxy:2.9-alpine',
            'ports': [
                '${CITADEL_API_HOST:-0.0.0.0}:${CITADEL_API_PORT:-8085}:8085',
                '${HAPROXY_STATS_HOST:-0.0.0.0}:${HAPROXY_STATS_PORT:-8404}:8404',
            ],
            'volumes': ['./haproxy-dynamic.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro'],
            'depends_on': [f'citadel-{i}' for i in range(1, num_nodes + 1)],
            'networks': ['citadel-mesh'],
        },
    }

    # Add citadel-builder only if docker_rust_build is enabled
    if docker_rust_build:
        services['citadel-builder'] = {
            'image': 'rust:1.92-slim-trixie',
            'working_dir': '/citadel',
            'environment': {'CARGO_BUILD_JOBS': '2'},
            'command': '''bash -c "
                apt-get update && apt-get install -y git curl libclang-dev build-essential &&
                curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
                cargo binstall -y watchexec-cli
                rustup component add rustfmt
                watchexec -r -w crates/citadel-lens/src -- cargo build --release -p citadel-lens &&
                sleep infinity
            "''',
            'volumes': [
                f'{citadel_src}:/citadel',
                'citadel_cargo:/usr/local/cargo/registry',
                'citadel_target:/citadel/target',
            ],
        }

    # Generate citadel nodes
    for i in range(1, num_nodes + 1):
        # Use docker volume for target when building in docker, host path otherwise
        if docker_rust_build:
            target_volume = 'citadel_target:/citadel/target:ro'
        else:
            target_volume = f'{citadel_src}/target:/citadel/target:ro'

        # Each node connects to its 3 predecessors (distributes load)
        node_peers = get_peers_for_node(i)

        node = {
            **citadel_node_base,
            'environment': {
                **citadel_node_base['environment'],
                'CITADEL_PEERS': node_peers,
            },
            'volumes': [
                f'{citadel_src}:/citadel:ro',
                target_volume,
                f'citadel_data_{i}:/data',
            ],
        }
        # First node gets exposed port
        if i == 1:
            node['ports'] = ['8080:8080']
        services[f'citadel-{i}'] = node

    # Build volumes dict (bind mounts for src, volumes for caches)
    volumes = {
        'flagship_node_modules': None,
    }
    if docker_rust_build:
        volumes['citadel_cargo'] = None
        volumes['citadel_target'] = None
    for i in range(1, num_nodes + 1):
        volumes[f'citadel_data_{i}'] = None

    return {
        'services': services,
        'networks': {'citadel-mesh': {'driver': 'bridge'}},
        'volumes': volumes,
    }


def generate_haproxy_config(num_nodes: int) -> str:
    """Generate HAProxy config for N nodes."""
    servers = '\n'.join(
        f'    server citadel-{i} citadel-{i}:8080 check'
        for i in range(1, num_nodes + 1)
    )

    return f"""global
    maxconn 4096

defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    option httplog

frontend http_front
    bind *:8085
    default_backend citadel_nodes

backend citadel_nodes
    balance roundrobin
    option httpchk GET /health
{servers}

frontend stats
    bind *:8404
    stats enable
    stats uri /stats
    stats refresh 10s
"""


def main():
    load_env()

    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == 'up':
        # Parse arguments
        args = sys.argv[2:]
        docker_rust_build = '--docker-rust-build' in args
        args = [a for a in args if not a.startswith('--')]
        num_nodes = int(args[0]) if args else 5

        if docker_rust_build:
            print(f"Starting {num_nodes}-node Citadel cluster (Rust builds in Docker)...")
        else:
            target = get_cross_compile_target()
            if target:
                print(f"Starting {num_nodes}-node Citadel cluster (cross-compile to {target})...")
                print(f"  Build: cd ~/projects/citadel && cross build --release -p citadel-lens --target {target}")
            else:
                print(f"Starting {num_nodes}-node Citadel cluster (native build)...")
                print("  Build: cd ~/projects/citadel && cargo build --release -p citadel-lens")

        # Ensure base image exists (builds once, reused by all nodes)
        ensure_citadel_node_image()

        # Generate configs
        compose = generate_compose(num_nodes, docker_rust_build=docker_rust_build)
        haproxy = generate_haproxy_config(num_nodes)

        # Write configs
        with open('docker-compose.generated.yml', 'w') as f:
            yaml.dump(compose, f, default_flow_style=False, sort_keys=False)

        with open('haproxy-dynamic.cfg', 'w') as f:
            f.write(haproxy)

        # Start cluster
        subprocess.run([
            'docker', 'compose', '-f', 'docker-compose.generated.yml',
            'up', '-d', '--build', '--remove-orphans'
        ])

    elif cmd == 'down':
        subprocess.run([
            'docker', 'compose', '-f', 'docker-compose.generated.yml',
            'down'
        ])

    elif cmd == 'logs':
        args = ['docker', 'compose', '-f', 'docker-compose.generated.yml', 'logs', '-f']
        if len(sys.argv) > 2:
            args.append(sys.argv[2])
        subprocess.run(args)

    elif cmd == 'ps':
        subprocess.run([
            'docker', 'compose', '-f', 'docker-compose.generated.yml', 'ps'
        ])

    else:
        print(f"Unknown command: {cmd}")
        print(__doc__)
        sys.exit(1)


if __name__ == '__main__':
    main()
