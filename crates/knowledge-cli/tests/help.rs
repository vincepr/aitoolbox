use assert_cmd::Command;

#[test]
fn help_lists_knowledge_subcommands() {
    let mut cmd = Command::cargo_bin("knowledge-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("knowledge"))
        .stdout(predicates::str::contains("query"))
        .stdout(predicates::str::contains("init"))
        .stdout(predicates::str::contains("capture"));
}
