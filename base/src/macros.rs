use tokio::io::{self, AsyncWriteExt};

/// 异步println宏
#[macro_export]
macro_rules! aprintln {
    () => {
        $crate::macros::async_println("").await
    };
    ($msg:expr) => {
        $crate::macros::async_println($msg).await
    };
    ($($arg:tt)*) => {
        $crate::macros::async_println(&format!($($arg)*)).await
    };
}

#[cfg(feature = "tokio")]
pub async fn async_println(msg: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(msg.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}
