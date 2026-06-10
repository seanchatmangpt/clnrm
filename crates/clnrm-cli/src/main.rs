//! Cleanroom CLI implementation

mod commands;
mod doctor;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "self-test" {
        #[derive(clap::Parser, Debug)]
        #[command(name = "clnrm self-test")]
        struct SelfTestArgs {
            #[arg(short, long)]
            suite: Option<String>,

            #[arg(short, long)]
            report: bool,

            #[arg(long, default_value = "none")]
            otel_exporter: String,

            #[arg(long)]
            otel_endpoint: Option<String>,
        }

        use clap::Parser;
        let mut sub_args = vec![args[0].clone()];
        sub_args.extend_from_slice(&args[2..]);

        let parsed = match SelfTestArgs::try_parse_from(sub_args) {
            Ok(p) => p,
            Err(e) => {
                e.exit();
            }
        };

        let result = clnrm_core::cli::commands::run_self_tests(
            parsed.suite,
            parsed.report,
            parsed.otel_exporter,
            parsed.otel_endpoint,
        )
        .await;

        match result {
            Ok(_) => {
                // run_self_tests already displays success to stderr
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }
    } else {
        clap_noun_verb::run().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
