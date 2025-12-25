// impl rkyv::Deserialize<Value, HighDeserializer<rancor::Error>> for ArchivedValue {
// use rkyv::api::high::HighDeserializer;
// 	fn deserialize(
// 		&self,
// 		deserializer: &mut HighDeserializer<rancor::Error>,
// 	) -> Result<Value, rancor::Error> {
// 		todo!()
// 	}
// }

fn main() {}


// pub trait RkyvEncodeExt<T, A, E>
// where
// 	A: Allocator<E>,
// 	T: Serialize<HighSerializer<AlignedVec, A, T>>,
// {
// 	fn encode(&self) -> AppResult<Vec<u8>>;
// }
//
// impl<T, A, E> RkyvEncodeExt<T, A, E> for T
// where
// 	A: Allocator<E>,
// 	T: Serialize<HighSerializer<AlignedVec, A, T>>,
// 	// Self: Serialize<HighSerializer<AlignedVec, A, T>>,
// {
// 	fn encode(&self) -> AppResult<Vec<u8>> {
// 		let mut arena = Arena::new();
// 		let bytes = to_bytes_with_alloc::<_, Error>(self, arena.acquire())
// 			.map_err(map_err!(&RkyvErr::EncodeWithArena))?;
// 		Ok(bytes.into_vec())
// 	}
// }
//
// pub trait RkyvDecodeExt<T, E>
// where
// 	E: Source,
// 	T: Portable + for<'a> CheckBytes<HighValidator<'a, E>>,
// {
// 	fn decode(&self) -> AppResult<Self>
// 	where
// 		Self: Sized;
// }
//
// impl<T, E> RkyvDecodeExt<T, E> for &[u8]
// where
// 	E: Source,
// 	T: Portable + for<'a> CheckBytes<HighValidator<'a, E>>,
// {
// 	fn decode(&self) -> AppResult<Self> {
// 		let archived =
// 			rkyv::access::<T, Error>(self).map_err(map_err!(&RkyvErr::DecodeToArchivedType))?;
//
// 		deserialize::<Self, Error>(archived).map_err(map_err!(&RkyvErr::DeserFromArchived))
// 	}
// }
//
// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use rkyv_derive::{Archive, Deserialize, Serialize};
//
// 	#[derive(Clone, Debug, Default, PartialEq, Archive, Deserialize, Serialize)]
// 	pub struct Value {
// 		pub val0: f64,
// 		pub val1: Option<String>,
// 	}
//
// 	#[test]
// 	fn test_encode_decode() {
// 		let val = Value {
// 			val0: 1.2,
// 			val1: Some("rkyv".into()),
// 		};
// 		let bytes = val.encode().unwrap();
// 		let val2 = bytes.decode::<Value>().unwrap();
// 		assert_eq!(val, val2);
// 	}
// }