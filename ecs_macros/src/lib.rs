use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics ::ecs::Component for #name #ty_generics #where_clause {}
    })
}

#[proc_macro_derive(ComponentBundle)]
pub fn component_bundle_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (field_registrations, field_components) = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields_named) => {
                let fields: Vec<_> = fields_named.named.iter().collect();

                let registrations = fields
                    .iter()
                    .map(|field| {
                        let ty = &field.ty;
                        quote! {
                            world.ensure_component_registered::<#ty>();
                        }
                    })
                    .collect::<Vec<_>>();

                let components = fields
                    .iter()
                    .map(|field| {
                        let field_name = &field.ident;
                        quote! {
                            components.push(Box::new(self.#field_name) as Box<dyn std::any::Any>);
                        }
                    })
                    .collect::<Vec<_>>();

                (registrations, components)
            }
            _ => {
                return TokenStream::from(quote! {
                    compile_error!("ComponentBundle derive only supports structs with named fields");
                });
            }
        },
        _ => {
            return TokenStream::from(quote! {
                compile_error!("ComponentBundle derive only supports structs");
            });
        }
    };

    TokenStream::from(quote! {
        impl #impl_generics ::ecs::ComponentBundle for #name #ty_generics #where_clause {
            fn register_components(world: &mut ::ecs::World) {
                #(#field_registrations)*
            }

            fn into_components(self) -> Vec<Box<dyn std::any::Any>> {
                let mut components = Vec::new();
                #(#field_components)*
                components
            }
        }
    })
}
