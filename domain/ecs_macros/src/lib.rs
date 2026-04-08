use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

fn ecs_crate_path() -> proc_macro2::TokenStream {
    match crate_name("ecs") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            quote!(::#ident)
        }
        Err(_) => quote!(::ecs),
    }
}

#[proc_macro_derive(Component)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #ecs::Component for #name #ty_generics #where_clause {}
    })
}

#[proc_macro_derive(StatefulComponent)]
pub fn stateful_component_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #ecs::StatefulComponent for #name #ty_generics #where_clause {}
    })
}

#[proc_macro_derive(Resource)]
pub fn resource_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #ecs::Resource for #name #ty_generics #where_clause {}
    })
}

#[proc_macro_derive(Bundle)]
pub fn bundle_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
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
        impl #impl_generics #ecs::Bundle for #name #ty_generics #where_clause {
            fn register(world: &mut #ecs::World) {
                #(#registrations)*
            }

            fn insert(self, world: &mut #ecs::World, entity: #ecs::Entity) -> Result<(), #ecs::EntityError> {
                #(#inserts)*
                Ok(())
            }

            fn remove(world: &mut #ecs::World, entity: #ecs::Entity) -> Result<Self, #ecs::EntityError> {
                Ok(Self {
                    #(#removals,)*
                })
            }
        }
    })
}

#[proc_macro_derive(Reflect)]
pub fn reflect_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    expand_reflect(input, quote!(#ecs::reflect::ReflectClassification::Plain))
}

#[proc_macro_derive(ReflectComponent)]
pub fn reflect_component_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    expand_reflect(
        input,
        quote!(#ecs::reflect::ReflectClassification::Component),
    )
}

#[proc_macro_derive(ReflectResource)]
pub fn reflect_resource_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    expand_reflect(
        input,
        quote!(#ecs::reflect::ReflectClassification::Resource),
    )
}

fn expand_reflect(input: TokenStream, classification: proc_macro2::TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;

    if !generics.params.is_empty() {
        return TokenStream::from(quote! {
            compile_error!("Reflect derives currently support only non-generic named structs");
        });
    }

    let stable_name = name.to_string();
    let rust_type_name = quote!(::std::any::type_name::<Self>());

    let Data::Struct(data) = input.data else {
        return TokenStream::from(quote! {
            compile_error!("Reflect derives currently support only structs");
        });
    };

    let Fields::Named(fields) = data.fields else {
        return TokenStream::from(quote! {
            compile_error!("Reflect derives currently support only named structs");
        });
    };

    let field_accessors = fields.named.iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("named field");
        let get_ref_fn = format_ident!("__ecs_reflect_get_ref_{}", field_ident);
        let get_mut_fn = format_ident!("__ecs_reflect_get_mut_{}", field_ident);

        quote! {
            fn #get_ref_fn<'a>(
                owner: &'a dyn ::std::any::Any
            ) -> Option<#ecs::reflect::ReflectValueRef<'a>> {
                let typed = owner.downcast_ref::<#name>()?;
                Some(#ecs::reflect::ReflectValueRef::new(&typed.#field_ident))
            }

            fn #get_mut_fn<'a>(
                owner: &'a mut dyn ::std::any::Any
            ) -> Option<#ecs::reflect::ReflectValueMut<'a>> {
                let typed = owner.downcast_mut::<#name>()?;
                Some(#ecs::reflect::ReflectValueMut::new(&mut typed.#field_ident))
            }
        }
    });

    let field_infos = fields.named.iter().map(|field| {
        let field_ident = field.ident.as_ref().expect("named field");
        let field_name_string = field_ident.to_string();
        let field_ty = &field.ty;
        let get_ref_fn = format_ident!("__ecs_reflect_get_ref_{}", field_ident);
        let get_mut_fn = format_ident!("__ecs_reflect_get_mut_{}", field_ident);

        quote! {
            #ecs::reflect::FieldInfo::new(
                #field_name_string,
                #field_name_string,
                <#field_ty as #ecs::reflect::Reflect>::type_info().id,
                #get_ref_fn,
                #get_mut_fn,
            )
        }
    });

    TokenStream::from(quote! {
        impl #ecs::reflect::Reflect for #name {
            fn type_info() -> &'static #ecs::reflect::TypeInfo
            where
                Self: Sized,
            {
                #(#field_accessors)*

                static TYPE_INFO: ::std::sync::OnceLock<#ecs::reflect::TypeInfo> =
                    ::std::sync::OnceLock::new();
                static STRUCT_INFO: ::std::sync::OnceLock<#ecs::reflect::StructInfo> =
                    ::std::sync::OnceLock::new();

                TYPE_INFO.get_or_init(|| {
                    let reflect_type_id = #ecs::reflect::allocate_reflect_type_id();

                    let struct_info = STRUCT_INFO.get_or_init(|| {
                        let fields = vec![
                            #(#field_infos),*
                        ];
                        let leaked_fields: &'static [#ecs::reflect::FieldInfo] =
                            ::std::boxed::Box::leak(fields.into_boxed_slice());
                        #ecs::reflect::StructInfo::new(leaked_fields)
                    });

                    #ecs::reflect::TypeInfo::new(
                        reflect_type_id,
                        #rust_type_name,
                        #stable_name,
                        #classification,
                        #ecs::reflect::ReflectShape::Struct(struct_info),
                    )
                })
            }
        }
    })
}
