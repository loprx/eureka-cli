# eureka-cli v0.2 demo
#
# Generate the README's GIF:
#   brew install vhs ttyd ffmpeg
#   NO_PROXY="10.0.0.0/8" vhs assets/demo.tape
#
# Prereq:
#   eureka-cli config add demo <YOUR-EUREKA-URL> -D
#
# Output: assets/demo.gif

Output assets/demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set PlaybackSpeed 1.0

# ─── Intro ────────────────────────────────────────────────────────────
Type "# eureka-cli v0.2 — the kubectl of Eureka"
Sleep 1500
Enter
Sleep 500
Type "clear"
Enter
Sleep 300

# ─── 1. List with kubectl-style table ────────────────────────────────
Type "# kubectl-style table: no borders, scannable"
Sleep 1200
Enter
Type "eureka-cli apps ls"
Sleep 400
Enter
Sleep 2500

# ─── 2. Wide output ──────────────────────────────────────────────────
Type "clear && eureka-cli instances ls -o wide"
Sleep 600
Enter
Sleep 3000

# ─── 3. Selector ─────────────────────────────────────────────────────
Type "clear"
Enter
Type "# -l filter: kubectl-style selector"
Sleep 1000
Enter
Type "eureka-cli instances ls -l 'status=UP,app=API-GATEWAY'"
Sleep 400
Enter
Sleep 2500

# ─── 4. Unhealthy shortcut ───────────────────────────────────────────
Type "# 'unhealthy' = -l 'status!=UP', the on-call shortcut"
Sleep 1000
Enter
Type "eureka-cli instances unhealthy"
Sleep 400
Enter
Sleep 1500

# ─── 5. Sort-by ──────────────────────────────────────────────────────
Type "clear && eureka-cli instances ls --sort-by status -o wide"
Sleep 600
Enter
Sleep 2500

# ─── 6. Describe ─────────────────────────────────────────────────────
Type "clear"
Enter
Type "# describe: kubectl-style multi-section view"
Sleep 1000
Enter
Type "eureka-cli apps describe API-GATEWAY"
Sleep 400
Enter
Sleep 4000

# ─── 7. JSONPath + pipe ──────────────────────────────────────────────
Type "clear"
Enter
Type "# jsonpath: pipe-friendly extraction"
Sleep 1000
Enter
Type "eureka-cli -o 'jsonpath=$.instances[*].ipAddr' instances ls | head -5"
Sleep 400
Enter
Sleep 2500

Type "# chain to xargs for ops automation"
Sleep 800
Enter
Type "eureka-cli -o 'jsonpath=$.instances[*].instanceId' -l 'app=ORDER-SERVICE' instances ls"
Sleep 400
Enter
Sleep 2000

# ─── 8. Watch (briefly) ──────────────────────────────────────────────
Type "clear"
Enter
Type "# -w live refresh, kubectl semantics"
Sleep 1000
Enter
Type "eureka-cli instances unhealthy -w --watch-interval 2"
Sleep 400
Enter
Sleep 4500
Ctrl+C
Sleep 800

# ─── 9. Config (kubeconfig style) ────────────────────────────────────
Type "clear && eureka-cli config ls"
Sleep 600
Enter
Sleep 2000

# ─── 10. Completion (last 3 seconds) ─────────────────────────────────
Type "clear && eureka-cli completion zsh | head -10"
Sleep 600
Enter
Sleep 2500

# ─── Outro ───────────────────────────────────────────────────────────
Type "clear"
Enter
Type "# Single 5.8MB static binary. Works on CentOS 7 → Ubuntu 24."
Sleep 2500
