use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, punctuated::Punctuated, Attribute, ItemStruct, Meta, Token};

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

    if has_user_derive(attrs) {
        return syn::Error::new_spanned(
            ident,
            "#[id] injects derives automatically; remove any #[derive(...)] or cfg_attr(..., derive(...)) on this struct",
        )
        .to_compile_error();
    }

    if !generics.params.is_empty() {
        return syn::Error::new_spanned(generics, "#[id] does not support generics")
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
    let allocator_ident = format_ident!("{}Allocator", ident);
    quote! {
        #( #attrs )*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #vis struct #ident(::id::TypedId<#tag_ident>);

        #[doc(hidden)]
        #vis enum #tag_ident {}

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

        impl core::convert::TryFrom<u64> for #ident {
            type Error = ::id::InvalidRawId;

            fn try_from(value: u64) -> Result<Self, Self::Error> {
                ::id::TypedId::try_from_raw(value).map(Self)
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

        #vis type #allocator_ident = ::id::MonotonicIdAllocator<#tag_ident>;
    }
}

fn has_user_derive(attrs: &[Attribute]) -> bool {
    attrs.iter().any(attribute_contains_derive)
}

fn attribute_contains_derive(attr: &Attribute) -> bool {
    if attr.path().is_ident("derive") {
        return true;
    }

    if !attr.path().is_ident("cfg_attr") {
        return false;
    }

    // Be conservative: if cfg_attr cannot be parsed as meta list, treat it as
    // containing a derive to avoid bypassing the duplicate-derive guard.
    cfg_attr_meta_contains_derive(attr).unwrap_or(true)
}

fn cfg_attr_meta_contains_derive(attr: &Attribute) -> syn::Result<bool> {
    let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    Ok(metas.into_iter().skip(1).any(meta_contains_derive))
}

fn meta_contains_derive(meta: Meta) -> bool {
    match meta {
        Meta::Path(path) => path.is_ident("derive"),
        Meta::NameValue(name_value) => name_value.path.is_ident("derive"),
        Meta::List(list) => {
            if list.path.is_ident("derive") {
                return true;
            }
            if list.path.is_ident("cfg_attr") {
                if let Ok(metas) =
                    list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                {
                    return metas.into_iter().skip(1).any(meta_contains_derive);
                }
                return true;
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn detects_direct_derive_attribute() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[derive(Clone, Copy)])];
        assert!(has_user_derive(&attrs));
    }

    #[test]
    fn detects_cfg_attr_wrapped_derive_attribute() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[cfg_attr(feature = "x", derive(Clone))])];
        assert!(has_user_derive(&attrs));
    }

    #[test]
    fn ignores_non_derive_cfg_attr() {
        let attrs: Vec<Attribute> =
            vec![parse_quote!(#[cfg_attr(feature = "x", repr(transparent))])];
        assert!(!has_user_derive(&attrs));
    }

    #[test]
    fn generates_try_from_u64_instead_of_from_u64() {
        let item: ItemStruct = parse_quote! {
            pub struct RenderFlowId;
        };
        let expanded = expand_id(item);
        let text = quote!(#expanded).to_string();
        assert!(text.contains("TryFrom < u64 >"));
        assert!(!text.contains("impl From < u64 >"));
    }
}
