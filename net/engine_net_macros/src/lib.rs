use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, ItemStruct, LitBool, LitInt, Path, Result, Token, parse_macro_input};

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
    let direction = optional_enum_variant_expr(args.direction, "ReplicationDirection");
    let reliability = optional_enum_variant_expr(args.reliability, "Reliability");
    let prediction = optional_enum_variant_expr(args.prediction, "PredictionMode");
    let priority = optional_enum_variant_expr(args.priority, "BandwidthPriority");
    let frequency_hz = match args.frequency_hz {
        Some(value) => quote! { Some(#value) },
        None => quote! { None },
    };

    quote! {
        #parsed

        impl #impl_generics ::engine_net::replication::NetComponentMetadata for #struct_ident #ty_generics #where_clause {
            fn replication_descriptor() -> ::engine_net::replication::ReplicatedComponentDescriptor {
                let profile = #profile;
                ::engine_net::replication::ReplicatedComponentDescriptor::new(
                    stringify!(#struct_ident).to_string(),
                    #authority,
                    profile,
                    #interest,
                    #owner_prediction,
                    ::engine_net::replication::ReplicationSemanticsOverrides {
                        direction: #direction,
                        reliability: #reliability,
                        frequency_hz: #frequency_hz,
                        prediction: #prediction,
                        priority: #priority,
                    },
                )
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

fn optional_enum_variant_expr(value: Option<Path>, enum_name: &str) -> proc_macro2::TokenStream {
    match value {
        Some(path) => {
            let expr = path_to_replication_expr(path, enum_name);
            quote! { Some(#expr) }
        }
        None => quote! { None },
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
    reliability: Option<Path>,
    prediction: Option<Path>,
    priority: Option<Path>,
    frequency_hz: Option<u16>,
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
                "reliability" => {
                    args.reliability = Some(input.parse()?);
                }
                "prediction" => {
                    args.prediction = Some(input.parse()?);
                }
                "priority" => {
                    args.priority = Some(input.parse()?);
                }
                "frequency_hz" => {
                    let value: LitInt = input.parse()?;
                    args.frequency_hz = Some(value.base10_parse()?);
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
                            "unsupported net_component argument `{other}` (expected authority, direction, reliability, prediction, priority, frequency_hz, profile, interest, owner_prediction)"
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
