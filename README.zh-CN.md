# eureka-cli

[![CI](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/loprx/eureka-cli)](https://github.com/loprx/eureka-cli/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#许可证)

[English](README.md) | 简体中文

一个独立的、静态链接的 Netflix / Spring Cloud Eureka 命令行工具。单二进制、零 JVM、零运行时依赖,适用于只能投递二进制、无法暴露端口的环境。

## 适用场景

为 Eureka Dashboard 不可达的环境设计:

- 无公网环境(内网隔离 / 气隙网络)
- 无法暴露 Eureka Dashboard 端口
- 仅 SSH 可访问的目标机器
- K8s Pod 内运维(没浏览器、不方便 port-forward)
- 堡垒机 / 跳板机场景

## 为什么需要它

当你只能在主机上放一个二进制、无法暴露 Eureka 端口、不能跑完整 JVM、shell 脚本又显得脆弱时,`eureka-cli` 用一个约 4 MB 的 ELF 文件就能完成全部生命周期调用(注册 / 心跳 / 状态 / 元数据 / 注销)和读查询。

已验证兼容:
- Netflix Eureka 1.x
- Spring Cloud Netflix Eureka(Boot 2.7 / Spring Cloud 2021)
- Spring Cloud Netflix Eureka(Boot 3.3 / Spring Cloud 2023)

## 安装

### 预编译二进制

从 [releases](https://github.com/loprx/eureka-cli/releases) 选一个。Linux 版本是 static-pie musl,在任何 glibc 2.17+ 或 musl 发行版上都能跑。

```bash
# Linux x86_64(从 CentOS 7 到 Ubuntu 24 都能跑)
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

# Windows: 从 releases 页面下载 eureka-cli-windows-amd64.exe
```

### 源码构建

需要 Rust 1.95+。

```bash
git clone https://github.com/loprx/eureka-cli.git
cd eureka-cli
cargo build --release
sudo install target/release/eureka-cli /usr/local/bin/
```

## 快速开始

```bash
# 临时用法:每次手动指定 URL
eureka-cli --server http://my-eureka:8761/eureka apps list

# 持久化用法:存为命名配置并设为默认
eureka-cli servers add prod http://my-eureka:8761/eureka --set-default
eureka-cli apps list      # 自动用 'prod'
```

## 命令

所有命令都有 k8s 风格的简写,在 `--help` 中可见。

| 命令 | 简写 | 作用 |
|---|---|---|
| `apps list` | `a ls` | 列出所有已注册的应用 |
| `apps get <APP>` | `a get` | 显示某个应用及其实例 |
| `apps instances <APP>` | `a i` | 列出某个应用的实例 |
| `instances list` | `i ls` | 平铺列出所有应用下的所有实例 |
| `instances get <ID>` | `i get` | 显示单个实例(用 `-a APP` 消除歧义) |
| `register ...` | `reg` | 注册一个新实例 |
| `heartbeat <APP> <ID>` | `hb` | 续约心跳 |
| `status set <APP> <ID> <STATUS>` | `st set` | 覆盖状态(UP, DOWN, OUT_OF_SERVICE 等) |
| `status remove <APP> <ID>` | `st rm` | 清除状态覆盖 |
| `metadata set <APP> <ID> <K> <V>` | `meta set` | 更新一个元数据键 |
| `vip get <VIP>` | — | 按 vipAddress 查询 |
| `vip get-secure <SVIP>` | `vip gs` | 按 secureVipAddress 查询 |
| `deregister <APP> <ID>` | `dereg` | 注销实例 |
| `servers ...` | `s ...` | 管理命名服务器配置(详见下方) |

输出格式适用于所有读命令:`--output table|json|yaml`(默认 table)。

## 多服务器配置

像 `kubectl` 的 context 一样,在多个 Eureka 集群之间切换,不用每次输 URL。

```bash
eureka-cli servers list                         # 显示所有配置
eureka-cli servers current                      # 显示当前默认
eureka-cli servers add prod http://eu/ -D       # -D 表示设为默认
eureka-cli servers use staging                  # 切换默认
eureka-cli servers remove old
```

`--server` 参数的解析顺序:

1. 如果以 `http://` 或 `https://` 开头,直接当作 URL 使用。
2. 否则在配置中查找命名服务器。
3. 如果 `--server` 和 `EUREKA_SERVER` 都没设置,使用默认服务器。

配置文件:`~/.config/eureka-cli/config.yaml`(首次 `servers add` 时自动创建)。

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

`servers add` 时会拒绝缺少 `http(s)://` 的 URL;如果加载时发现旧版遗留的非法配置,会给出可直接复制的修复提示。

## 示例

### 完整生命周期

```bash
ID=$(hostname)-$(date +%s)

eureka-cli reg \
  --app MY-SERVICE --instance-id "$ID" \
  --hostname "$(hostname)" --ip "$(hostname -I | awk '{print $1}')" \
  --port 8080 --vip-address my-service \
  --metadata version=1.0.0 --metadata env=prod

# 持续续约
while true; do eureka-cli hb MY-SERVICE "$ID"; sleep 25; done &

eureka-cli st set MY-SERVICE "$ID" OUT_OF_SERVICE   # 流量摘除
eureka-cli dereg MY-SERVICE "$ID"                   # 注销
```

### 配合 `--output json` 做脚本

```bash
# 把所有 UP 状态的实例平铺成 "<app> <ip>:<port>"
eureka-cli --output json instances list \
  | jq -r '.[] | select(.status == "UP") | "\(.app) \(.ipAddr):\(.port["$"])"'
```

### 在 dev / staging / prod 之间切换

```bash
for srv in dev staging prod; do
  echo "=== $srv ==="
  eureka-cli --server "$srv" apps list
done
```

## 环境变量

| 变量 | 作用 |
|---|---|
| `EUREKA_SERVER` | 等同 `--server`(URL 或命名) |
| `RUST_LOG` | tracing 过滤器,如 `eureka_cli=debug` |

## 兼容性

生产环境验证矩阵:5 台客户端 × 4 个服务器版本 × 20 个命令 = 400 次调用全部通过。

| 客户端 OS / glibc | Eureka 1.x | Spring Cloud 2021 | Spring Cloud 2023 |
|---|---|---|---|
| CentOS 7(glibc 2.17) | ✅ | ✅ | ✅ |
| CentOS 7(glibc 2.31) | ✅ | ✅ | ✅ |
| Ubuntu 24.04(glibc 2.39) | ✅ | ✅ | ✅ |

Linux x86_64 二进制是 `static-pie linked`(musl),在任何 glibc 2.17+ 主机上都能跑,无额外依赖。

## 构建

正式版二进制由 GitHub Actions 的 `release.yml` 工作流在 tag 推送时构建。本地构建方法:

```bash
# 原生构建(macOS / Linux / Windows)
cargo build --release

# 在 Apple Silicon 上构建 Linux x86_64 musl(用 Docker)
DOCKER_BUILDKIT=1 docker build \
  --platform linux/amd64 \
  --target builder \
  -t eureka-cli-builder:musl \
  -f Dockerfile.musl .
docker create --name extract eureka-cli-builder:musl
docker cp extract:/build/target/x86_64-unknown-linux-musl/release/eureka-cli ./
docker rm extract
```

`Dockerfile.musl` 强制 `--platform=linux/amd64`,让宿主 musl-gcc 与目标架构对齐;否则 ARM64 musl-gcc 会拒绝 `ring` 构建脚本生成的 `-m64` 参数。

## 发布

打 tag 并推送:

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions 会并行构建 5 个平台并创建带二进制附件的 release。

## 许可证

双许可,任选其一:

- Apache License, Version 2.0([LICENSE-APACHE](LICENSE-APACHE))
- MIT license([LICENSE-MIT](LICENSE-MIT))
