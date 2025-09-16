use base_infra::aprintln;
use tokio::io;

// Usage example
#[tokio::main]
async fn main() -> io::Result<()> {
    aprintln!();

    aprintln!("Hello from async println!");

    aprintln!("Formatted message: {} + {} = {}", 1, 2, 3)?;

    Ok(())
}
