use darling::{ast, util, FromDeriveInput, FromVariant};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Error Data From Derive Input - Enum
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct ErrorData {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<EnumVariants, ()>,
    app_type: syn::Type,
    prefix: String,
}

/// Enum Variants
#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct EnumVariants {
    ident: syn::Ident,
    fields: ast::Fields<util::Ignored>,
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

/// 处理生成ErrorInfo的逻辑
/// 注意拆开代码片段
/// 注意处理匹配函数
pub(crate) fn process_error_info(input: DeriveInput) -> TokenStream {
    let ErrorData {
        ident: name,
        generics,
        data: ast::Data::Enum(data),
        app_type,
        prefix,
    } = ErrorData::from_derive_input(&input).expect("Can Not Parse Input")
    else {
        panic!("ErrorInfo derive only supports enums")
    };

    let code = data
        .iter()
        .map(|v| {
            let EnumVariants {
                ident,
                fields,
                code,
                app_code,
                client_msg,
            } = v;
            // 前缀+Code 生成错误码
            let code = format!("{}{}", prefix, code);
            // 根据变量类型生成不同的匹配代码
            // 匹配模式中的 Err(_) 这部分
            let variant_code = match fields.style {
                ast::Style::Unit => quote! { #name::#ident},
                ast::Style::Tuple => {
                    quote! { #name::#ident(_) }
                }
                ast::Style::Struct => {
                    quote! {#name::#ident{..}}
                }
            };
            // 这里补齐 匹配内容 生成新的ErrorInfo
            // ErrorInfo::new(app_code, code, client_msg, self)
            quote! {
                #variant_code=>{
                    ErrorInfo::new(
                        #app_code,
                        #code,
                        #client_msg,
                        self
                    )
                }
            }
        })
        .collect::<Vec<_>>();

    // 生成最终的impl部分
    quote! {
        // 导入相关类型和trait 这里需要注意的是 crate的名字和路径
        use error_code::{ErrorInfo,ToErrorInfo as _};

        // 这里参照ToErrorInfo的trait定义
        impl #generics ToErrorInfo for #name #generics {
            // 最终的输出类型，由关联类型决定是最佳实践
            // 这里我们实现时候对应的应该是 http::StatusCode
            type T = #app_type;
            // 实现Trait定义的函数ß
            fn to_error_info(&self) -> ErrorInfo<Self::T> {
                match self {
                    #(#code),*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_struct() {
        let input = r#"
        #[derive(thiserror::Error, ToErrorInfo)]
        #[error_info(app_type="http::StatusCode", prefix="01")]
        pub enum MyError {
        #[error("Invalid command: {0}")]
        #[error_info(code="IC", app_code="400")]
        InvalidCommand(String),

        #[error("Invalid argument: {0}")]
        #[error_info(code="IA", app_code="400", client_msg="friendly msg")]
        InvalidArgument(String),

        #[error("{0}")]
        #[error_info(code="RE", app_code="500")]
        RespError(#[from] RespError),
        }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = ErrorData::from_derive_input(&parsed).unwrap();
        println!("{:#?}", info);

        assert_eq!(info.ident.to_string(), "MyError");
        assert_eq!(info.prefix, "01");

        let code = process_error_info(parsed);

        println!("{}", code);
    }
}
