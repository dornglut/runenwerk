use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Generics, parse_macro_input};

#[proc_macro_derive(GpuUniform)]
pub fn derive_gpu_uniform(input: TokenStream) -> TokenStream {
    expand_gpu_params(input, LayoutKind::Uniform)
}

#[proc_macro_derive(GpuStorage)]
pub fn derive_gpu_storage(input: TokenStream) -> TokenStream {
    expand_gpu_params(input, LayoutKind::Storage)
}

#[derive(Copy, Clone)]
enum LayoutKind {
    Uniform,
    Storage,
}

fn expand_gpu_params(input: TokenStream, layout: LayoutKind) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if !input.generics.params.is_empty() {
        return compile_error("Gpu derive does not currently support generic type parameters");
    }

    let struct_ident = input.ident;
    let raw_ident = format_ident!("{}GpuRaw", struct_ident);
    let render_path = render_module_path();
    let bytemuck_path = quote! { #render_path::bytemuck };
    let gpu_layout_trait = match layout {
        LayoutKind::Uniform => quote! { #render_path::GpuUniform },
        LayoutKind::Storage => quote! { #render_path::GpuStorage },
    };

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return compile_error("Gpu derive only supports structs with named fields");
            }
        },
        _ => {
            return compile_error("Gpu derive only supports structs");
        }
    };

    match layout {
        LayoutKind::Storage => {
            let field_defs = fields.iter().map(|field| {
                let field_ident = field.ident.as_ref().expect("named field");
                let ty = &field.ty;
                quote! {
                    pub #field_ident: <#ty as #render_path::ToGpuValue>::Gpu
                }
            });

            let field_inits = fields.iter().map(|field| {
                let field_ident = field.ident.as_ref().expect("named field");
                quote! {
                    #field_ident: #render_path::ToGpuValue::to_gpu_value(&self.#field_ident)
                }
            });

            let field_bounds = fields.iter().map(|field| {
                let ty = &field.ty;
                quote! {
                    #ty: #render_path::ToGpuValue
                }
            });
            let where_clause = append_bounds(input.generics, field_bounds.collect());

            TokenStream::from(quote! {
                #[doc(hidden)]
                #[repr(C)]
                #[derive(Clone, Copy, #bytemuck_path::Pod, #bytemuck_path::Zeroable)]
                pub struct #raw_ident {
                    #(#field_defs,)*
                }

                impl #render_path::GpuParams for #struct_ident #where_clause {
                    type Raw = #raw_ident;

                    fn to_gpu(&self) -> Self::Raw {
                        #raw_ident {
                            #(#field_inits,)*
                        }
                    }
                }

                impl #gpu_layout_trait for #struct_ident #where_clause {}
            })
        }
        LayoutKind::Uniform => {
            let struct_tag = struct_ident.to_string().to_uppercase();
            let total_size_ident = format_ident!("__{}_GPU_UNIFORM_SIZE", struct_tag);

            let mut offset_consts = Vec::<proc_macro2::TokenStream>::new();
            let mut write_calls = Vec::<proc_macro2::TokenStream>::new();
            let mut where_bounds = Vec::<proc_macro2::TokenStream>::new();
            let mut next_offset_expr = quote! { 0usize };

            for field in &fields {
                let field_ident = field.ident.as_ref().expect("named field");
                let field_tag = field_ident.to_string().to_uppercase();
                let offset_ident = format_ident!("__{}_GPU_OFFSET_{}", struct_tag, field_tag);
                let ty = &field.ty;

                offset_consts.push(quote! {
                    const #offset_ident: usize = #render_path::align_up_const(
                        #next_offset_expr,
                        <#ty as #render_path::GpuUniformField>::ABI_ALIGN
                    );
                });

                write_calls.push(quote! {
                    #render_path::write_uniform_field::<#ty>(&mut bytes, #offset_ident, &self.#field_ident);
                });

                where_bounds.push(quote! {
                    #ty: #render_path::GpuUniformField
                });

                next_offset_expr = quote! {
                    #offset_ident + <#ty as #render_path::GpuUniformField>::ABI_SIZE
                };
            }

            let total_size_def = quote! {
                const #total_size_ident: usize = #render_path::align_up_const(#next_offset_expr, 16usize);
            };
            let where_clause = append_bounds(input.generics, where_bounds);

            TokenStream::from(quote! {
                #(#offset_consts)*
                #total_size_def

                #[doc(hidden)]
                #[repr(C)]
                #[derive(Clone, Copy, #bytemuck_path::Pod, #bytemuck_path::Zeroable)]
                pub struct #raw_ident {
                    pub bytes: [u8; #total_size_ident],
                }

                impl #render_path::GpuParams for #struct_ident #where_clause {
                    type Raw = #raw_ident;

                    fn to_gpu(&self) -> Self::Raw {
                        let mut bytes = [0u8; #total_size_ident];
                        #(#write_calls)*
                        #raw_ident { bytes }
                    }
                }

                impl #gpu_layout_trait for #struct_ident #where_clause {}
            })
        }
    }
}

fn append_bounds(
    generics: Generics,
    mut bounds: Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let mut where_parts = Vec::new();
    if let Some(where_clause) = generics.where_clause {
        where_parts.extend(
            where_clause
                .predicates
                .into_iter()
                .map(|predicate| quote! { #predicate }),
        );
    }
    where_parts.append(&mut bounds);

    if where_parts.is_empty() {
        quote! {}
    } else {
        quote! {
            where #(#where_parts,)*
        }
    }
}

fn compile_error(message: &str) -> TokenStream {
    TokenStream::from(quote! {
        compile_error!(#message);
    })
}

fn render_module_path() -> proc_macro2::TokenStream {
    match crate_name("engine") {
        Ok(FoundCrate::Itself) => quote! { ::engine::plugins::render },
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{}", name);
            quote! { ::#ident::plugins::render }
        }
        Err(_) => quote! { ::engine::plugins::render },
    }
}
