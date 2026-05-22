use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, Lifetime, parse_macro_input,
    parse_quote,
};

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

#[proc_macro_derive(SystemParam)]
pub fn system_param_derive(input: TokenStream) -> TokenStream {
    let ecs = ecs_crate_path();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let Data::Struct(data) = input.data else {
        return TokenStream::from(quote! {
            compile_error!("SystemParam derive only supports structs with named fields");
        });
    };
    let Fields::Named(fields) = data.fields else {
        return TokenStream::from(quote! {
            compile_error!("SystemParam derive only supports structs with named fields");
        });
    };
    if fields.named.is_empty() {
        return TokenStream::from(quote! {
            compile_error!("SystemParam derive requires at least one named field");
        });
    }

    let fields = fields.named.into_iter().collect::<Vec<_>>();
    let field_idents = fields
        .iter()
        .map(|field| field.ident.as_ref().expect("named field"))
        .collect::<Vec<_>>();
    let field_names = field_idents
        .iter()
        .map(|field_ident| field_ident.to_string())
        .collect::<Vec<_>>();
    let field_types = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    let state_indices = (0..fields.len()).map(syn::Index::from).collect::<Vec<_>>();

    let system_param_lifetime = fresh_system_param_lifetime(&generics);
    let mut impl_generics = generics.clone();
    impl_generics
        .params
        .insert(0, parse_quote!(#system_param_lifetime));
    let where_clause = impl_generics.make_where_clause();
    for field_ty in &field_types {
        where_clause
            .predicates
            .push(parse_quote!(#field_ty: #ecs::SystemParam<#system_param_lifetime>));
    }
    let (impl_generics, _, where_clause) = impl_generics.split_for_impl();
    let (_, ty_generics, _) = generics.split_for_impl();
    let group_label = name.to_string();

    TokenStream::from(quote! {
        impl #impl_generics #ecs::SystemParam<#system_param_lifetime> for #name #ty_generics #where_clause {
            type State = (
                #(<#field_types as #ecs::SystemParam<#system_param_lifetime>>::State,)*
            );

            fn init_state(world: &mut #ecs::World) -> Result<Self::State, #ecs::SystemParamError> {
                Ok((
                    #(<#field_types as #ecs::SystemParam<#system_param_lifetime>>::init_state(world)?,)*
                ))
            }

            fn access(state: &Self::State) -> #ecs::QueryAccess {
                let mut access = #ecs::QueryAccess::default();
                #(
                    access.extend(<#field_types as #ecs::SystemParam<#system_param_lifetime>>::access(&state.#state_indices));
                )*
                access
            }

            fn slot_descriptor() -> #ecs::ParamSlotDescriptor {
                #ecs::ParamSlotDescriptor::group(
                    "param_group",
                    #group_label,
                    ::std::any::type_name::<Self>(),
                    vec![
                        #(
                            #ecs::ParamSlotDescriptor::named_child(
                                #field_names,
                                <#field_types as #ecs::SystemParam<#system_param_lifetime>>::slot_descriptor(),
                            ),
                        )*
                    ],
                )
            }

            unsafe fn extract(
                state: &#system_param_lifetime mut Self::State,
                world: *mut #ecs::World,
                commands: *mut #ecs::Commands,
            ) -> Result<Self, #ecs::SystemParamError> {
                Ok(Self {
                    #(
                        #field_idents: unsafe {
                            <#field_types as #ecs::SystemParam<#system_param_lifetime>>::extract(
                                &mut state.#state_indices,
                                world,
                                commands,
                            )?
                        },
                    )*
                })
            }
        }
    })
}

fn fresh_system_param_lifetime(generics: &syn::Generics) -> Lifetime {
    let base = "__ecs_system_param_w";
    let mut candidate = base.to_string();
    let mut suffix = 0;

    while generics.params.iter().any(|param| {
        matches!(
            param,
            syn::GenericParam::Lifetime(lifetime)
                if lifetime.lifetime.ident == candidate.as_str()
        )
    }) {
        suffix += 1;
        candidate = format!("{base}_{suffix}");
    }

    Lifetime::new(&format!("'{candidate}"), proc_macro2::Span::call_site())
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

    match input.data {
        Data::Struct(data) => {
            expand_reflect_struct(ecs, name, classification, stable_name, rust_type_name, data)
        }
        Data::Enum(data) => {
            expand_reflect_enum(ecs, name, classification, stable_name, rust_type_name, data)
        }
        Data::Union(_) => TokenStream::from(quote! {
            compile_error!("Reflect derives currently support only structs and unit enums");
        }),
    }
}

fn expand_reflect_struct(
    ecs: proc_macro2::TokenStream,
    name: Ident,
    classification: proc_macro2::TokenStream,
    stable_name: String,
    rust_type_name: proc_macro2::TokenStream,
    data: DataStruct,
) -> TokenStream {
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

fn expand_reflect_enum(
    ecs: proc_macro2::TokenStream,
    name: Ident,
    classification: proc_macro2::TokenStream,
    stable_name: String,
    rust_type_name: proc_macro2::TokenStream,
    data: DataEnum,
) -> TokenStream {
    let mut variant_idents = Vec::new();
    let mut variant_symbols = Vec::new();
    for variant in data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return TokenStream::from(quote! {
                compile_error!("Reflect enum derives currently support only unit/no-payload variants");
            });
        }
        variant_symbols.push(variant.ident.to_string());
        variant_idents.push(variant.ident);
    }

    if variant_idents.is_empty() {
        return TokenStream::from(quote! {
            compile_error!("Reflect enum derives require at least one unit variant");
        });
    }

    let variant_infos = variant_symbols.iter().map(|symbol| {
        quote! {
            #ecs::reflect::EnumVariantInfo::new(#symbol, #symbol)
        }
    });

    let current_arms = variant_idents
        .iter()
        .zip(variant_symbols.iter())
        .map(|(ident, symbol)| {
            quote! {
                #name::#ident => Some(#symbol),
            }
        });

    let set_arms = variant_idents
        .iter()
        .zip(variant_symbols.iter())
        .map(|(ident, symbol)| {
            quote! {
                #symbol => {
                    *typed = #name::#ident;
                    true
                }
            }
        });

    TokenStream::from(quote! {
        impl #ecs::reflect::Reflect for #name {
            fn type_info() -> &'static #ecs::reflect::TypeInfo
            where
                Self: Sized,
            {
                fn __ecs_reflect_current_variant(
                    owner: &dyn ::std::any::Any
                ) -> Option<&'static str> {
                    let typed = owner.downcast_ref::<#name>()?;
                    match typed {
                        #(#current_arms)*
                    }
                }

                fn __ecs_reflect_set_unit_variant(
                    owner: &mut dyn ::std::any::Any,
                    symbol: &str,
                ) -> bool {
                    let Some(typed) = owner.downcast_mut::<#name>() else {
                        return false;
                    };
                    match symbol {
                        #(#set_arms)*
                        _ => false,
                    }
                }

                static TYPE_INFO: ::std::sync::OnceLock<#ecs::reflect::TypeInfo> =
                    ::std::sync::OnceLock::new();
                static ENUM_INFO: ::std::sync::OnceLock<#ecs::reflect::EnumInfo> =
                    ::std::sync::OnceLock::new();

                TYPE_INFO.get_or_init(|| {
                    let reflect_type_id = #ecs::reflect::allocate_reflect_type_id();

                    let enum_info = ENUM_INFO.get_or_init(|| {
                        let variants = vec![
                            #(#variant_infos),*
                        ];
                        let leaked_variants: &'static [#ecs::reflect::EnumVariantInfo] =
                            ::std::boxed::Box::leak(variants.into_boxed_slice());
                        #ecs::reflect::EnumInfo::new(
                            leaked_variants,
                            __ecs_reflect_current_variant,
                            __ecs_reflect_set_unit_variant,
                        )
                    });

                    #ecs::reflect::TypeInfo::new(
                        reflect_type_id,
                        #rust_type_name,
                        #stable_name,
                        #classification,
                        #ecs::reflect::ReflectShape::Enum(enum_info),
                    )
                })
            }
        }
    })
}
