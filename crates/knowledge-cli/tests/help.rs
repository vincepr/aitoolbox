use assert_cmd::Command;

#[test]
fn help_lists_knowledge_subcommands() {
    let mut cmd = Command::cargo_bin("knowledge-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Local-first knowledge system CLI backed by SQLite and compact Markdown notes.",
        ))
        .stdout(predicates::str::contains("Examples:"))
        .stdout(predicates::str::contains("get"))
        .stdout(predicates::str::contains("init"))
        .stdout(predicates::str::contains("quickstart"))
        .stdout(predicates::str::contains("capture-lesson"))
        .stdout(predicates::str::contains("capture-issue"))
        .stdout(predicates::str::contains("completions"))
        .stdout(predicates::str::contains("alias"));
}
