use assert_cmd::Command;
use indoc::indoc;

#[test]
fn test_cli() {
    let input = indoc! {"
        2
        1 1
        -1 0
    "};
    let expected = indoc! {"
        2
        -1
    "};

    Command::cargo_bin("a_plus_b").unwrap()
        .write_stdin(input)
        .assert()
        .stdout(expected);
}
