# eureka-cli

[![CI](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/loprx/eureka-cli)](https://github.com/loprx/eureka-cli/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

English | [简体中文](README.zh-CN.md)

A `kubectl` for Netflix / Spring Cloud Eureka. Single static binary, no JVM, no runtime — for environments where you can ship a binary but not expose ports or open a browser.

![demo](assets/demo.gif)

## What's new in v0.2

`eureka-cli` v0.2 reframes the project as **the kubectl of Eureka**: it reuses the muscle memory ops people already have from `kubectl`, `docker`, and `helm`, and applies it to "I have hundreds of instances, I just need to find the broken one fast."

- **`-l/--selector`** — filter by any field, including nested metadata: `-l status=UP,metadata.version=v2`
- **`-o wide`** — extra columns (APP / VIP / METADATA on instances; UP/DOWN counts on apps)
- **`--sort-by`** — sort by status / ip / any field path
- **`unhealthy`** — shortcut for `-l 'status!=UP'` on both apps and instances
- **`describe`** — kubectl-style multi-section view (Identity / Status / Network / Lease / DataCenter / Metadata / Timestamps)
- **`-o jsonpath=...`** — pipe-friendly: `eureka-cli -o 'jsonpath=$.instances[*].ipAddr' instances ls | xargs ...`
- **`-w/--watch`** — live refresh, kubectl `-w` semantics, Ctrl+C to exit
- **kubectl-style table** — no borders, space-aligned columns, more rows visible per screen
- **`completion`** — `eureka-cli completion {bash,zsh,fish,powershell}`
- **`config`** — `kubeconfig`-style alias of the existing `servers` command (which becomes a deprecated alias kept until v0.4)

## Scenarios

Designed for places where the Eureka Dashboard is unreachable:

- No public network access (air-gapped / internal-only hosts)
- Cannot expose Eureka Dashboard port externally
- SSH-only access to target machines
- Operations inside K8s Pods (no browser, no port-forward)
- Bastion / jump-host environments

## Install

### Homebrew (macOS / Linux)

```bash
brew install loprx/tap/eureka-cli
```

### Pre-built binaries

Pick from [releases](https://github.com/loprx/eureka-cli/releases). The Linux ones are static-pie musl, run on any glibc 2.17+ or musl distro.

```bash
# Linux x86_64 (works on CentOS 7 → Ubuntu 24)
curl -L -o eureka-cli https://github.com/loprx/eureka-cli/releases/latest/download/eureka-cli-linux-amd64
chmod +x eureka-cli && sudo mv eureka-cli /usr/local/bin/

# Linux ARM64
curl -L -o eureka-cli https://github.com/loprx/eureka-cli/releases/latest/download/eureka-cli-linux-arm64
chmod +x eureka-cli && sudo mv eureka-cli /usr/local/bin/

# macOS Apple Silicon
curl -L -o eureka-cli https://github.com/loprx/eureka-cli/releases/latest/download/eureka-cli-darwin-arm64
chmod +x eureka-cli && sudo mv eureka-cli /usr/local/bin/

# macOS Intel
curl -L -o eureka-cli https://github.com/loprx/eureka-cli/releases/latest/download/eureka-cli-darwin-amd64
chmod +x eureka-cli && sudo mv eureka-cli /usr/local/bin/

# Windows: download eureka-cli-windows-amd64.exe from the releases page
```

### Shell completion

```bash
# zsh — add to your fpath, e.g.
eureka-cli completion zsh > ~/.zsh/completions/_eureka-cli

# bash
eureka-cli completion bash > /etc/bash_completion.d/eureka-cli

# fish
eureka-cli completion fish > ~/.config/fish/completions/eureka-cli.fish
```

### From source

Requires Rust 1.95+.

```bash
git clone https://github.com/loprx/eureka-cli.git
cd eureka-cli
cargo build --release
sudo install target/release/eureka-cli /usr/local/bin/
```

## Quick start

```bash
# One-off: pass a URL
eureka-cli --server http://my-eureka:8761/eureka apps list

# Or save it as a default profile
eureka-cli config add prod http://my-eureka:8761/eureka --set-default
eureka-cli apps list                    # uses 'prod'
eureka-cli config use staging           # switch contexts, kubectl-style
```


## Read queries (the kubectl part)

The core of v0.2. All work on both `apps` and `instances`.

```bash
# List, kubectl-style table (no borders, scannable)
eureka-cli apps ls
eureka-cli instances ls

# Filter — exact match, comma-AND, supports nested metadata
eureka-cli instances ls -l status=UP
eureka-cli instances ls -l 'status!=UP'                 # only the broken ones
eureka-cli instances ls -l 'app=USER-SERVICE,metadata.version=v2'

# Wide output — APP / VIP / METADATA columns on instances; UP/DOWN counts on apps
eureka-cli instances ls -o wide
eureka-cli apps ls -o wide

# Sort by any field path
eureka-cli instances ls --sort-by status
eureka-cli instances ls --sort-by ip_addr

# Shortcut for "what's broken"
eureka-cli apps unhealthy
eureka-cli instances unhealthy

# Multi-section detail (kubectl describe style)
eureka-cli apps describe USER-SERVICE
eureka-cli instances describe -a USER-SERVICE 10.0.0.1:user-service:8080

# JSONPath — for shell scripts and pipes
eureka-cli -o 'jsonpath=$.instances[*].ipAddr' instances ls
eureka-cli -o 'jsonpath=$.instances[*].instanceId' -l 'status!=UP' instances ls \
  | xargs -I{} echo "would page on-call about {}"

# Watch mode — kubectl -w semantics, Ctrl+C to exit
eureka-cli instances ls -w
eureka-cli instances ls -w --watch-interval 5
```

> **Note on `app=` selector:** Eureka uppercases application names server-side, so `-l app=foo` will not match a registered "FOO". Use the actual stored value (`-l app=FOO`). Other fields preserve case as given.

## Lifecycle (write operations)

The original v0.1 surface — for register/heartbeat/status/metadata/deregister flows from inside k8s pods, jumphosts, or any binary-only environment.

```bash
ID=$(hostname)-$(date +%s)

eureka-cli register \
  --app MY-SERVICE --instance-id "$ID" \
  --hostname "$(hostname)" --ip "$(hostname -I | awk '{print $1}')" \
  --port 8080 --vip-address my-service \
  --metadata version=1.0.0 --metadata env=prod

# keep the lease
while true; do eureka-cli heartbeat MY-SERVICE "$ID"; sleep 25; done &

eureka-cli status set MY-SERVICE "$ID" OUT_OF_SERVICE   # drain
eureka-cli metadata set MY-SERVICE "$ID" canary true    # tweak metadata
eureka-cli deregister MY-SERVICE "$ID"                  # remove
```


## Commands

All commands have kubectl-style aliases; full list is in `--help`.

| Command | Aliases | What it does |
|---|---|---|
| `apps list` | `a ls` | List every registered application |
| `apps get <APP>` | — | Show one application + its instances |
| `apps describe <APP>` | `a desc` | Multi-section detail view of an application |
| `apps instances <APP>` | `a i` | List instances of an app |
| `apps unhealthy` | — | Apps with any non-UP instance |
| `instances list` | `i ls` | Flatten every instance across all apps |
| `instances get <ID>` | — | Show one instance (use `-a APP` to disambiguate) |
| `instances describe <ID>` | `i desc` | Multi-section detail view of an instance |
| `instances unhealthy` | — | Instances with status != UP |
| `register …` | `reg` | Register a new instance |
| `heartbeat <APP> <ID>` | `hb` | Send a renewal |
| `status set <APP> <ID> <STATUS>` | `st set` | Override status (UP, DOWN, OUT_OF_SERVICE, …) |
| `status remove <APP> <ID>` | `st rm` | Clear an override |
| `metadata set <APP> <ID> <K> <V>` | `meta set` | Update one metadata key |
| `vip get <VIP>` | — | Look up by vipAddress |
| `vip get-secure <SVIP>` | `vip gs` | Look up by secure vipAddress |
| `deregister <APP> <ID>` | `dereg` | Remove an instance |
| `config …` | — | Manage server profiles (kubeconfig style) |
| `servers …` | `s …` | Deprecated alias of `config`, removed in v0.4 |
| `completion <SHELL>` | — | Emit shell completion script |
| `version` | `v` | Show CLI version |

### Global flags

| Flag | Purpose |
|---|---|
| `-s, --server <NAME-OR-URL>` | Pick a server profile or pass a URL directly |
| `-o, --output <FMT>` | `table` (default) / `wide` / `json` / `yaml` / `jsonpath=<expr>` |
| `-l, --selector <EXPR>` | Filter, e.g. `status=UP,metadata.version=v2` (`=` and `!=`, comma-AND) |
| `-w, --watch` | Re-render on an interval until Ctrl+C |
| `--watch-interval <SECS>` | Watch period, default `2` |
| `--sort-by <FIELD>` | Sort output by any field path (e.g. `status`, `ip_addr`) |
| `--timeout <SECS>` | HTTP request timeout |
| `-v, --verbose` / `-q, --quiet` | Logging level |


## Server profiles (`config`, kubeconfig-style)

Switch between Eureka clusters without retyping URLs.

```bash
eureka-cli config list                         # show all
eureka-cli config current                      # show default
eureka-cli config add prod http://eu/ -D       # -D = set as default
eureka-cli config use staging                  # change default
eureka-cli config remove old
```

`--server` resolves in this order:

1. If it starts with `http://` or `https://`, used directly.
2. Otherwise looked up as a named server in the config.
3. If neither `--server` nor `EUREKA_SERVER` is set, the default server is used.

Config file: `~/.config/eureka-cli/config.yaml` (auto-created on first `config add`).

```yaml
server:
  default: prod
  servers:
    local:
      url: http://localhost:8761/eureka
      description: Local dev
    prod:
      url: https://eureka.example.com/eureka
      description: Production
  timeout: 30
  retry:
    max_attempts: 3
    backoff_ms: 1000
output:
  format: table
  color: auto
logging:
  level: info
```

> The legacy `servers …` command keeps working but emits a one-line stderr deprecation notice. It's removed in v0.4.

## Examples

### "What's broken right now?"

```bash
# Just the unhealthy ones, kubectl-style — works whether you have 5 or 500 instances
eureka-cli instances unhealthy -o wide

# Same thing, manual selector form
eureka-cli instances ls -l 'status!=UP' -o wide

# Same thing, live
eureka-cli instances unhealthy -w
```

### "Find every v2 deploy of user-service in zone us-east-1"

```bash
eureka-cli instances ls \
  -l 'app=USER-SERVICE,metadata.version=v2,metadata.zone=us-east-1' \
  -o wide
```


### "Page on-call about every DOWN instance"

```bash
eureka-cli -o 'jsonpath=$.instances[*].instanceId' \
  -l 'status=DOWN' instances ls \
  | xargs -I{} ./notify-oncall.sh {}
```

### "Watch a deploy roll out"

```bash
eureka-cli instances ls -l 'app=USER-SERVICE' -w --watch-interval 2
```

### Switch between dev / staging / prod in a loop

```bash
for srv in dev staging prod; do
  echo "=== $srv ==="
  eureka-cli --server "$srv" apps unhealthy
done
```

## Environment variables

| Var | Effect |
|---|---|
| `EUREKA_SERVER` | Same as `--server` (URL or named profile) |
| `RUST_LOG` | Tracing filter, e.g. `eureka_cli=debug` |

## Compatibility

End-to-end matrix run: 6 client hosts × 3 Eureka servers × 26 checks per round = **468 checks, all green**.

| Client OS / glibc | Eureka 1.10 | Eureka 2.0 | Spring Cloud (prod) |
|---|---|---|---|
| CentOS 7 (glibc 2.17) | ✅ | ✅ | ✅ |
| Ubuntu 24.04 (glibc 2.39) | ✅ | ✅ | ✅ |

The Linux x86_64 binary is `static-pie linked` (musl), so it runs on any glibc 2.17+ host without extra dependencies.

## Build

The release binaries come from the GitHub Actions `release.yml` workflow on tag push. To build them locally:

```bash
# Native (macOS / Linux / Windows)
cargo build --release

# Linux x86_64 musl from Apple Silicon (uses Docker)
./scripts/build-musl.sh
```

The `Dockerfile.musl` forces `--platform=linux/amd64` so the host musl-gcc matches the target arch — without that, ARM64 musl-gcc rejects the `-m64` flag that `ring`'s build script emits.

## Releasing

Tag and push:

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions builds for all 5 platforms in parallel and creates a release with the binaries attached.

## Recording the demo GIF

The demo GIF at the top of this README is generated from `assets/demo.tape` using [VHS](https://github.com/charmbracelet/vhs):

```bash
brew install vhs ttyd ffmpeg
NO_PROXY="10.0.0.0/8" vhs assets/demo.tape   # outputs assets/demo.gif
```

The `.tape` file is committed alongside the GIF, so anyone can regenerate the demo against their own Eureka.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.