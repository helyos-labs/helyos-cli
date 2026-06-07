<div align="center">

<img alt="Helyos" src="assets/helyos-logo.png" width="200">

# helyos-cli

**Helyos CLI -- deploy and manage containers from the terminal**

[![CI](https://github.com/helyos-labs/helyos-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/helyos-labs/helyos-cli/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.3.0-blue.svg)](Cargo.toml)

`helyos` is the command-line interface for [Helyos](https://github.com/helyos-labs/helyos)
-- simple distributed container orchestration in Rust. It talks to a running
[helyosd](https://github.com/helyos-labs/helyosd) daemon over HTTPS and provides
a complete set of commands for project management, deployments, scaling, secrets,
routing, clustering, and node operations -- plus kubectl-style login and named
connection contexts for working against one or many clusters.

> 80% of Kubernetes use-cases with 20% of the complexity.

[Quick Start](#quick-start) -- [Login & Contexts](#login--contexts) -- [Command Reference](#command-reference) -- [Configuration](#configuration) -- [Related Repositories](#related-repositories)

</div>

---

## Features

- **kubectl-style remote control** -- `helyos login <server>` pins the daemon CA and stores a named, switchable context
- **Secure by default** -- HTTPS transport with CA pinning; bearer-token authentication
- **Interactive project scaffolding** -- `helyos init` generates a deployment YAML with guided prompts
- **Declarative deployments** -- deploy from YAML specs with a single command
- **Full resource management** -- projects, deployments, pods, secrets, routes, certificates, nodes
- **Server-side API tokens** -- mint, list, and revoke tokens with `helyos auth token …`
- **Live log streaming** -- tail container logs in real time
- **Live dashboard** -- `helyos top` for an at-a-glance cluster view
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

Requires Rust 1.85+ (edition 2024).

## Quick Start

When you start `helyosd` locally, it writes a ready-to-use CLI context, so the
CLI just works against the local daemon with no extra setup:

```bash
# 1. Create a project interactively
helyos init

# 2. Or scaffold a project non-interactively, then deploy
#    (init writes the spec to <name>/app.yaml)
helyos init myapp --image nginx:alpine
helyos deploy myapp/app.yaml

# 3. Check status
helyos status
helyos pods
helyos deployments

# 4. Scale up
helyos scale api 5 --project ecommerce

# 5. View logs
helyos logs api --project ecommerce --tail 100
```

To talk to a remote daemon, log in first -- see [Login & Contexts](#login--contexts).

## Login & Contexts

`helyos` connects to daemons over HTTPS using bearer tokens, and remembers each
connection as a named **context** (server URL, token, pinned CA, default project).
This is the kubectl-style workflow: log in once, then switch between clusters.

### Log in to a remote cluster

`helyosd` auto-generates an API token on first run and logs it once. Mint
additional tokens later with [`helyos auth token create`](#authentication). To
connect:

```bash
# Pin the daemon's self-signed CA (trust-on-first-use) and store a context.
# The CLI assumes https:// and port :6443 if you omit them.
helyos login cluster.example.com --token nxa-api_xxxxxxxx

# Non-interactive / CI: read the token from stdin and verify the CA fingerprint
# fail-closed (no prompt), naming the context explicitly:
echo "$HELYOS_API_TOKEN" | helyos login https://cluster.example.com:6443 \
  --token-stdin \
  --ca-fingerprint 9f:86:d0:81:... \
  --name prod --project web
```

On success the token is validated against `GET /api/v1/whoami`, the CA is pinned,
and the context becomes active. Verify with:

```bash
helyos whoami            # show the identity of the active token
helyos context current   # show the active context
```

`helyos login <server>` flags:

| Flag | Description |
|---|---|
| `--token <TOKEN>` | API token (else `--token-stdin`, `$HELYOS_API_TOKEN`, or interactive prompt) |
| `--token-stdin` | Read the API token from stdin (one line) |
| `--ca-file <PEM>` | Trust this CA PEM file instead of fetching it out-of-band |
| `--ca-fingerprint <SHA256>` | Require the fetched CA to match this SHA-256 (fail-closed, no prompt) |
| `--insecure-skip-tls-verify` | Skip TLS verification entirely (insecure) |
| `--name <NAME>` | Context name (default: derived from the host) |
| `--project <NAME>` | Default project for this context |
| `--no-set-current` | Do not switch the active context to this one |

### Switch and manage contexts

```bash
helyos context ls                  # list contexts (alias: list); * marks active
helyos context use prod            # switch the active context
helyos context current             # show the active context
helyos context rename prod prod-eu # rename a context
helyos context set prod --server https://new-host:6443 --project web
helyos context rm prod             # remove a context

# Override the active context for a single command:
helyos --context staging status
```

### Log out

```bash
# Drop the token from a context (keeps the server URL + pinned CA so you can
# re-login with a single paste). Defaults to the active context.
helyos logout
helyos logout prod
```

## Command Reference

### Core Workflow

```
helyos init [NAME] [--image IMAGE]          Create a new project (interactive if no args)
helyos deploy <FILE> [--timeout SECS]       Deploy a service from a YAML spec
helyos status                               Show cluster overview
helyos top                                  Live cluster dashboard
helyos pods [-p PROJECT]                    List all pods
helyos deployments [-p PROJECT]             List all deployments
helyos logs <NAME> [-p PROJECT] [--tail N]  Stream container logs
helyos scale <NAME> <REPLICAS> [-p PROJECT] Scale a deployment
helyos stop <NAME> [-p PROJECT]             Stop a deployment
helyos rm <NAME> [-p PROJECT] [-y]          Remove a deployment
```

### Login & Contexts

```
helyos login <SERVER> [flags]               Pin the daemon CA and store a context
helyos logout [NAME]                         Drop the token from a context (active by default)
helyos whoami                                Show the identity of the active token
helyos context ls                            List contexts (alias: list)
helyos context use <NAME>                    Switch the active context
helyos context current                       Show the active context
helyos context rename <OLD> <NEW>            Rename a context
helyos context set <NAME> [--server URL] [--project NAME]  Edit a context
helyos context rm <NAME>                     Remove a context
```

### Authentication

Server-side API tokens (managed via the active context's connection):

```
helyos auth token create <NAME> [--ttl SECS]  Create a token (secret shown once)
helyos auth token ls                          List API tokens (alias: list)
helyos auth token revoke <NAME>               Revoke an API token by name
```

### Project Management

```
helyos project list                     List all projects
helyos project create <NAME>            Create a project
helyos project suspend <NAME>           Suspend a project (stops all deployments)
helyos project resume <NAME>            Resume a suspended project
helyos project delete <NAME> [-y]       Delete a project and all its resources
```

### Secrets

```
helyos secret set <NAME> [--value VALUE] -p PROJECT   Store an encrypted secret (prompts/stdin if no value)
helyos secret list -p PROJECT                         List secret names
helyos secret rm <NAME> -p PROJECT                    Delete a secret
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
helyos completions <SHELL>            Generate shell completions
```

### Global Flags

```
--server <URL>     helyosd server address (or HELYOS_SERVER, or active context;
                   defaults to http://localhost:6443)
--token <TOKEN>    API bearer token (or HELYOS_API_TOKEN, or active context)
--context <NAME>   Use a specific context for this command (overrides current-context)
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

restart: always  # always | onfailure | never

resources:
  cpu: 0.5
  memory: 256M
```

## Configuration

Connection settings are resolved with the precedence: **CLI flag > environment
variable > config file > built-in default**. The built-in default server is
`http://localhost:6443`, but in practice the active context supplies an HTTPS
endpoint -- `helyosd` writes a local context on first start, and `helyos login`
adds remote ones (both on the standard daemon port `6443`).

```bash
# Per-command override
helyos --server https://10.0.1.1:6443 --token "$TOKEN" status

# Or via environment variables
export HELYOS_SERVER=https://10.0.1.1:6443
export HELYOS_API_TOKEN=nxa-api_xxxxxxxx
helyos status
```

### Config file

Contexts live in `~/.helyos/config.toml` (override the path with `HELYOS_CONFIG`).
The CLI manages this file for you via `helyos login` and `helyos context …`, but
it is human-readable. It uses a small TOML subset: a top-level `current-context`
key plus one `[context.NAME]` section per target.

```toml
# Managed by `helyos`. Edit with `helyos context` / `helyos login`.
current-context = "prod"

[context.local]
server = "http://localhost:6443"
token = "nxa-api_local"
project = "default"

[context.prod]
server = "https://cluster.example.com:6443"
token = "nxa-api_xxxxxxxx"
token-name = "alice"
project = "web"
ca = "<base64-encoded pinned CA PEM>"
ca-sha256 = "9f:86:d0:81:..."
```

Per-context keys: `server` (required), `token`, `token-name`, `project`,
`insecure` (`true`/`false`), `ca` (base64-encoded pinned CA), and `ca-sha256`
(the CA fingerprint). The active context is `current-context` when set; when a
single context exists it is used implicitly. A legacy flat file with only
top-level `server`/`token` is read as a context named `default`.

The file is written atomically with `0600` permissions, and the previous version
is backed up to `config.toml.bak` on each save.

## Development

```bash
# Build
cargo build

# Run tests (58 tests)
cargo test

# Build release binary
cargo build --release
```

CI enforces formatting -- run `cargo fmt` before pushing.

## Related Repositories

| Repository | Description |
|---|---|
| [helyos](https://github.com/helyos-labs/helyos) | Meta repository -- overview, architecture, and docs |
| [helyos-core](https://github.com/helyos-labs/helyos-core) | Core domain types, traits, and orchestrator |
| [helyosd](https://github.com/helyos-labs/helyosd) | Daemon -- container runtime, state, REST API, clustering |

## License

Apache-2.0 -- see [LICENSE](LICENSE) for details.
