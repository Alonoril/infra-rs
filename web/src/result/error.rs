use base_infra::gen_impl_code_enum;

gen_impl_code_enum! {
    WebErr {
        AxumError = ("WEB000", "Axum server error"),
        MissingExtension = ("WEB001", "Axum server error: MissingExtension"),
        UserAgentNotFound = ("WEB002", "User-Agent header not found in request"),
        NotFound = ("WEB003", "The requested resource does not exist on this server!"),
        RequestTimeout = ("WEB004", "Request timeout"),
        InternalServerError = ("WEB005", "unhandled internal error"),

        ReqJsonErr = ("AXUM01", "Error in the json payload"),
        QueryParamsErr = ("AXUM02", ""),
    }
}
