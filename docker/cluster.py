#!/usr/bin/env python3
"""
Dynamic Citadel cluster launcher.

Usage:
    ./cluster.py up [N]      - Start N nodes (default: 20)
    ./cluster.py down        - Stop the cluster
    ./cluster.py logs [svc]  - View logs (optional: specific service)
    ./cluster.py ps          - List running containers

Environment:
    Reads from .env.docker.dev if present. Copy .env.docker.dev.example to get started.
"""

import os
import subprocess
import sys
from pathlib import Path
import yaml


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

def generate_compose(num_nodes: int) -> dict:
    """Generate docker-compose config for N citadel nodes."""

    # Base citadel node config (used as template)
    citadel_node_base = {
        'image': 'debian:trixie-slim',
        'command': '''bash -c "
            apt-get update && apt-get install -y inotify-tools &&
            while [ ! -f /citadel/target/release/lens-node ]; do sleep 2; done &&
            while true; do
                /citadel/target/release/lens-node &
                PID=$$!
                inotifywait -e close_write /citadel/target/release/lens-node
                kill $$PID 2>/dev/null || true
                sleep 1
            done
        "''',
        'environment': {
            'CITADEL_PEERS': ','.join(f'citadel-{i}:9000' for i in range(1, min(4, num_nodes + 1))),
            'ADMIN_PUBLIC_KEY': '${ADMIN_PUBLIC_KEY:-}',
            'RUST_LOG': '${RUST_LOG:-info}',
        },
        'depends_on': ['citadel-builder'],
        'networks': ['citadel-mesh'],
    }

    # Build services dict
    services = {
        'flagship-dev': {
            'image': 'node:22-alpine',
            'working_dir': '/app',
            'command': 'sh -c "corepack enable && pnpm install && pnpm dev --host 0.0.0.0 --port 5175"',
            'ports': ['${FLAGSHIP_HOST:-0.0.0.0}:${FLAGSHIP_PORT:-9999}:5175'],
            'volumes': ['..:/app', 'flagship_node_modules:/app/node_modules'],
            'environment': [
                'NODE_ENV=development',
                'VITE_API_URL=https://citadel.lon.riff.cc/api/v1',
                'VITE_ALLOWED_HOSTS=flagship.lon.riff.cc',
            ],
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
        'citadel-builder': {
            'image': 'rust:1.83-bookworm',
            'working_dir': '/citadel',
            'command': '''bash -c "
                apt-get update && apt-get install -y git curl libclang-dev build-essential &&
                curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
                cargo binstall -y watchexec-cli
                rustup component add rustfmt
                if [ ! -d /citadel/.git ]; then
                    git clone https://github.com/rifflabs/citadel.git /citadel-tmp &&
                    mv /citadel-tmp/* /citadel-tmp/.* /citadel/ 2>/dev/null || true &&
                    rm -rf /citadel-tmp
                fi &&
                watchexec -r -w crates/citadel-lens/src -- cargo build --release -p citadel-lens &&
                sleep infinity
            "''',
            'volumes': [
                'citadel_src:/citadel',
                'citadel_cargo:/usr/local/cargo/registry',
                'citadel_target:/citadel/target',
            ],
        },
    }

    # Generate citadel nodes
    for i in range(1, num_nodes + 1):
        node = {
            **citadel_node_base,
            'environment': dict(citadel_node_base['environment']),  # Copy
            'volumes': [
                'citadel_src:/citadel:ro',
                'citadel_target:/citadel/target:ro',
                f'citadel_data_{i}:/data',
            ],
        }
        # First node gets exposed port
        if i == 1:
            node['ports'] = ['8080:8080']
        services[f'citadel-{i}'] = node

    # Build volumes dict
    volumes = {
        'flagship_node_modules': None,
        'citadel_src': None,
        'citadel_cargo': None,
        'citadel_target': None,
    }
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
        num_nodes = int(sys.argv[2]) if len(sys.argv) > 2 else 20
        print(f"Starting {num_nodes}-node Citadel cluster...")

        # Generate configs
        compose = generate_compose(num_nodes)
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
