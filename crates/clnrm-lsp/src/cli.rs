use crate::backend::ClnrmBackend;
use clap_noun_verb_macros::verb;
use lsp_max::{LspService, Server};

#[verb("serve")]
pub fn cmd_serve(stdio: bool) -> clap_noun_verb::Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        if stdio {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();
            let (service, socket) = LspService::new(ClnrmBackend::new);
            Server::new(stdin, stdout, socket)
                .serve(service)
                .await
                .unwrap();
        }
    });
    Ok(())
}

pub fn main() -> clap_noun_verb::Result<()> {
    clap_noun_verb::run()
}
