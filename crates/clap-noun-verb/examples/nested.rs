//! Nested noun-verb CLI example demonstrating compound commands

use clap_noun_verb::{noun, run_cli, verb, Result, VerbArgs};

fn main() -> Result<()> {
    run_cli(|cli| {
        cli.name("nested-demo")
            .about("Demonstrates nested noun-verb CLI patterns")
            .noun(noun!("dev", "Development tools", {
                noun!("test", "Testing utilities", [
                    verb!("run", "Run tests", |_args: &VerbArgs| {
                        println!("Running tests...");
                        Ok(())
                    }),
                    verb!("watch", "Watch for changes and rerun tests", |_args: &VerbArgs| {
                        println!("Watching for test changes...");
                        Ok(())
                    }),
                    verb!("coverage", "Generate test coverage report", |_args: &VerbArgs| {
                        println!("Generating coverage report...");
                        Ok(())
                    }),
                ]),
                noun!("lint", "Code linting tools", [
                    verb!("check", "Check code style", |_args: &VerbArgs| {
                        println!("Checking code style...");
                        Ok(())
                    }),
                    verb!("fix", "Auto-fix linting issues", |_args: &VerbArgs| {
                        println!("Auto-fixing linting issues...");
                        Ok(())
                    }),
                ]),
                noun!("format", "Code formatting tools", [
                    verb!("check", "Check formatting", |_args: &VerbArgs| {
                        println!("Checking code formatting...");
                        Ok(())
                    }),
                    verb!("apply", "Apply formatting", |_args: &VerbArgs| {
                        println!("Applying code formatting...");
                        Ok(())
                    }),
                ]),
            }))
            .noun(noun!("ai", "AI-powered development tools", {
                noun!("orchestrate", "AI test orchestration", [
                    verb!("run", "Run AI-orchestrated tests", |_args: &VerbArgs| {
                        println!("Running AI-orchestrated tests...");
                        Ok(())
                    }),
                    verb!("predict", "Predict test failures", |_args: &VerbArgs| {
                        println!("Predicting potential test failures...");
                        Ok(())
                    }),
                    verb!("optimize", "Optimize test execution", |_args: &VerbArgs| {
                        println!("Optimizing test execution...");
                        Ok(())
                    }),
                ]),
                noun!("analyze", "AI-powered analysis", [
                    verb!("performance", "Analyze performance bottlenecks", |_args: &VerbArgs| {
                        println!("Analyzing performance bottlenecks...");
                        Ok(())
                    }),
                    verb!("quality", "Analyze code quality", |_args: &VerbArgs| {
                        println!("Analyzing code quality...");
                        Ok(())
                    }),
                ]),
                noun!("monitor", "AI monitoring", [
                    verb!("start", "Start AI monitoring", |_args: &VerbArgs| {
                        println!("Starting AI monitoring...");
                        Ok(())
                    }),
                    verb!("status", "Check monitoring status", |_args: &VerbArgs| {
                        println!("Monitoring status: Active");
                        Ok(())
                    }),
                ]),
            }))
    })
}
