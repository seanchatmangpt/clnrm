use std::process::Command;

fn main() {
    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-e")
        .arg("TEST_VAR=test_value")
        .arg("alpine:latest")
        .arg("sh")
        .arg("-c")
        .arg("echo $TEST_VAR")
        .output()
        .unwrap();
    println!("out: '{}'", String::from_utf8_lossy(&output.stdout));
}
