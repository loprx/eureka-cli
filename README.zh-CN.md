# eureka-cli

[![CI](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/loprx/eureka-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/loprx/eureka-cli)](https://github.com/loprx/eureka-cli/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#许可证)

[English](README.md) | 简体中文

Netflix / Spring Cloud Eureka 的 `kubectl`。单文件静态二进制,无 JVM、无运行时,专为"能放二进制但不能开端口、不能开浏览器"的环境设计。

![demo](assets/demo.gif)

## v0.2 新增

v0.2 把这个项目重新定位为 **Eureka 的 kubectl**:复用大家从 `kubectl`、`docker`、`helm` 已有的运维肌肉记忆,解决"实例几百个,只想快速找出坏的那个"的痛点。

- **`-l/--selector`** — 按任意字段过滤,支持嵌套 metadata: `-l status=UP,metadata.version=v2`
- **`-o wide`** — 多列输出(instance 加 APP/VIP/METADATA 列;app 加 UP/DOWN 计数)
- **`--sort-by`** — 按 status / ip / 任意字段路径排序
- **`unhealthy`** — apps 和 instances 都有的快捷子命令,等价于 `-l 'status!=UP'`
- **`describe`** — kubectl 风格分组视图(Identity / Status / Network / Lease / DataCenter / Metadata / Timestamps)
- **`-o jsonpath=...`** — 适合管道: `eureka-cli -o 'jsonpath=$.instances[*].ipAddr' instances ls | xargs ...`
- **`-w/--watch`** — 实时刷新,kubectl `-w` 心智,Ctrl+C 退出
- **kubectl 风格表格** — 无框线、空格对齐,同屏显示更多行
- **`completion`** — `eureka-cli completion {bash,zsh,fish,powershell}`
- **`config`** — kubeconfig 风格,`servers` 转为 deprecated 别名(v0.4 移除)

## 适用场景

为 Eureka Dashboard 不可达的环境设计:

- 没有公网访问(气隙网络 / 内网专用)
- Eureka Dashboard 端口对外封闭
- 目标机只能 SSH
- 在 K8s Pod 里执行操作(没浏览器、不想 port-forward)
- 跳板机 / 堡垒机环境

## 安装

### Homebrew (macOS / Linux)

```bash
brew install loprx/tap/eureka-cli
```

### 预编译二进制

从 [releases](https://github.com/loprx/eureka-cli/releases) 选一个。Linux 版本是 static-pie musl,任何 glibc 2.17+ 或 musl 发行版都能直接跑。

```bash
# Linux x86_64 (从 CentOS 7 到 Ubuntu 24 都能跑)
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

### Shell 自动补全

```bash
# zsh — 写脚本,再告诉 zsh 这个目录在哪
mkdir -p ~/.zsh/completions
eureka-cli completion zsh > ~/.zsh/completions/_eureka-cli

# 一次性配置:在 ~/.zshrc 的 `compinit` 之前加这两行:
#   fpath=(~/.zsh/completions $fpath)
#   autoload -Uz compinit && compinit
# 当前 shell 立即生效: exec zsh   (或开个新终端)

# bash
eureka-cli completion bash | sudo tee /etc/bash_completion.d/eureka-cli >/dev/null
# 重新加载: source /etc/bash_completion.d/eureka-cli   (或开新 shell)

# fish — 目录默认在 fish 的补全路径上,不用额外配置
mkdir -p ~/.config/fish/completions
eureka-cli completion fish > ~/.config/fish/completions/eureka-cli.fish
```

shell 重新加载后,试试看:

```bash
eureka-cli <TAB>          # 列出所有子命令
eureka-cli apps <TAB>     # list / get / describe / instances / unhealthy
eureka-cli -<TAB>         # 全局 flag: -l / -o / -w / --sort-by ...
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
# 临时:每次手动指定 URL
eureka-cli --server http://my-eureka:8761/eureka apps list

# 或者保存为默认 profile
eureka-cli config add prod http://my-eureka:8761/eureka --set-default
eureka-cli apps list                    # 自动用 'prod'
eureka-cli config use staging           # kubectl 风格切换上下文
```

## 查询命令(kubectl 风格)

v0.2 的核心。`apps` 和 `instances` 子命令上都生效。

```bash
# 列表,kubectl 风格表格(无框线、易扫读)
eureka-cli apps ls
eureka-cli instances ls

# 过滤 — 精确匹配,逗号 AND,支持嵌套 metadata
eureka-cli instances ls -l status=UP
eureka-cli instances ls -l 'status!=UP'                 # 只看坏的
eureka-cli instances ls -l 'app=USER-SERVICE,metadata.version=v2'

# 多列输出 — instance 上加 APP/VIP/METADATA 列;app 上加 UP/DOWN 计数
eureka-cli instances ls -o wide
eureka-cli apps ls -o wide

# 按任意字段路径排序
eureka-cli instances ls --sort-by status
eureka-cli instances ls --sort-by ip_addr

# "哪些是坏的" 快捷
eureka-cli apps unhealthy
eureka-cli instances unhealthy

# 多分组详情(kubectl describe 风格)
eureka-cli apps describe USER-SERVICE
eureka-cli instances describe -a USER-SERVICE 10.0.0.1:user-service:8080

# JSONPath — 给 shell 脚本和管道用
eureka-cli -o 'jsonpath=$.instances[*].ipAddr' instances ls
eureka-cli -o 'jsonpath=$.instances[*].instanceId' -l 'status!=UP' instances ls \
  | xargs -I{} echo "would page on-call about {}"

# Watch 模式 — kubectl -w 语义,Ctrl+C 退出
eureka-cli instances ls -w
eureka-cli instances ls -w --watch-interval 5
```

> **关于 `app=` selector:** Eureka 服务端会把 application name 全转大写,所以 `-l app=foo` 不会命中已注册的 "FOO"。要用真实存储值(`-l app=FOO`)。其他字段保留原始大小写。

## 生命周期(写操作)

v0.1 原有的能力 — register/heartbeat/status/metadata/deregister 流程,适合 K8s pod、跳板机、任何只能放二进制的环境。

```bash
ID=$(hostname)-$(date +%s)

eureka-cli register \
  --app MY-SERVICE --instance-id "$ID" \
  --hostname "$(hostname)" --ip "$(hostname -I | awk '{print $1}')" \
  --port 8080 --vip-address my-service \
  --metadata version=1.0.0 --metadata env=prod

# 持续续约
while true; do eureka-cli heartbeat MY-SERVICE "$ID"; sleep 25; done &

eureka-cli status set MY-SERVICE "$ID" OUT_OF_SERVICE   # 流量摘除
eureka-cli metadata set MY-SERVICE "$ID" canary true    # 改 metadata
eureka-cli deregister MY-SERVICE "$ID"                  # 注销
```


## 命令一览

所有命令都有 kubectl 风格的别名,完整列表见 `--help`。

| 命令 | 别名 | 作用 |
|---|---|---|
| `apps list` | `a ls` | 列出所有注册的应用 |
| `apps get <APP>` | — | 显示单个应用 + 它的实例 |
| `apps describe <APP>` | `a desc` | 应用的多分组详情 |
| `apps instances <APP>` | `a i` | 列出某个应用的实例 |
| `apps unhealthy` | — | 含有非 UP 实例的应用 |
| `instances list` | `i ls` | 平铺所有应用的实例 |
| `instances get <ID>` | — | 显示单个实例(用 `-a APP` 消歧义) |
| `instances describe <ID>` | `i desc` | 实例的多分组详情 |
| `instances unhealthy` | — | 状态不是 UP 的实例 |
| `register …` | `reg` | 注册一个新实例 |
| `heartbeat <APP> <ID>` | `hb` | 续约 |
| `status set <APP> <ID> <STATUS>` | `st set` | 覆盖状态(UP, DOWN, OUT_OF_SERVICE…) |
| `status remove <APP> <ID>` | `st rm` | 清除状态覆盖 |
| `metadata set <APP> <ID> <K> <V>` | `meta set` | 更新一个 metadata 键 |
| `vip get <VIP>` | — | 按 vipAddress 查找 |
| `vip get-secure <SVIP>` | `vip gs` | 按 secure vipAddress 查找 |
| `deregister <APP> <ID>` | `dereg` | 注销实例 |
| `config …` | — | 管理服务器 profile(kubeconfig 风格) |
| `servers …` | `s …` | `config` 的 deprecated 别名,v0.4 移除 |
| `completion <SHELL>` | — | 输出 shell 自动补全脚本 |
| `version` | `v` | 显示 CLI 版本 |

### 全局 flag

| Flag | 作用 |
|---|---|
| `-s, --server <NAME-OR-URL>` | 选择服务器 profile,或直接传 URL |
| `-o, --output <FMT>` | `table`(默认) / `wide` / `json` / `yaml` / `jsonpath=<expr>` |
| `-l, --selector <EXPR>` | 过滤,例如 `status=UP,metadata.version=v2`(支持 `=` 和 `!=`,逗号 AND) |
| `-w, --watch` | 按间隔重渲染,直到 Ctrl+C |
| `--watch-interval <SECS>` | watch 周期,默认 `2` |
| `--sort-by <FIELD>` | 按任意字段路径排序(如 `status`、`ip_addr`) |
| `--timeout <SECS>` | HTTP 请求超时 |
| `-v, --verbose` / `-q, --quiet` | 日志等级 |


## 服务器 profile(`config`,kubeconfig 风格)

不同 Eureka 集群之间切换,免去每次输 URL。

```bash
eureka-cli config list                         # 列出全部
eureka-cli config current                      # 当前默认
eureka-cli config add prod http://eu/ -D       # -D = 同时设为默认
eureka-cli config use staging                  # 切换默认
eureka-cli config remove old
```

`--server` 解析顺序:

1. 以 `http://` / `https://` 开头 → 直接当 URL 用
2. 否则在 config 里查命名 server
3. 若 `--server` 和 `EUREKA_SERVER` 都没设 → 用默认 server

配置文件:`~/.config/eureka-cli/config.yaml`(首次 `config add` 时自动建)。

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

> 老的 `servers …` 命令仍然能用,但会在 stderr 打一行 deprecation 提示。v0.4 会移除。

## 示例

### "现在哪些是坏的?"

```bash
# 只看不健康的,kubectl 风格 — 5 个还是 500 个实例都好用
eureka-cli instances unhealthy -o wide

# 同样,但用手写 selector 形式
eureka-cli instances ls -l 'status!=UP' -o wide

# 同样,实时看
eureka-cli instances unhealthy -w
```

### "找出 us-east-1 区所有 v2 版的 user-service"

```bash
eureka-cli instances ls \
  -l 'app=USER-SERVICE,metadata.version=v2,metadata.zone=us-east-1' \
  -o wide
```

### "把每个 DOWN 状态的实例报警给 on-call"

```bash
eureka-cli -o 'jsonpath=$.instances[*].instanceId' \
  -l 'status=DOWN' instances ls \
  | xargs -I{} ./notify-oncall.sh {}
```

### "盯着发布滚动"

```bash
eureka-cli instances ls -l 'app=USER-SERVICE' -w --watch-interval 2
```

### 在 dev / staging / prod 之间循环

```bash
for srv in dev staging prod; do
  echo "=== $srv ==="
  eureka-cli --server "$srv" apps unhealthy
done
```


## 环境变量

| 变量 | 作用 |
|---|---|
| `EUREKA_SERVER` | 等同于 `--server`(URL 或命名 profile) |
| `RUST_LOG` | tracing 过滤器,例如 `eureka_cli=debug` |

## 兼容性

端到端矩阵测试:6 台客户端 × 3 个 Eureka server × 每轮 26 项检查 = **468 项检查全过**。

| 客户端 OS / glibc | Eureka 1.10 | Eureka 2.0 | Spring Cloud(生产) |
|---|---|---|---|
| CentOS 7 (glibc 2.17) | ✅ | ✅ | ✅ |
| Ubuntu 24.04 (glibc 2.39) | ✅ | ✅ | ✅ |

Linux x86_64 二进制是 `static-pie linked`(musl),任何 glibc 2.17+ 主机都能跑,不需要额外依赖。

## 构建

发布版二进制由 GitHub Actions 在 tag push 时自动构建。本地构建:

```bash
# 原生(macOS / Linux / Windows)
cargo build --release

# 在 Apple Silicon 上构建 Linux x86_64 musl(用 Docker)
./scripts/build-musl.sh
```

`Dockerfile.musl` 强制 `--platform=linux/amd64`,这样宿主的 musl-gcc 才能匹配目标架构 — 否则 ARM64 的 musl-gcc 会拒绝 `ring` 构建脚本里发的 `-m64` flag。

## 发布

打 tag 推送即可:

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions 会并行为 5 个平台构建,并把 binary 挂到 release 上。

## 许可证

双许可,任选其一:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
