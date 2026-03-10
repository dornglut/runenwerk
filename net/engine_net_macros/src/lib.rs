use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, ItemStruct, LitBool, Path, Result, Token, parse_macro_input};

#[proc_macro_attribute]
pub fn net_entity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as ItemStruct);
    let struct_ident = &parsed.ident;
    let (impl_generics, ty_generics, where_clause) = parsed.generics.split_for_impl();
    quote! {
        #parsed

        impl #impl_generics ::engine_net::replication::NetEntity for #struct_ident #ty_generics #where_clause {}
    }
    .into()
}

#[proc_macro_attribute]
pub fn net_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(item as ItemStruct);
    let args = parse_macro_input!(attr as NetComponentArgs);

    let struct_ident = &parsed.ident;
    let (impl_generics, ty_generics, where_clause) = parsed.generics.split_for_impl();
    let authority = enum_variant_expr(args.authority, "AuthorityModel", "Server");
    let profile = enum_variant_expr(args.profile, "ReplicationProfilePreset", "ReliableState");
    let interest = enum_variant_expr(args.interest, "InterestPolicy", "Global");
    let owner_prediction = args.owner_prediction.unwrap_or(false);
    let direction = match args.direction {
        Some(direction_path) => path_to_replication_expr(direction_path, "ReplicationDirection"),
        None => {
            quote! { ::engine_net::replication::ReplicationProfile::from_preset(profile).direction }
        }
    };

    quote! {
        #parsed

        impl #impl_generics ::engine_net::replication::NetComponentMetadata for #struct_ident #ty_generics #where_clause {
            fn replication_descriptor() -> ::engine_net::replication::ReplicatedComponentDescriptor {
                let profile = #profile;
                let direction = #direction;
                ::engine_net::replication::ReplicatedComponentDescriptor {
                    component_name: stringify!(#struct_ident).to_string(),
                    authority: #authority,
                    direction,
                    profile,
                    interest: #interest,
                    owner_prediction: #owner_prediction,
                }
            }
        }
    }
    .into()
}

fn enum_variant_expr(
    value: Option<Path>,
    enum_name: &str,
    default_variant: &str,
) -> proc_macro2::TokenStream {
    match value {
        Some(path) => path_to_replication_expr(path, enum_name),
        None => {
            let default_ident = Ident::new(default_variant, proc_macro2::Span::call_site());
            let enum_ident = Ident::new(enum_name, proc_macro2::Span::call_site());
            quote! { ::engine_net::replication::#enum_ident::#default_ident }
        }
    }
}

fn path_to_replication_expr(path: Path, enum_name: &str) -> proc_macro2::TokenStream {
    if path.leading_colon.is_some() {
        return quote! { #path };
    }

    if path.segments.len() == 1 {
        let enum_ident = Ident::new(enum_name, proc_macro2::Span::call_site());
        let variant = &path.segments[0].ident;
        return quote! { ::engine_net::replication::#enum_ident::#variant };
    }

    let first = &path.segments[0].ident;
    let enum_ident = Ident::new(enum_name, proc_macro2::Span::call_site());
    if first == &enum_ident {
        quote! { ::engine_net::replication::#path }
    } else {
        quote! { #path }
    }
}

#[derive(Default)]
struct NetComponentArgs {
    authority: Option<Path>,
    direction: Option<Path>,
    profile: Option<Path>,
    interest: Option<Path>,
    owner_prediction: Option<bool>,
}

impl Parse for NetComponentArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut args = NetComponentArgs::default();
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "authority" => {
                    args.authority = Some(input.parse()?);
                }
                "direction" => {
                    args.direction = Some(input.parse()?);
                }
                "profile" => {
                    args.profile = Some(input.parse()?);
                }
                "interest" => {
                    args.interest = Some(input.parse()?);
                }
                "owner_prediction" => {
                    let value: LitBool = input.parse()?;
                    args.owner_prediction = Some(value.value);
                }
                other => {
                    return Err(Error::new(
                        key.span(),
                        format!(
                            "unsupported net_component argument `{other}` (expected authority, direction, profile, interest, owner_prediction)"
                        ),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(args)
    }
}
