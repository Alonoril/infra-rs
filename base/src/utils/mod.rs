mod str_util;
pub mod time;
pub mod uuid;
pub mod vec_util;

use anyhow::ensure;
pub use str_util::*;


fn ensure_slice_len_eq(data: &[u8], len: usize) -> anyhow::Result<()> {
    let dlen = data.len();
    ensure!(dlen == len, "Unexpected data len {dlen}, expected {len}.",);
    Ok(())
}

fn ensure_slice_len_gt(data: &[u8], len: usize) -> anyhow::Result<()> {
    let dlen = data.len();
    ensure!(
		dlen > len,
		"Unexpected data len {dlen}, expected to be greater than {len}.",
	);
    Ok(())
}
