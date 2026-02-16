use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ComponentBundle)]
pub fn component_bundle_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let field_inits = if let Data::Struct(data) = input.data {
        match data.fields {
            Fields::Named(fields_named) => {
                let fields: Vec<_> = fields_named.named.iter().collect();
                fields.iter().map(|f| {
                    let field_name = &f.ident;
                    let ty = &f.ty;
                    quote! {
                        vec.push((
                            std::any::TypeId::of::<#ty>(),
                            Box::new(self.#field_name) as Box<dyn std::any::Any>
                        ));
                    }
                }).collect::<Vec<_>>()
            }
            _ => unimplemented!("Only named fields are supported"),
        }
    } else {
        unimplemented!("Only structs are supported")
    };

    let expanded = quote! {
        impl ComponentBundle for #name {
            fn into_components(self) -> Vec<(std::any::TypeId, Box<dyn std::any::Any>)> {
                let mut vec = Vec::new();
                #(#field_inits)*
                vec
            }
        }
    };

    TokenStream::from(expanded)
}
