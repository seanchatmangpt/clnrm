use lsp_max::lsp_types::*;
use lsp_max::{Client, LanguageServer};
use std::sync::Mutex;

pub struct ClnrmBackend {
    pub client: Client,
}

#[lsp_max::async_trait]
impl LanguageServer for ClnrmBackend {
    async fn initialize(
        &self,
        _params: InitializeParams,
    ) -> lsp_max::jsonrpc::Result<InitializeResult> {
        let _ = lsp_max::get_registry();
        lsp_max::MESH.get_or_init(|| Mutex::new(lsp_max::max_runtime::AutonomicMesh::default()));

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "clnrm-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> lsp_max::jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let content = params.text_document.text.clone();
        self.diagnose(uri, content).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            self.diagnose(uri, change.text).await;
        }
    }
}

impl ClnrmBackend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn diagnose(&self, uri: Url, _content: String) {
        use lsp_max::lsp_types::Diagnostic;
        // Populate diagnostics below. Every diagnostic must carry:
        //   code: Some(NumberOrString::String("YOUR-CODE".to_string()))
        //   data: Some(serde_json::json!({"source_id": "clnrm-lsp"}))
        let diags: Vec<Diagnostic> = Vec::new();
        self.client.publish_diagnostics(uri, diags, None).await;
    }
}
