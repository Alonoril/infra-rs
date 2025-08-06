use crate::gen_impl_code_enum;

gen_impl_code_enum! {
	BinErr {
		BinEncodeErr = ("BIN001", "Bincode encode error"),
		BinDecodeErr = ("BIN002", "Bincode decode error"),
	}
}
