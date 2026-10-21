//! The built `buffoonery` binary as a process: exit codes, stdin, and which
//! stream each message goes to.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn buffoonery(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_buffoonery"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(stdin.as_bytes())
        .expect("stdin takes the board");
    child.wait_with_output().expect("the binary exits")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[test]
fn a_good_board_exits_0() {
    let out = buffoonery(&["score", "-"], "played: [AS, AH]");
    assert_eq!(out.status.code(), Some(0));
    assert!(text(&out.stdout).ends_with("Pair  32 × 2 = 64\n"));
    assert_eq!(text(&out.stderr), "");
}

#[test]
fn no_file_reads_stdin() {
    let out = buffoonery(&["score", "--json"], "played: [AS]");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        text(&out.stdout),
        format!("{}\n", buffoonery_component::score("played: [AS]").unwrap())
    );
}

#[test]
fn a_bad_board_exits_1_with_the_message_on_stderr() {
    let out = buffoonery(&["score", "-"], "jokers: [{ident: NOPE}]");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(text(&out.stdout), "");
    assert_eq!(
        text(&out.stderr),
        "buffoonery: jokers[0]: unknown joker ident \"NOPE\"\n"
    );
}

#[test]
fn a_missing_file_exits_2() {
    let out = buffoonery(&["score", "no_such_board.yaml"], "");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(text(&out.stdout), "");
    assert!(text(&out.stderr).contains("no_such_board.yaml"));
}

#[test]
fn an_unknown_flag_exits_2() {
    let out = buffoonery(&["score", "--frobnicate"], "");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(text(&out.stdout), "");
    assert_ne!(text(&out.stderr), "");
}

#[test]
fn version_matches_the_engine() {
    let out = buffoonery(&["version"], "");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(text(&out.stdout), format!("{}\n", buffoonery::VERSION));
}
