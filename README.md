# eureka-cli

[![CI](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/loprx/eureka-cli)](https://github.com/loprx/eureka-cli/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

English | [简体中文](README.zh-CN.md)

A standalone, statically-linked CLI for Netflix / Spring Cloud Eureka. Single binary, no JVM, no runtime, works in environments where you can ship a binary but not expose ports.

## Why

When you can drop a binary on a host but can't expose Eureka, can't run a full JVM, and shell scripts feel fragile — `eureka-cli` does the lifecycle calls (register / heartbeat / status / metadata / deregister) and read queries from one ~4 MB ELF.

Tested against:
- Netflix Eureka 1.x
- Spring Cloud Netflix Eureka (Boot 2.7 / Spring Cloud 2021)
- Spring Cloud Netflix Eureka (Boot 3.3 / Spring Cloud 2023)

## Install

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
# One-off: pass a URL each time
eureka-cli --server http://my-eureka:8761/eureka apps list

# Persistent: save it as a named server, set as default
eureka-cli servers add prod http://my-eureka:8761/eureka --set-default
eureka-cli apps list      # uses 'prod'
```

## Commands

All commands have k8s-style aliases shown in `--help`.

| Command | Aliases | What it does |
|---|---|---|
| `apps list` | `a ls` | List every registered application |
| `apps get <APP>` | `a get` | Show one application + its instances |
| `apps instances <APP>` | `a i` | List instances of an app |
| `instances list` | `i ls` | Flatten every instance across all apps |
| `instances get <ID>` | `i get` | Show one instance (use `-a APP` to disambiguate) |
| `register ...` | `reg` | Register a new instance |
| `heartbeat <APP> <ID>` | `hb` | Send a renewal |
| `status set <APP> <ID> <STATUS>` | `st set` | Override status (UP, DOWN, OUT_OF_SERVICE, …) |
| `status remove <APP> <ID>` | `st rm` | Clear an override |
| `metadata set <APP> <ID> <K> <V>` | `meta set` | Update one metadata key |
| `vip get <VIP>` | — | Look up by vipAddress |
| `vip get-secure <SVIP>` | `vip gs` | Look up by secure vipAddress |
| `deregister <APP> <ID>` | `dereg` | Remove an instance |
| `servers ...` | `s ...` | Manage named server configs (see below) |

Output format applies to all read commands: `--output table|json|yaml` (table is default).

## Multi-server config

Switch between Eureka clusters without retyping URLs. Like `kubectl` contexts.

```bash
eureka-cli servers list                         # show all
eureka-cli servers current                      # show default
eureka-cli servers add prod http://eu/ -D       # -D = set as default
eureka-cli servers use staging                  # change default
eureka-cli servers remove old
```

`--server` flag resolves in this order:

1. If it starts with `http://` or `https://`, it's used directly.
2. Otherwise it's looked up as a named server in the config.
3. If neither `--server` nor `EUREKA_SERVER` is set, the default server is used.

Config file: `~/.config/eureka-cli/config.yaml` (auto-created on first `servers add`).

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

The CLI rejects URLs without `http(s)://` at `servers add` time, and gives a copy-pasteable fix hint if it finds an old corrupt entry on load.

## Examples

### Full lifecycle

```bash
ID=$(hostname)-$(date +%s)

eureka-cli reg \
  --app MY-SERVICE --instance-id "$ID" \
  --hostname "$(hostname)" --ip "$(hostname -I | awk '{print $1}')" \
  --port 8080 --vip-address my-service \
  --metadata version=1.0.0 --metadata env=prod

# keep the lease
while true; do eureka-cli hb MY-SERVICE "$ID"; sleep 25; done &

eureka-cli st set MY-SERVICE "$ID" OUT_OF_SERVICE   # drain
eureka-cli dereg MY-SERVICE "$ID"                   # remove
```

### Scripting with `--output json`

```bash
# every UP instance, flattened to "<app> <ip>:<port>"
eureka-cli --output json instances list \
  | jq -r '.[] | select(.status == "UP") | "\(.app) \(.ipAddr):\(.port["$"])"'
```

### Switch between dev / staging / prod

```bash
for srv in dev staging prod; do
  echo "=== $srv ==="
  eureka-cli --server "$srv" apps list
done
```

## Environment variables

| Var | Effect |
|---|---|
| `EUREKA_SERVER` | Same as `--server` (URL or named) |
| `RUST_LOG` | Tracing filter, e.g. `eureka_cli=debug` |

## Compatibility

Verified against the following clients in production: 5 hosts × 4 server versions × 20 commands = 400 invocations all green.

| Client OS / glibc | Eureka 1.x | Spring Cloud 2021 | Spring Cloud 2023 |
|---|---|---|---|
| CentOS 7 (glibc 2.17) | ✅ | ✅ | ✅ |
| CentOS 7 (glibc 2.31) | ✅ | ✅ | ✅ |
| Ubuntu 24.04 (glibc 2.39) | ✅ | ✅ | ✅ |

Linux x86_64 binary is `static-pie linked` (musl), so it runs on any glibc 2.17+ host without extra dependencies.

## Build

The release binaries come from the GitHub Actions `release.yml` workflow on tag push. To build them locally:

```bash
# Native (macOS / Linux / Windows)
cargo build --release

# Linux x86_64 musl from Apple Silicon (uses Docker)
DOCKER_BUILDKIT=1 docker build \
  --platform linux/amd64 \
  --target builder \
  -t eureka-cli-builder:musl \
  -f Dockerfile.musl .
docker create --name extract eureka-cli-builder:musl
docker cp extract:/build/target/x86_64-unknown-linux-musl/release/eureka-cli ./
docker rm extract
```

The `Dockerfile.musl` forces `--platform=linux/amd64` so the host musl-gcc matches the target arch — without that, ARM64 musl-gcc rejects the `-m64` flag that `ring`'s build script emits.

## Releasing

Tag and push:

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions builds for all 5 platforms in parallel and creates a release with the binaries attached.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
