use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/sample_repo")
        .canonicalize()
        .expect("fixture repo should exist")
}

#[test]
fn index_creates_a_local_bundle() {
    let fixture = fixture_root();
    Command::cargo_bin("repointel")
        .expect("binary exists")
        .arg("index")
        .arg(&fixture)
        .arg("--json")
        .assert()
        .success()
        .stdout(contains("\"repo_name\""));
}

#[test]
fn symbol_reports_login_context() {
    let fixture = fixture_root();
    Command::cargo_bin("repointel")
        .expect("binary exists")
        .current_dir(&fixture)
        .arg("index")
        .arg(".")
        .assert()
        .success();

    Command::cargo_bin("repointel")
        .expect("binary exists")
        .current_dir(&fixture)
        .arg("symbol")
        .arg("login")
        .arg("--json")
        .assert()
        .success()
        .stdout(contains("\"qualified_name\""))
        .stdout(contains("login"));
}

#[test]
fn impact_reports_blast_radius() {
    let fixture = fixture_root();
    Command::cargo_bin("repointel")
        .expect("binary exists")
        .current_dir(&fixture)
        .arg("index")
        .arg(".")
        .assert()
        .success();

    Command::cargo_bin("repointel")
        .expect("binary exists")
        .current_dir(&fixture)
        .arg("impact")
        .arg("login")
        .arg("--json")
        .assert()
        .success()
        .stdout(contains("\"blast_radius_files\""));
}
