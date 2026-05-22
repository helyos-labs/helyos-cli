<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/white_logo.png" width="200">
  <source media="(prefers-color-scheme: light)" srcset="assets/black_logo.png" width="200">
  <img alt="NexaNet" src="assets/black_logo.png" width="200">
</picture>

# nexa-cli

**NexaNet CLI -- deploy and manage containers from the terminal**

[![CI](https://github.com/nexa-net/nexa-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/nexa-net/nexa-cli/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

nexa-cli is the command-line interface for NexaNet. It talks to a running
[nexad](https://github.com/nexa-net/nexad) instance over HTTP and provides
a complete set of commands for project management, deployments, scaling,
secrets, routing, clustering, and node operations.

</div>

---

## Features

- **Interactive project scaffolding** -- `nexa init` generates a deployment YAML with guided prompts
- **Declarative deployments** -- deploy from YAML specs with a single command
- **Full resource management** -- projects, deployments, pods, secrets, routes, certificates, nodes
- **Live log streaming** -- tail container logs in real time
- **Cluster operations** -- initialize clusters, manage join tokens, drain and remove nodes
- **JSON output mode** -- `--json` flag for scripting and CI/CD pipelines
- **Styled terminal output** -- colored tables, spinners, and human-friendly time formatting
- **CNI plugin installer** -- `nexa setup cni` downloads and installs standard CNI binaries

## Installation

```bash
# Build from source
cargo install --git https://github.com/nexa-net/nexa-cli

# Or clone and build
git clone https://github.com/nexa-net/nexa-cli.git
cd nexa-cli
cargo build --release
# Binary is at ./target/release/nexa
```

## Quick Start

```bash
# 1. Create a project interactively
nexa init

# 2. Or create a project and deploy in one step
nexa init myapp --image nginx:alpine
nexa deploy myapp.yaml

# 3. Check status
nexa status
nexa pods
nexa deployments

# 4. Scale up
nexa scale api 5 --project ecommerce

# 5. View logs
nexa logs api --project ecommerce --tail 100
```

## Command Reference

### Core Workflow

```
nexa init [NAME] [--image IMAGE]      Create a new project (interactive if no args)
nexa deploy <FILE>                    Deploy a service from a YAML spec
nexa status                           Show cluster overview
nexa pods [-p PROJECT]                List all pods
nexa deployments [-p PROJECT]         List all deployments
nexa logs <NAME> [-p PROJECT] [--tail N]  Stream container logs
nexa scale <NAME> <REPLICAS> [-p PROJECT] Scale a deployment
nexa stop <NAME> [-p PROJECT]         Stop a deployment
nexa rm <NAME> [-p PROJECT]           Remove a deployment
```

### Project Management

```
nexa project list                     List all projects
nexa project create <NAME>            Create a project
nexa project suspend <NAME>           Suspend a project (stops all deployments)
nexa project resume <NAME>            Resume a suspended project
nexa project delete <NAME>            Delete a project and all its resources
```

### Secrets

```
nexa secret set <NAME> <VALUE> -p PROJECT    Store an encrypted secret
nexa secret list -p PROJECT                  List secret names
nexa secret rm <NAME> -p PROJECT             Delete a secret
```

### Routing and TLS

```
nexa routes [-p PROJECT]              List all routes
nexa route add <DOMAIN> -p PROJECT --deployment NAME [--https]
                                      Add a route for a domain
nexa route rm <DOMAIN>                Remove a route
nexa cert import <DOMAIN> --cert FILE --key FILE
                                      Import a TLS certificate
```

### Cluster and Nodes

```
nexa cluster init                     Initialize the cluster
nexa cluster token show               Show the join token
nexa cluster token rotate             Rotate the join token
nexa cluster config get-scheduler     View scheduler configuration
nexa cluster config set <KEY> <VALUE> Set scheduler strategy or weights
nexa nodes                            List cluster nodes
nexa node drain <NAME>                Drain a node (stop scheduling)
nexa node rm <NAME>                   Remove a node from the cluster
```

### System Setup

```
nexa setup cni [--bin-dir DIR] [--version VER]
                                      Download and install CNI plugins
```

### Global Flags

```
--server <URL>     nexad server address [default: http://localhost:6443]
--json             Output results as JSON (for scripting)
--help             Show help
--version          Show version
```

## Deployment Spec Format

`nexa deploy` accepts a YAML file describing the desired state:

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

nexa-cli connects to nexad at `http://localhost:6443` by default. Override with:

```bash
# Per-command
nexa --server http://10.0.1.1:6443 status

# Or export the environment variable
export NEXA_SERVER=http://10.0.1.1:6443
nexa status
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
| [nexa-core](https://github.com/nexa-net/nexa-core) | Core domain types, traits, and orchestrator |
| [nexad](https://github.com/nexa-net/nexad) | Daemon -- container runtime, state, API, clustering |
| [nexa-proxy](https://github.com/nexa-net/nexa-proxy) | Lightweight reverse proxy with weighted load balancing |

## License

Apache-2.0 -- see [LICENSE](LICENSE) for details.
