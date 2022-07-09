mod tools;
mod common;

use std::path::Path;
use syn::{parse_macro_input, LitStr, LitInt, Token};
use syn::parse::{Parse, ParseStream, Result};
use quote::quote;
use tools::{struct_as_bytes, read_as_bytes, load_obj, load_font_to_binary};
use proc_macro::TokenStream;

extern crate proc_macro;

struct IncludeShip {
    sources: Vec<String>,
    output: Vec<u8>,
}

struct IncludeFont {
    sources: Vec<String>,
    output: (Vec<u8>, Vec<u8>),
}

impl Parse for IncludeShip {
    fn parse(input: ParseStream) -> Result<Self> {
        let path_lit = input.parse::<LitStr>()?;
        let head_name = path_lit.value();
        let info_path = format!("{}.dat", head_name);
        let obj_path = format!("{}.obj", head_name);
        let texture_path = format!("{}.png", head_name);

        let info_data = match read_as_bytes(&Path::new(&format!("src/{}", info_path))) {
            Ok(o) => o,
            Err(_) => panic!("Could not find info path {}", info_path)
        };
        let texture_data = match read_as_bytes(&Path::new(&format!("src/{}", texture_path))) {
            Ok(o) => o,
            Err(_) => panic!("Could not find texture path {}", texture_path)
        };
        let (vertices, indices) = match load_obj(&Path::new(&format!("src/{}", obj_path))) {
            Ok(o) => o,
            Err(_) => panic!("Could not find object path {}", obj_path)
        };

        let info_size = info_data.len() as u64;
        let texture_size = texture_data.len() as u64;
        let num_indices = indices.len() as u64;
        let intro_bytes = [info_size, texture_size, num_indices];
        let vertices_bytes = vertices.iter().map(|v| { struct_as_bytes(v)}).collect::<Vec<&[u8]>>().concat();
        let indices_bytes = indices.iter().map(|i| { struct_as_bytes(i)}).collect::<Vec<&[u8]>>().concat();

        let output = [
            struct_as_bytes(&intro_bytes),
            &info_data[..],
            &texture_data[..], 
            &vertices_bytes[..],
            &indices_bytes[..],
        ].concat();

        Ok(Self {
            sources: vec![info_path, obj_path, texture_path],
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

impl IncludeShip {
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
pub fn include_ship(tokens: TokenStream) -> TokenStream {
    let ship: IncludeShip = parse_macro_input!(tokens as IncludeShip);
    ship.expand()
}

#[proc_macro]
pub fn include_font(tokens: TokenStream) -> TokenStream {
    let font: IncludeFont = parse_macro_input!(tokens as IncludeFont);
    font.expand()
}