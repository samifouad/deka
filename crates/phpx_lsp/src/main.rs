#[tokio::main]
async fn main() {
    if let Err(err) = phpx_lsp::run_stdio().await {
        eprintln!("[phpx_lsp] fatal: {}", err);
        std::process::exit(1);
    }
}
