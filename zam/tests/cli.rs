use std::process::Command;

#[test]
fn invalid_arguments_fail_before_loading_or_writing_projects() {
    for args in [
        vec!["check", "--unknown"],
        vec!["build", "--unknown"],
        vec!["fmt", "--unknown"],
        vec!["run", "--unknown"],
        vec!["new", "--unknown"],
        vec!["check", ".", "extra"],
        vec!["new", "demo", "--template", "other"],
        vec!["--version", "extra"],
        vec!["--json", "check"],
        vec!["check", "--json", "--json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("usage:"), "{args:?}: {error}");
    }
}
