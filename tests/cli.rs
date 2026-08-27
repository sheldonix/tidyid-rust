#![forbid(unsafe_code)]

use std::process::Command;

use tidyid::is_valid_id;

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tidyid"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn generates_default_custom_and_uppercase_ids() {
    for (arguments, length, uppercase) in [
        (&[][..], 32, false),
        (&["--size", "16"][..], 16, false),
        (&["-s", "3"][..], 3, false),
        (&["-u", "-s", "64"][..], 64, true),
    ] {
        let output = run(arguments);
        assert!(output.status.success());
        let value = String::from_utf8(output.stdout).unwrap();
        assert!(is_valid_id(value.trim(), Some(length), uppercase));
    }
}

#[test]
fn reports_metadata_and_invalid_arguments() {
    assert_eq!(
        String::from_utf8(run(&["--version"]).stdout)
            .unwrap()
            .trim(),
        "2.1.1"
    );
    assert!(
        String::from_utf8(run(&["--help"]).stdout)
            .unwrap()
            .contains("--allow-uppercase")
    );
    let invalid = run(&["--size", "2"]);
    assert!(!invalid.status.success());
    assert!(
        String::from_utf8(invalid.stderr)
            .unwrap()
            .contains("between 3 and 256")
    );
    let unknown = run(&["--unknown"]);
    assert!(!unknown.status.success());
    assert!(
        String::from_utf8(unknown.stderr)
            .unwrap()
            .contains("Unknown argument")
    );
}
