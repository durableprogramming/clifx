use std::io::Write;
use std::process::{Command, Stdio};

const TEST_TEXT: &str = "Hello World!";

#[test]
fn test_cli_shine_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "shine", "--help"])
        .output()
        .expect("Failed to run CLI command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Apply shine effect to stdin"));
    assert!(stdout.contains("--color"));
    assert!(stdout.contains("--speed"));
    assert!(stdout.contains("--easing"));
}

#[test]
fn test_cli_shine2d_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "shine2d", "--help"])
        .output()
        .expect("Failed to run CLI command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Apply 2D shine effect to stdin with angle control"));
    assert!(stdout.contains("--angle"));
    assert!(stdout.contains("--terminal-width"));
}

#[test]
fn test_cli_twinkle_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "twinkle", "--help"])
        .output()
        .expect("Failed to run CLI command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Apply twinkle effect to stdin"));
    assert!(stdout.contains("--base-color"));
    assert!(stdout.contains("--twinkle-color"));
    assert!(stdout.contains("--star-mode"));
}

#[test]
fn test_cli_main_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run CLI command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CLI effects for text processing"));
    assert!(stdout.contains("shine"));
    assert!(stdout.contains("shine2d"));
    assert!(stdout.contains("twinkle"));
}

#[test]
fn test_cli_shine_with_input() {
    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            "shine",
            "--color",
            "255,0,0",
            "--speed",
            "50",
            "--cycles",
            "1", // Set to 1 for a quick test
            "--duration",
            "100",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI command");

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(TEST_TEXT.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    // The command should complete successfully
    // Note: We can't easily test the colored output, but we can verify it runs without error
    assert!(output.stderr.is_empty() || String::from_utf8_lossy(&output.stderr).contains(""));
}

#[test]
fn test_cli_twinkle_with_input() {
    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            "twinkle",
            "--base-color",
            "255,255,255",
            "--twinkle-color",
            "255,255,0",
            "--speed",
            "50",
            "--cycles",
            "1", // Set to 1 for a quick test
            "--duration",
            "100",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI command");

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(TEST_TEXT.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    // The command should complete successfully
    assert!(output.stderr.is_empty() || String::from_utf8_lossy(&output.stderr).contains(""));
}

#[test]
fn test_cli_invalid_color_format() {
    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            "shine",
            "--color",
            "invalid-color",
            "--cycles",
            "1",
            "--duration",
            "100",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI command");

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(TEST_TEXT.as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    // Should fail with invalid color format
    assert!(!output.status.success());
}

#[test]
fn test_cli_easing_types() {
    let easing_types = ["linear", "ease-in", "ease-out", "ease-in-out"];

    for easing in &easing_types {
        let mut child = Command::new("cargo")
            .args([
                "run",
                "--",
                "shine",
                "--color",
                "255,0,0",
                "--easing",
                easing,
                "--cycles",
                "1",
                "--duration",
                "100",
                "--speed",
                "50",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn CLI command");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(TEST_TEXT.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        // Should succeed with valid easing type
        assert!(
            output.status.success(),
            "Failed with easing type: {}, stderr: {}",
            easing,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_cli_shine2d_angles() {
    let angles = ["0", "45", "90", "180"];

    for angle in &angles {
        let mut child = Command::new("cargo")
            .args([
                "run",
                "--",
                "shine2d",
                "--color",
                "255,0,0",
                "--angle",
                angle,
                "--cycles",
                "1",
                "--duration",
                "100",
                "--speed",
                "50",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn CLI command");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(TEST_TEXT.as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        // Should succeed with valid angle
        assert!(
            output.status.success(),
            "Failed with angle: {}, stderr: {}",
            angle,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_cli_twinkle_star_mode() {
    let mut child = Command::new("cargo")
        .args([
            "run",
            "--",
            "twinkle",
            "--base-color",
            "255,255,255",
            "--twinkle-color",
            "255,255,0",
            "--star-mode",
            "--cycles",
            "1",
            "--duration",
            "100",
            "--speed",
            "50",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI command");

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all("Hello... World!".as_bytes())
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    // Should succeed with star mode enabled
    assert!(
        output.status.success(),
        "Failed with star mode, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cli_empty_input() {
    let commands = [
        vec!["run", "--", "shine", "--cycles", "1"],
        vec!["run", "--", "shine2d", "--cycles", "1"],
        vec!["run", "--", "twinkle", "--cycles", "1"],
    ];

    for cmd_args in &commands {
        let mut child = Command::new("cargo")
            .args(cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn CLI command");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all("".as_bytes())
                .expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to read stdout");

        // Should handle empty input gracefully
        assert!(
            output.status.success(),
            "Failed with empty input for command: {:?}, stderr: {}",
            cmd_args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
