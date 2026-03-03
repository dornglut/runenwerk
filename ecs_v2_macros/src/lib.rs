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
        impl #impl_generics ::ecs_v2::Component for #name #ty_generics #where_clause {}
    })
}

#[proc_macro_derive(Bundle)]
pub fn bundle_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let Data::Struct(data) = input.data else {
        return TokenStream::from(quote! {
            compile_error!("Bundle derive only supports structs");
        });
    };

    let Fields::Named(fields) = data.fields else {
        return TokenStream::from(quote! {
            compile_error!("Bundle derive only supports structs with named fields");
        });
    };

    let registrations = fields.named.iter().map(|field| {
        let ty = &field.ty;
        quote! {
            world.__register_component::<#ty>();
        }
    });

    let inserts = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("named field");
        quote! {
            world.__insert_component(entity, self.#field_name)?;
        }
    });

    let removals = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("named field");
        let ty = &field.ty;
        quote! {
            #field_name: world.__remove_component::<#ty>(entity)?
        }
    });

    TokenStream::from(quote! {
        impl #impl_generics ::ecs_v2::Bundle for #name #ty_generics #where_clause {
            fn register(world: &mut ::ecs_v2::World) {
                #(#registrations)*
            }

            fn insert(self, world: &mut ::ecs_v2::World, entity: ::ecs_v2::Entity) -> Result<(), ::ecs_v2::EntityError> {
                #(#inserts)*
                Ok(())
            }

            fn remove(world: &mut ::ecs_v2::World, entity: ::ecs_v2::Entity) -> Result<Self, ::ecs_v2::EntityError> {
                Ok(Self {
                    #(#removals,)*
                })
            }
        }
    })
}
