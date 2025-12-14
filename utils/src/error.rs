base_infra::gen_impl_code_enum! {
	UtlErr {
		BigDecToF32= ("BGN001", "Failed to convert BigDecimal to f32"),
		BigDecToF64= ("BGN002", "Failed to convert BigDecimal to f64"),

		// date
		StrToNaiveDt = ("CHR001", "Failed to parse NaiveDateTime from string "),
		TimestampToDate = ("CHR002", "Failed to parse DateTime from timestamp"),
		LocalDtNotExistDstGap = ("CHR003", "local time does not exist (DST gap)"),
		TruncateDateTime = ("CHR004", "Valid DateTime when truncating to "),

	}
}
