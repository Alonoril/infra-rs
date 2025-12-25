use crate::map_err;
use crate::result::AppResult;
use bincode::{Decode, config, de, enc};
use tracing::debug;

crate::gen_impl_code_enum! {
	BinErr {
		BinEncodeErr = ("BIN001", "Bincode encode error"),
		BinDecodeErr = ("BIN002", "Bincode decode error"),
	}
}

pub trait BinEncodeExt {
	fn bin_encode(&self) -> AppResult<Vec<u8>>;
}

impl<E: enc::Encode> BinEncodeExt for E {
	fn bin_encode(&self) -> AppResult<Vec<u8>> {
		bincode::encode_to_vec(self, config::standard()).map_err(map_err!(&BinErr::BinEncodeErr))
	}
}

pub trait BinDecodeExt {
	fn bin_decode_len<D: de::Decode<()>>(&self) -> AppResult<(D, usize)>;
	fn bin_decode<D: de::Decode<()>>(&self) -> AppResult<D> {
		let (data, len): (D, usize) = self.bin_decode_len::<D>()?;
		debug!("BinDecode with len {}", len);
		Ok(data)
	}
}

impl BinDecodeExt for &[u8] {
	fn bin_decode_len<D: Decode<()>>(&self) -> AppResult<(D, usize)> {
		let res: (D, usize) = bincode::decode_from_slice(self, config::standard())
			.map_err(map_err!(&BinErr::BinDecodeErr))?;
		Ok(res)
	}
}

impl BinDecodeExt for Vec<u8> {
	fn bin_decode_len<D: Decode<()>>(&self) -> AppResult<(D, usize)> {
		(&self[..]).bin_decode_len::<D>()
	}
}

#[cfg(test)]
mod tests {
	use crate::codec::bincode::{BinDecodeExt, BinEncodeExt};
	use bincode::{Decode, Encode, config};

	#[derive(Encode, Decode, PartialEq, Debug)]
	struct Entity {
		x: f32,
		y: f32,
	}

	#[derive(Encode, Decode, PartialEq, Debug)]
	struct World(Vec<Entity>);

	#[test]
	fn test_encode_decode() {
		let config = config::standard();
		let world = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);

		let encoded = world.bin_encode().unwrap();
		assert_eq!(encoded.len(), 1 + 4 * 4);

		let (decoded, len): (World, usize) = encoded.bin_decode_len().unwrap();
		assert_eq!(world, decoded);
		assert_eq!(len, encoded.len()); // read all bytes
	}
}
