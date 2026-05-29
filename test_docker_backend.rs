use std::process::Command;

fn main() {
    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run").arg("--rm");
    docker_cmd.arg("-e").arg("TEST_VAR=test_value");
    docker_cmd.arg("alpine:latest");
    docker_cmd.arg("sh");
    docker_cmd.arg("-c");
    docker_cmd.arg("echo $TEST_VAR");
    
    let output = docker_cmd.output().unwrap();
    println!("out: '{}'", String::from_utf8_lossy(&output.stdout));
}
