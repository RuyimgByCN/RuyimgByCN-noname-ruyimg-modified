//! Line-oriented acceptance harness for the actual native host, using unchanged Worker messages.
use desktop_client::NativeEngineHost;
use std::io::{self, BufRead, Write};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = NativeEngineHost::new()?;
    let mut stdout = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let request = serde_json::from_str(&line?)?;
        let response = host.request(request).await?;
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }
    Ok(())
}
