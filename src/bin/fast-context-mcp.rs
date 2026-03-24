#[cfg(feature = "mcp")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use fast_context::mcp_server::FastContextMcpServer;
    use rmcp::{transport::stdio, ServiceExt};

    FastContextMcpServer::new()
        .serve(stdio())
        .await?
        .waiting()
        .await?;

    Ok(())
}

#[cfg(not(feature = "mcp"))]
fn main() {
    eprintln!("The fast-context-mcp binary requires the `mcp` feature.");
    std::process::exit(1);
}
