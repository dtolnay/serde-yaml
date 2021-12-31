use crate::ast::{Enum, Field, Input, Struct, Variant};
use crate::attr::{Comment, Format};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Index, Member, Result};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(node)?;

    Ok(match input {
        Input::Struct(input) => impl_struct(input),
        Input::Enum(input) => impl_enum(input),
    })
}

fn impl_field_format(fields: &[Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|f| {
            let format = match &f.attrs.format {
                Format::None => quote! { None },
                Format::Block => quote! { Some(Format::Block) },
                Format::Binary => quote! { Some(Format::Binary) },
                Format::Decimal => quote! { Some(Format::Decimal) },
                Format::Hex => quote! { Some(Format::Hex) },
                Format::Octal => quote! { Some(Format::Octal) },
            };
            match &f.member {
                Member::Named(id) => {
                    let id = id.to_string();
                    quote! { MemberId::Name(#id) => #format }
                }
                Member::Unnamed(Index { index: i, .. }) => {
                    quote! { MemberId::Index(#i) => #format }
                }
            }
        })
        .collect::<Vec<_>>()
}

fn impl_field_comment(fields: &[Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|f| {
            let comment = match &f.attrs.comment {
                Comment::None => quote! { None },
                Comment::Static(s) => quote! {
                    Some(#s.to_string())
                },
                Comment::Field(id) => quote! {
                    Some(self.#id.to_string())
                },
                Comment::Function(id) => quote! {
                    self.#id()
                },
            };
            match &f.member {
                Member::Named(id) => {
                    let id = id.to_string();
                    quote! { MemberId::Name(#id) => #comment }
                }
                Member::Unnamed(Index { index: i, .. }) => {
                    quote! { MemberId::Index(#i) => #comment }
                }
            }
        })
        .collect::<Vec<_>>()
}

fn impl_struct(input: Struct) -> TokenStream {
    let formats = impl_field_format(&input.fields);
    let comments = impl_field_comment(&input.fields);
    let name = &input.ident;
    let namestr = name.to_string();
    quote! {
        const _: () = {
            extern crate serde_yaml;
            extern crate inventory;
            use serde_yaml::yamlformat::{YamlFormat, YamlFormatType, Format, MemberId};

            impl YamlFormat for #name {
                fn format(&self, field: &MemberId) -> Option<Format> {
                    match field {
                        #(#formats),*,
                        _ => None,
                    }
                }
                fn comment(&self, field: &MemberId) -> Option<String> {
                    match field {
                        #(#comments),*,
                        _ => None,
                    }
                }
            }
            impl #name {
                fn __type_id() -> usize {
                    YamlFormatType::of::<Self>()
                }
                unsafe fn __reconstitute(ptr: *const ()) -> &'static dyn YamlFormat {
                    YamlFormatType::cast::<Self>(ptr)
                }
            }
            inventory::submit! {
                YamlFormatType {
                    id: #name::__type_id,
                    reconstitute: #name::__reconstitute,
                }
            }
        };
    }
}

fn impl_enum(input: Enum) -> TokenStream {
    println!("{:?}", input);
    quote! {}
}
