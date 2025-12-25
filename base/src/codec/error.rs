crate::gen_impl_code_enum! {
	RkyvErr {
		EncodeWithArena = ("RKYV01", "Failed to decode with rkyv Arena"),
		DecodeToArchivedType = ("RKYV02", "Failed to decode to ArchivedType"),
		DeserFromArchived = ("RKYV03", "Failed to deserialize from rkyv ArchivedType"),
	}
}
