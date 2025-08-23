use base_infra::aprintln;
use tokio::io;

// 使用示例
#[tokio::main]
async fn main() -> io::Result<()> {
    aprintln!();

    aprintln!("Hello from async println!");

    aprintln!("Formatted message: {} + {} = {}", 1, 2, 3)?;

    Ok(())
}
