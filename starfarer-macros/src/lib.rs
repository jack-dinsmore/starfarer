#![allow(clippy::unused_io_amount)]

mod tools;
mod common;

use std::path::Path;
use syn::{parse_macro_input, LitStr, LitInt, Token};
use syn::parse::{Parse, ParseStream, Result};
use quote::quote;
use tools::{load_obj, load_font_to_binary};
use proc_macro::TokenStream;

extern crate proc_macro;

struct IncludeModel {
    sources: Vec<String>,
    output: Vec<u8>,
}

struct IncludeFont {
    sources: Vec<String>,
    output: (Vec<u8>, Vec<u8>),
}

impl Parse for IncludeModel {
    fn parse(input: ParseStream) -> Result<Self> {
        let path_lit = input.parse::<LitStr>()?;
        let head_name = path_lit.value();
        let model_name = std::path::Path::new(&head_name).file_name().unwrap().to_str().unwrap();
        let obj_path = format!("{}/{}.obj", head_name, model_name);
        let mtl_path = format!("{}/{}.mtl", head_name, model_name);

        let output = match load_obj(Path::new(&obj_path[6..]), Path::new(&mtl_path[6..])) {
            Ok(h) => h,
            Err(_) => panic!("Could not find object {} or {}", obj_path, mtl_path),
        };
        let output = bincode::serialize(&output).unwrap();
        Ok(Self {
            sources: vec![obj_path, mtl_path],
            output,
        })
    }
}

impl Parse for IncludeFont {
    fn parse(input: ParseStream) -> Result<Self> {
        let path_lit = input.parse::<LitStr>()?;
        let path_str = path_lit.value();

        input.parse::<Token![,]>()?;

        let size = input.parse::<LitInt>()?;
        
        let output = load_font_to_binary(Path::new(&format!("src/{}", path_str)), size.base10_parse()?);

        Ok(Self {
            sources: vec![path_str],
            output,
        })
    }
}

impl IncludeModel {
    pub fn expand(self) -> TokenStream {
        let Self { sources, output } = self;

        let expanded = quote! {
            {
                #({ const _FORCE_DEP: &[u8] = include_bytes!(#sources); })*
                &[#(#output),*]
            }
        };
        TokenStream::from(expanded)
    }
}

impl IncludeFont {
    pub fn expand(self) -> TokenStream {
        let Self { sources, output } = self;
        let (img_bin, kern_bin) = output;

        let expanded = quote! {
            {
                #({ const _FORCE_DEP: &[u8] = include_bytes!(#sources); })*
                ( &[#(#img_bin),*], &[#(#kern_bin),*], )
            }
        };
        TokenStream::from(expanded)
    }
}

#[proc_macro]
pub fn include_model(tokens: TokenStream) -> TokenStream {
    let ship: IncludeModel = parse_macro_input!(tokens as IncludeModel);
    ship.expand()
}

#[proc_macro]
pub fn include_font(tokens: TokenStream) -> TokenStream {
    let font: IncludeFont = parse_macro_input!(tokens as IncludeFont);
    font.expand()
}