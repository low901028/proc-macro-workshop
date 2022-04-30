extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed, Type};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    eprintln!("===input: {:#?}", input);
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // eprintln!("{:?}", format!("===ast: {}", input.into()));
    eprintln!("===ast input atrrs: {:#?}", input.attrs);
    eprintln!("===ast input name: {:#?}", input.ident);
    eprintln!("===ast input vis: {:#?}", input.vis);
    eprintln!("===ast input generics: {:#?}", input.generics);
    eprintln!("===ast input data: {:#?}", input.data);
    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // 类似：CommonBuilder格式
    let builder = format_ident!("{}Builder", name);

    /// 获取struct/enum/union的内容部分
    /// 类似struct的如下内容：DataStruct.data部分
    /// {
    ///     executable: String,
    ///     args: Vec<String>,
    ///     env: Vec<String>,
    ///     current_dir: String,
    /// }
    /// 经过使用syn库转换为syn.AST 大概格式如下：
    /// Struct(
    ///     DataStruct {
    ///         struct_token: Struct,           # 标识当前是一个struct
    ///         fields: Named(                  # 在当前struct中定义的field
    ///             FieldsNamed {               # 字段都是其命名的
    ///                 brace_token: Brace,
    ///                 named: [                # 命名field
    ///                     Field {             # 类似: executable: String,
    ///                         attrs: [],      # 该field在定义时指定了attributes
    ///                         vis: Inherited, # 该field的可见性
    ///                         ident: Some(    # 该field的名称及代码中位置
    ///                             Ident {
    ///                                 ident: "executable",  # 名称
    ///                                 span: #0 bytes(1016..1026), # 位置
    ///                             },
    ///                         ),
    ///                         colon_token: Some( # 冒号:
    ///                             Colon,
    ///                         ),
    ///                         ty: Path(           # 该field类型
    ///                             TypePath {
    ///                                 qself: None,
    ///                                 path: Path {
    ///                                     leading_colon: None,
    ///                                     segments: [
    ///                                         PathSegment {
    ///                                             ident: Ident {
    ///                                                 ident: "String",
    ///                                                 span: #0 bytes(1028..1034),
    ///                                             },
    ///                                             arguments: None,
    ///                                         },
    ///                                     ],
    ///                                 },
    ///                             },
    ///                         ),
    ///                     },
    ///                     Comma,
    ///                 ],
    ///             },
    ///         ),
    ///         semi_token: None,
    ///     },
    /// )
    let data: FieldsNamed = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(n),
            ..
        }) => n,
        other => unimplemented!("{:?}", other),
    };

    let fields = data.named.iter().filter_map(|field| {
        let ty = &field.ty;
        match &field.ident {
            Some(ident) => Some((ident, ty, inner_for_option(ty))),
            _ => None,
        }
    });

    let names = data.named.iter().filter_map(|field| match &field.ident {
        None => None,
        Some(ident) => Some((ident, inner_for_option(&field.ty))),
    });

    let initialize = names.clone().map(|(name, _)| quote! { #name: None });

    let extract = names.clone().map(|(name, option)| match option {
        None => quote! { #name: self.#name.clone()? },
        Some(_) => quote! { #name: self.#name.clone() },
    });

    let quoted_fields = fields.clone().map(|(name, ty, option)| match option {
        None => quote! { #name: Option<#ty> },
        Some(ty) => quote! { #name: Option<#ty> },
    });

    let methods = fields.clone().map(|(name, ty, option)| match option {
        None => quote! {
            pub fn #name(&mut self, value: #ty) -> &mut Self {
                self.#name = Some(value);
                self
            }
        },

        Some(ty) => quote! {
            pub fn #name(&mut self, value: #ty) -> &mut Self {
                self.#name = Some(value);
                self
            }
        },
    });

    let expanded = quote! {
        impl #name {
            fn builder() -> #builder {
                #builder {
                    #(
                        #initialize,
                    )*
                }
            }
        }

        struct #builder {
            #(
                #quoted_fields,
            )*
        }

        impl #builder {
            pub fn build(&self) -> Option<#name> {
                Some(#name {
                    #(
                        #extract,
                    )*
                })
            }

            #(
                #methods
            )*
        }
    };
    eprintln!("===expanded: {:#?}", expanded);
    TokenStream::from(expanded)
}

fn inner_for_option(ty: &Type) -> Option<Type> {
    match ty {
        Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) if segments[0].ident == "Option" => {
            let segment = &segments[0];

            match &segment.arguments {
                syn::PathArguments::AngleBracketed(generic) => {
                    match generic.args.first().unwrap() {
                        syn::GenericArgument::Type(ty) => Some(ty.clone()),
                        _ => None,
                    }
                }
                _ => None,
            }
        }

        _ => None,
    }
}
