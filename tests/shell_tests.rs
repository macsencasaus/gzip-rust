use std::process::Command;
use std::str;

#[test]
fn stdin() {
    test_script("./tests/stdin.sh")
}

#[test]
fn reproducible() {
    test_script("./tests/reproducible.sh")
}

fn test_script(sh_script_path: &str) {
    let output = Command::new("bash")
        .arg(sh_script_path)
        .output()
        .expect("Failed to execute test script");

    let stdout = str::from_utf8(&output.stdout).unwrap_or("Unable to decode stdout");
    let stderr = str::from_utf8(&output.stderr).unwrap_or("Unable to decode stderr");

    if !output.status.success() {
        println!("Test failed with exit code: {:?}", output.status.code());
        println!("Stdout:\n{}", stdout);
        println!("Stderr:\n{}", stderr);
        panic!("Test failed");
    }
}
