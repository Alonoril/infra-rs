use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ReturnType, Type, TypePath, parse_macro_input};

#[proc_macro_attribute]
pub fn resp_data(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut fnc = parse_macro_input!(input as ItemFn);

    // 1. Resolve return type AppResult<T>
    let inner_ty = match parse_return_type(&fnc) {
        Ok(inner) => inner,
        Err(err) => return err.to_compile_error().into(),
    };

    // 2. Modify the return type to  AxumResult<impl IntoResponse>
    fnc.sig.output = syn::parse_quote! {
        -> ::web_infra::result::AxumResult<impl ::axum::response::IntoResponse>
    };

    // 3. Wrap the function body
    let block = fnc.block;
    fnc.block = syn::parse_quote!({
        let res: #inner_ty = (async #block).await?;
        ::web_infra::success!(res)
    });

    // output
    TokenStream::from(quote! {
        #fnc
    })
}

fn parse_return_type(fnc: &ItemFn) -> Result<Type, syn::Error> {
    let output = match &fnc.sig.output {
        ReturnType::Type(_, ty) => ty,
        _ => {
            return Err(syn::Error::new_spanned(
                &fnc.sig.output,
                "resp_data requires a return type like AppResult<T>",
            ));
        }
    };

    let output: &Type = output;
    match output {
        Type::Path(tp) => unwrap_app_result(output, tp),
        _ => Err(syn::Error::new_spanned(
            output,
            "Return type must be AppResult<T>",
        )),
    }
}

/// resolve AppResult<T> ---
fn unwrap_app_result(output: &Type, tp: &TypePath) -> Result<Type, syn::Error> {
    let segment = tp.path.segments.last().unwrap();
    if segment.ident != "AppResult" {
        return Err(syn::Error::new_spanned(
            output,
            "Return type must be AppResult<T>",
        ));
    }
    match &segment.arguments {
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
            segment,
            "AppResult<T> must have generic parameter",
        )),
    }
}
