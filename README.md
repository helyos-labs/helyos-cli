<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/white_logo.png" width="200">
  <source media="(prefers-color-scheme: light)" srcset="assets/black_logo.png" width="200">
  <img alt="Helyos" src="assets/black_logo.png" width="200">
</picture>

# helyos-cli

**Helyos CLI -- deploy and manage containers from the terminal**

[![CI](https://github.com/helyos-labs/helyos-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/helyos-labs/helyos-cli/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

helyos-cli is the command-line interface for Helyos. It talks to a running
[helyosd](https://github.com/helyos-labs/helyosd) instance over HTTP and provides
a complete set of commands for project management, deployments, scaling,
secrets, routing, clustering, and node operations.

</div>

---

## Features

- **Interactive project scaffolding** -- `helyos init` generates a deployment YAML with guided prompts
- **Declarative deployments** -- deploy from YAML specs with a single command
- **Full resource management** -- projects, deployments, pods, secrets, routes, certificates, nodes
- **Live log streaming** -- tail container logs in real time
- **Cluster operations** -- initialize clusters, manage join tokens, drain and remove nodes
- **JSON output mode** -- `--json` flag for scripting and CI/CD pipelines
- **Styled terminal output** -- colored tables, spinners, and human-friendly time formatting
- **CNI plugin installer** -- `helyos setup cni` downloads and installs standard CNI binaries

## Installation

```bash
# Build from source
cargo install --git https://github.com/helyos-labs/helyos-cli

# Or clone and build
git clone https://github.com/helyos-labs/helyos-cli.git
cd helyos-cli
cargo build --release
# Binary is at ./target/release/helyos
```

## Quick Start

```bash
# 1. Create a project interactively
helyos init

# 2. Or create a project and deploy in one step
helyos init myapp --image nginx:alpine
helyos deploy myapp.yaml

# 3. Check status
helyos status
helyos pods
helyos deployments

# 4. Scale up
helyos scale api 5 --project ecommerce

# 5. View logs
helyos logs api --project ecommerce --tail 100
```

## Command Reference

### Core Workflow

```
helyos init [NAME] [--image IMAGE]      Create a new project (interactive if no args)
helyos deploy <FILE>                    Deploy a service from a YAML spec
helyos status                           Show cluster overview
helyos pods [-p PROJECT]                List all pods
helyos deployments [-p PROJECT]         List all deployments
helyos logs <NAME> [-p PROJECT] [--tail N]  Stream container logs
helyos scale <NAME> <REPLICAS> [-p PROJECT] Scale a deployment
helyos stop <NAME> [-p PROJECT]         Stop a deployment
helyos rm <NAME> [-p PROJECT]           Remove a deployment
```

### Project Management

```
helyos project list                     List all projects
helyos project create <NAME>            Create a project
helyos project suspend <NAME>           Suspend a project (stops all deployments)
helyos project resume <NAME>            Resume a suspended project
helyos project delete <NAME>            Delete a project and all its resources
```

### Secrets

```
helyos secret set <NAME> <VALUE> -p PROJECT    Store an encrypted secret
helyos secret list -p PROJECT                  List secret names
helyos secret rm <NAME> -p PROJECT             Delete a secret
```

### Routing and TLS

```
helyos routes [-p PROJECT]              List all routes
helyos route add <DOMAIN> -p PROJECT --deployment NAME [--https]
                                      Add a route for a domain
helyos route rm <DOMAIN>                Remove a route
helyos cert import <DOMAIN> --cert FILE --key FILE
                                      Import a TLS certificate
```

### Cluster and Nodes

```
helyos cluster init                     Initialize the cluster
helyos cluster token show               Show the join token
helyos cluster token rotate             Rotate the join token
helyos cluster config get-scheduler     View scheduler configuration
helyos cluster config set <KEY> <VALUE> Set scheduler strategy or weights
helyos nodes                            List cluster nodes
helyos node drain <NAME>                Drain a node (stop scheduling)
helyos node rm <NAME>                   Remove a node from the cluster
```

### System Setup

```
helyos setup cni [--bin-dir DIR] [--version VER]
                                      Download and install CNI plugins
```

### Global Flags

```
--server <URL>     helyosd server address [default: http://localhost:6443]
--json             Output results as JSON (for scripting)
--help             Show help
--version          Show version
```

## Deployment Spec Format

`helyos deploy` accepts a YAML file describing the desired state:

```yaml
project: ecommerce

deployment:
  name: api

replicas: 3
image: ghcr.io/company/api:latest

ports:
  - 3000

env:
  DATABASE_URL: "postgres://localhost/ecommerce"
  REDIS_URL: "redis://localhost:6379"

network:
  public: true
  domain: api.example.com
  https: true

healthcheck:
  path: /health
  interval: 10s
  timeout: 5s
  retries: 3

volumes:
  - name: data
    mount: /app/data

restart: always  # always | on_failure | never

resources:
  cpu: 0.5
  memory: 256M
```

## Configuration

helyos-cli connects to helyosd at `http://localhost:6443` by default. Override with:

```bash
# Per-command
helyos --server http://10.0.1.1:6443 status

# Or export the environment variable
export HELYOS_SERVER=http://10.0.1.1:6443
helyos status
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Build release binary
cargo build --release
```

## Related Repositories

| Repository | Description |
|---|---|
| [helyos-core](https://github.com/helyos-labs/helyos-core) | Core domain types, traits, and orchestrator |
| [helyosd](https://github.com/helyos-labs/helyosd) | Daemon -- container runtime, state, API, clustering |
| [helyos-proxy](https://github.com/helyos-labs/helyos-proxy) | Lightweight reverse proxy with weighted load balancing |

## License

Apache-2.0 -- see [LICENSE](LICENSE) for details.
