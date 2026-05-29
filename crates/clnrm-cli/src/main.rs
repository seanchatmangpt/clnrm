//! Cleanroom CLI implementation

mod commands;
mod doctor;

use clap_noun_verb::Result;

#[tokio::main]
async fn main() -> Result<()> {
    clap_noun_verb::run()
}
