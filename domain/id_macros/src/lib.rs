use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn id(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    expand_id(item).into()
}

fn expand_id(item: ItemStruct) -> proc_macro2::TokenStream {
    let vis = &item.vis;
    let ident = &item.ident;
    let attrs = &item.attrs;
    let generics = &item.generics;

    if !generics.params.is_empty() {
        return syn::Error::new_spanned(
            generics,
            "#[id] does not support generics",
        )
          .to_compile_error();
    }

    match &item.fields {
        syn::Fields::Unit => {}
        _ => {
            return syn::Error::new_spanned(
                &item.fields,
                "#[id] can only be used on unit structs like `pub struct RenderFlowId;`",
            )
              .to_compile_error();
        }
    }

    let tag_ident = format_ident!("__{}Tag", ident);
    let sequence_ident = format_ident!("{}Sequence", ident);
    let debug_name = ident.to_string();

    quote! {
        #( #attrs )*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #vis struct #ident(::id::TypedId<#tag_ident>);

        #[doc(hidden)]
        #vis enum #tag_ident {}

        impl ::id::IdTag for #tag_ident {
            const DEBUG_NAME: &'static str = #debug_name;
        }

        impl #ident {
            pub const fn new(raw: u64) -> Self {
                Self(::id::TypedId::new(raw))
            }

            pub const fn raw(self) -> u64 {
                self.0.raw()
            }
        }

        impl core::fmt::Debug for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl core::fmt::Display for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl From<u64> for #ident {
            fn from(value: u64) -> Self {
                Self::new(value)
            }
        }

        impl From<#ident> for u64 {
            fn from(value: #ident) -> Self {
                value.raw()
            }
        }

        impl From<::id::TypedId<#tag_ident>> for #ident {
            fn from(value: ::id::TypedId<#tag_ident>) -> Self {
                Self(value)
            }
        }

        impl From<#ident> for ::id::TypedId<#tag_ident> {
            fn from(value: #ident) -> Self {
                value.0
            }
        }

        #vis type #sequence_ident = ::id::TypedIdSequence<#tag_ident>;
    }
}