use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};

#[proc_macro_attribute]
pub fn axum_resp(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(input as ItemFn);

    // 1. 解析返回类型 AppResult<T>
    let inner_ty = {
        let output = match &func.sig.output {
            ReturnType::Type(_, ty) => ty,
            _ => {
                return syn::Error::new_spanned(
                    &func.sig.output,
                    "axum_resp requires a return type like AppResult<T>",
                )
                .to_compile_error()
                .into();
            }
        };

        match unwrap_app_result(output) {
            Ok(inner) => inner,
            Err(err) => return err.to_compile_error().into(),
        }
    };

    // 2. 修改返回类型为 AxumResult<impl IntoResponse>
    func.sig.output = syn::parse_quote! {
        -> ::web_infra::result::AxumResult<impl ::axum::response::IntoResponse>
    };

    // 3. 包装函数体
    let block = func.block;
    func.block = syn::parse_quote!({
        let res: #inner_ty = (async #block).await?;
        ::web_infra::success!(res)
    });

    // 输出
    TokenStream::from(quote! {
        #func
    })
}

// --- 工具函数：解析 AppResult<T> ---
fn unwrap_app_result(ty: &Type) -> Result<Type, syn::Error> {
    match ty {
        Type::Path(tp) => {
            let seg = tp.path.segments.last().unwrap();
            if seg.ident != "AppResult" {
                return Err(syn::Error::new_spanned(
                    ty,
                    "Return type must be AppResult<T>",
                ));
            }
            match &seg.arguments {
                syn::PathArguments::AngleBracketed(ab) => {
                    if ab.args.len() != 1 {
                        return Err(syn::Error::new_spanned(
                            ab,
                            "AppResult<T> must have exactly one generic parameter",
                        ));
                    }
                    let inner_ty = ab.args.first().unwrap();
                    if let syn::GenericArgument::Type(t) = inner_ty {
                        Ok(t.clone())
                    } else {
                        Err(syn::Error::new_spanned(inner_ty, "Invalid generic type"))
                    }
                }
                _ => Err(syn::Error::new_spanned(
                    seg,
                    "AppResult<T> must have generic parameter",
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(
            ty,
            "Return type must be AppResult<T>",
        )),
    }
}
