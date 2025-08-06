use std::fmt::{Debug, Display};

pub type DynErrCode = dyn ErrorCode + Send + Sync + 'static;

pub trait ErrorCode: Debug + Display + Sync + Send + 'static {
    fn code(&self) -> &'static str;
    fn message(&self) -> &'static str;
}

#[macro_export]
macro_rules! gen_impl_code_enum {
    (
        $(
            $(#[$enum_attr:meta])*
            $enum_name:ident {
                $(
                    $(#[$variant_attr:meta])*
                    $variant_name:ident = ($code:expr, $message:expr),
                )*
            }
        )*
    ) => {
        $(
            $(#[$enum_attr])*
            #[derive(Debug, Copy, Clone, PartialEq, Eq)]
            pub enum $enum_name {
                $(
                    $(#[$variant_attr])*
                    $variant_name,
                )*
            }

            impl $crate::result::ErrorCode for $enum_name {
                fn code(&self) -> &'static str {
                    match self {
                        $(
                            $enum_name::$variant_name => $code,
                        )*
                    }
                }

                fn message(&self) -> &'static str {
                    match self {
                        $(
                            $enum_name::$variant_name => $message,
                        )*
                    }
                }
            }

            impl std::fmt::Display for $enum_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    use $crate::result::ErrorCode;
                    write!(f, "{}: {}", self.code(), self.message())
                }
            }
        )*
    };
}

// resp_codes! {
//     ("000000", SUCCESS, "Success");
//     ("000001", SYSTEM_ERROR, "System error");
//     ("000002", INTERNAL_ERROR, "Internal error");
//     ("000003", REQ_JSON_ERR, "Error in the json payload");
//     ("CFG003", CONFIG_ERROR, "Config error");
//     ("AU0001", WRONG_CREDENTIALS, "wrong credentials");
//     ("AU0002", JWT_INVALID, "jwt token not valid");
//     ("AU0003", JWT_CREATION_ERR, "jwt token creation error");
//     ("AU0004", LOGIN_TIMEOUT, "Login timeout");
//     ("AU0005", INVALID_AUTH_HEADER, "invalid auth header");
//     ("AU0006", NO_PERMISSION, "no permission");
//     ("AU0007", LOGOUT_SUCCESS, "Logout success");
//     ("AU0008", USER_NOT_FOUND, "User not found");
//     ("DB0001", DATABASE_ERROR, "Database error");
// }
gen_impl_code_enum! {
    SysErr {
        Success = ("000000", "Success"),
        SystemError = ("000001", "System error"),
        InternalError = ("000002", "Internal error"),
        InvalidParams = ("000003", "Invalid parameters"),

        SerdeError = ("JSN000", "Serde error"),
        ReqJsonErr = ("JSN001", "Error in the json payload"),
        DeserializeErr = ("JSN002", "Deserialize with Serde error"),

        ConfigError = ("CFG000", "Config error"),
        NoCfgFile = ("CFG001", "Config path not specified"),
        ConfigLoadFailed = ("CFG002", "Config load failed"),

        MutexLockErr = ("MUTEX1", "Cannot currently handle a poisoned lock"),

        ServerBindErr = ("SVR001", "Server bind failed"),
        ServerStartErr = ("SVR002", "Server start failed"),

        SystemTimeError = ("TIME001", "System time error"),
    }
}

#[cfg(test)]
mod tests {
    use crate::result::{ErrorCode, SysErr};

    #[test]
    fn test() {
        let error = SysErr::SystemError;
        println!("Error Code: {}", error.code());
        println!("Error Message: {}", error.message());
        println!("Error: {}", error); // 通过 Display trait 打印
    }
}
