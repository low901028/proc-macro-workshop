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
    ///                             TypePath {      # 类型路径: 类似A::B::C
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

    // 获取当前struct中所有fields: DataStruct::fields::FieldsNamed部分
    let data: FieldsNamed = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(n),
            ..
        }) => n,
        other => unimplemented!("{:?}", other),
    };

    // 遍历每个命名的field：名称、类型; 构建(名称, 类型Option<真实类型>)
    let fields = data.named.iter().filter_map(|field| {
        let ty = &field.ty;
        match &field.ident {
            Some(ident) => Some((ident, ty, inner_for_option(ty))),
            _ => None,
        }
    });

    // 遍历每个field：名称、类型; 构建(名称, Option<真实类型>)
    let names = data.named.iter().filter_map(|field| match &field.ident {
        None => None,
        Some(ident) => Some((ident, inner_for_option(&field.ty))),
    });

    // 构建每个字段的初始化值:None; 类似 字段name: None
    let initialize = names.clone().map(|(name, _)| quote! { #name: None });

    //
    let extract = names.clone().map(|(name, option)| match option {
        None => quote! { #name: self.#name.clone()? },
        Some(_) => quote! { #name: self.#name.clone() },
    });

    // 构建所有字段的模版： 类似 字段name： 类型Option<#ty>
    let quoted_fields = fields.clone().map(|(name, ty, option)| match option {
        None => quote! { #name: Option<#ty> },
        Some(ty) => quote! { #name: Option<#ty> },
    });

    // 构建每个字段对应的setter方法
    // 格式类似：
    // pub fn 字段名称(&mut self, 值value: 类型#ty) {
    //  self.#name = Some(value);
    //  self
    // }
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

    // 构建最终的Builder模式的对应的模版
    let expanded = quote! {
        // 生成Builder，并初始化struct不同字段的内容
        impl #name {
            fn builder() -> #builder {
                #builder {
                    #(
                        #initialize,
                    )*
                }
            }
        }

        // 定义Builder
        struct #builder {
            #(
                #quoted_fields,
            )*
        }

        // 实现Builder的build及不同字段赋值的方法
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

    // 最终输出proc_macro::TokenStream,并入被编译器rustc输出AST
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
