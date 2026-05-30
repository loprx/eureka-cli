//! Integration tests that exercise the *built binary* against real Unix
//! plumbing — pipes, signals, stdout/stderr separation. These catch the
//! class of bugs unit tests miss because they don't actually fork a process.
//!
//! Run with:  cargo test --test cli
//!
//! The binary is built by Cargo automatically before the test runs.

use std::io::Read;
use std::process::{Command, Stdio};

/// Path to the just-built binary. Cargo sets CARGO_BIN_EXE_<name> for any
/// `[[bin]]` defined in Cargo.toml.
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_eureka-cli")
}

/// Build a Command with the test binary, no Eureka network involved.
fn cmd(args: &[&str]) -> Command {
    let mut c = Command::new(bin());
    c.args(args);
    c
}

#[test]
fn version_succeeds() {
    let out = cmd(&["version"]).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("eureka-cli"), "got: {}", stdout);
}

#[test]
fn help_succeeds() {
    let out = cmd(&["--help"]).output().unwrap();
    assert!(out.status.success());
}

/// Regression for the bug a user hit: kubectl-style flag placement after the
/// subcommand must parse. Before `global = true` this exited 2 with
/// "unexpected argument '-l' found".
#[test]
fn global_flag_after_subcommand_parses() {
    // We can't actually fetch from a Eureka here, but parse-failures show up
    // as exit 2 with "error:" on stderr *before* any network attempt.
    let out = cmd(&[
        "instances",
        "list",
        "-l",
        "status=UP",
        "--server",
        "http://127.0.0.1:1/eureka",
        "--timeout",
        "1",
    ])
    .output()
    .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "kubectl-style flag placement should parse. stderr: {}",
        stderr
    );
}

#[test]
fn output_format_after_subcommand_parses() {
    let out = cmd(&[
        "apps",
        "list",
        "-o",
        "wide",
        "--server",
        "http://127.0.0.1:1/eureka",
        "--timeout",
        "1",
    ])
    .output()
    .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "stderr: {}",
        stderr
    );
}

#[test]
fn jsonpath_format_after_subcommand_parses() {
    let out = cmd(&[
        "instances",
        "list",
        "-o",
        "jsonpath=$.foo",
        "--server",
        "http://127.0.0.1:1/eureka",
        "--timeout",
        "1",
    ])
    .output()
    .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "stderr: {}",
        stderr
    );
}

/// Regression for SIGPIPE panic. Before the SIG_DFL fix in main.rs, this
/// command panicked with "BrokenPipe" once the consumer (head -1) closed
/// the pipe, leaving an "Os { code: 32, kind: BrokenPipe }" trail on stderr.
#[test]
fn completion_through_truncating_pipe_does_not_panic() {
    // Spawn `eureka-cli completion zsh` and read only the first 200 bytes,
    // then close — simulating `| head -1`. The process should exit cleanly,
    // not panic.
    let mut child = Command::new(bin())
        .args(["completion", "zsh"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let mut stdout = child.stdout.take().unwrap();
        let mut buf = [0u8; 200];
        let _ = stdout.read(&mut buf);
        // Drop stdout pipe, which sends SIGPIPE to the child on the next write.
    }

    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("panicked"),
        "completion piped to truncated reader should not panic. stderr:\n{}",
        stderr
    );
    assert!(
        !stderr.contains("BrokenPipe"),
        "completion should not surface BrokenPipe. stderr:\n{}",
        stderr
    );
}

#[test]
fn completion_full_output_succeeds() {
    let out = cmd(&["completion", "zsh"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("#compdef eureka-cli"));
}

#[test]
fn completion_bash_succeeds() {
    let out = cmd(&["completion", "bash"]).output().unwrap();
    assert!(out.status.success());
}

/// All four supported shells should generate something non-empty.
#[test]
fn completion_all_shells_succeed() {
    for shell in &["bash", "zsh", "fish", "powershell"] {
        let out = cmd(&["completion", shell]).output().unwrap();
        assert!(
            out.status.success(),
            "completion {} failed; stderr: {}",
            shell,
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !out.stdout.is_empty(),
            "completion {} produced empty output",
            shell
        );
    }
}

/// Each generated completion script should mention the binary name. Catches
/// regressions where clap_complete falls back to a default name.
#[test]
fn completion_scripts_reference_binary_name() {
    for shell in &["bash", "zsh", "fish"] {
        let out = cmd(&["completion", shell]).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("eureka-cli"),
            "{} completion missing binary name",
            shell
        );
    }
}

/// SIGPIPE regression for every shell, not just zsh. The bug only shows up
/// when the consumer closes early — `head -n 1` is enough.
#[test]
fn completion_through_truncating_pipe_does_not_panic_for_any_shell() {
    for shell in &["bash", "zsh", "fish", "powershell"] {
        let mut child = Command::new(bin())
            .args(["completion", shell])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        {
            let mut stdout = child.stdout.take().unwrap();
            let mut buf = [0u8; 50];
            let _ = stdout.read(&mut buf);
        }

        let out = child.wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stderr.contains("panicked"),
            "completion {} panicked on truncated pipe. stderr:\n{}",
            shell,
            stderr
        );
        assert!(
            !stderr.contains("BrokenPipe"),
            "completion {} surfaced BrokenPipe. stderr:\n{}",
            shell,
            stderr
        );
    }
}

/// help is a giant payload — exactly the kind of output users pipe to less/head.
/// Make sure it survives a truncated reader too.
#[test]
fn help_through_truncating_pipe_does_not_panic() {
    let mut child = Command::new(bin())
        .args(["--help"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdout = child.stdout.take().unwrap();
        let mut buf = [0u8; 30];
        let _ = stdout.read(&mut buf);
    }
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panicked"), "stderr:\n{}", stderr);
}

/// Both `servers` and `config` should reach the same code path for `list`.
/// `servers` additionally emits the deprecation notice on stderr.
#[test]
fn config_and_servers_aliases_both_parse() {
    for top in &["config", "servers"] {
        for verb in &["list", "current"] {
            let out = cmd(&[top, verb]).output().unwrap();
            // Both should complete (exit 0). The actual config file may not
            // exist — that returns an error, which is fine, just not a parse
            // failure.
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert!(
                !stderr.contains("unexpected argument"),
                "{} {} failed to parse: {}",
                top,
                verb,
                stderr
            );
        }
    }
}

#[test]
fn servers_alias_emits_deprecation_on_stderr() {
    let out = cmd(&["servers", "list"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("deprecated"),
        "servers should warn about deprecation, got stderr:\n{}",
        stderr
    );
}

#[test]
fn config_alias_does_not_emit_deprecation() {
    let out = cmd(&["config", "list"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("deprecated"),
        "config should NOT warn about deprecation, got stderr:\n{}",
        stderr
    );
}

/// Cross product of (apps subcommand) × (global flag placement). Exhaustive
/// because this is the exact bug class users hit.
#[test]
fn global_flags_after_every_listy_subcommand() {
    let common = ["--server", "http://127.0.0.1:1/eureka", "--timeout", "1"];
    let cases: &[&[&str]] = &[
        &["apps", "list", "-l", "status=UP"],
        &["apps", "list", "-o", "wide"],
        &["apps", "list", "-o", "json"],
        &["apps", "list", "-o", "yaml"],
        &["apps", "list", "-o", "jsonpath=$.foo"],
        &["apps", "list", "--sort-by", "status"],
        &["apps", "unhealthy", "-o", "wide"],
        &["instances", "list", "-l", "status=UP"],
        &["instances", "list", "-l", "status!=UP"],
        &["instances", "list", "-l", "metadata.version=v2"],
        &["instances", "list", "-l", "status=UP,app=FOO"],
        &["instances", "list", "-o", "wide"],
        &["instances", "list", "-o", "jsonpath=$.instances[*].ipAddr"],
        &["instances", "list", "--sort-by", "ip_addr"],
        &["instances", "unhealthy", "-o", "wide"],
        &["instances", "unhealthy", "--sort-by", "status"],
    ];
    for case in cases {
        let mut all: Vec<&str> = case.to_vec();
        all.extend_from_slice(&common);
        let out = Command::new(bin()).args(&all).output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stderr.contains("unexpected argument"),
            "case {:?} should parse but stderr says: {}",
            case,
            stderr
        );
    }
}

/// Same flags, both placement orders, must be equivalent w.r.t. parsing.
#[test]
fn global_flags_placement_is_symmetric() {
    let common = ["--server", "http://127.0.0.1:1/eureka", "--timeout", "1"];

    let pairs: &[(&[&str], &[&str])] = &[
        (
            &["-l", "status=UP", "instances", "list"],
            &["instances", "list", "-l", "status=UP"],
        ),
        (
            &["-o", "wide", "instances", "list"],
            &["instances", "list", "-o", "wide"],
        ),
        (
            &["--sort-by", "status", "apps", "list"],
            &["apps", "list", "--sort-by", "status"],
        ),
    ];
    for (a, b) in pairs {
        for variant in &[a, b] {
            let mut all: Vec<&str> = variant.to_vec();
            all.extend_from_slice(&common);
            let out = Command::new(bin()).args(&all).output().unwrap();
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert!(
                !stderr.contains("unexpected argument"),
                "variant {:?} should parse, stderr: {}",
                variant,
                stderr
            );
        }
    }
}

/// Bad selector syntax must error out — not crash, not silently pass.
#[test]
fn invalid_selector_fails_cleanly() {
    let out = cmd(&[
        "instances",
        "list",
        "-l",
        "garbage_no_op",
        "--server",
        "http://127.0.0.1:1/eureka",
        "--timeout",
        "1",
    ])
    .output()
    .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panicked"), "stderr: {}", stderr);
}

#[test]
fn invalid_output_format_fails_cleanly() {
    let out = cmd(&["apps", "list", "-o", "xml"]).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panicked"), "stderr: {}", stderr);
}

#[test]
fn unknown_subcommand_fails_with_exit_2() {
    let out = cmd(&["totally-not-a-command"]).output().unwrap();
    // clap convention: parse errors exit 2.
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn missing_required_arg_fails_cleanly() {
    // apps describe needs an APP id
    let out = cmd(&["apps", "describe"]).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("panicked"));
    assert!(stderr.to_lowercase().contains("required") || stderr.contains("Usage"));
}
