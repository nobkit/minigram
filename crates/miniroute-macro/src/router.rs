use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Fields, Ident, Result, Variant, parse2};

struct RouteVariant {
    variant_ident: Ident,
    route_type: Ident,
}

fn parse_to_attr(variant: &Variant) -> Result<Option<RouteVariant>> {
    for attr in &variant.attrs {
        if attr.path().is_ident("to") {
            let route_type: Ident = attr.parse_args()?;
            return Ok(Some(RouteVariant {
                variant_ident: variant.ident.clone(),
                route_type,
            }));
        }
    }
    Ok(None)
}

fn filter_to_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("to"))
        .cloned()
        .collect()
}

pub fn router_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match router_impl_inner(item) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn router_impl_inner(item: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(item)?;
    let enum_name = &input.ident;
    let vis = &input.vis;

    let data_enum = match &input.data {
        Data::Enum(e) => e,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "router can only be applied to enums",
            ));
        }
    };

    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) || variant.discriminant.is_some() {
            return Err(Error::new_spanned(
                variant,
                "router variants must be fieldless with no explicit discriminants",
            ));
        }
    }

    let mut route_variants: Vec<RouteVariant> = Vec::new();
    for variant in &data_enum.variants {
        if let Some(rv) = parse_to_attr(variant)? {
            route_variants.push(rv);
        }
    }

    let variant_count = data_enum.variants.len();

    let cleaned_variants: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let fields = &v.fields;
            let attrs = filter_to_attrs(&v.attrs);
            match fields {
                Fields::Unit => quote! { #(#attrs)* #ident },
                Fields::Unnamed(f) => quote! { #(#attrs)* #ident #f },
                Fields::Named(f) => quote! { #(#attrs)* #ident #f },
            }
        })
        .collect();

    let enum_attrs = &input.attrs;

    let spawn_calls: Vec<_> = route_variants
        .iter()
        .map(|rv| {
            let route_type = &rv.route_type;
            quote! { #route_type::spawn_tasks(spawner); }
        })
        .collect();

    let route_of_impls: Vec<_> = route_variants
        .iter()
        .map(|rv| {
            let route_type = &rv.route_type;
            let variant_ident = &rv.variant_ident;
            quote! {
                impl ::miniroute::__private::RouteOf<#enum_name> for #route_type {
                    const VARIANT: #enum_name = #enum_name::#variant_ident;
                }
            }
        })
        .collect();

    let variant_defmt_entries: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let name = ident.to_string();
            quote! { #ident = #name }
        })
        .collect();

    let output = quote! {
        #(#enum_attrs)*
        #[derive(Clone, Copy, PartialEq)]
        #vis enum #enum_name {
            #(#cleaned_variants),*
        }

        impl #enum_name {
            /// The global router watch for inter-route navigation.
            fn router() -> &'static ::miniroute::__private::Watch<
                ::miniroute::__private::CriticalSectionRawMutex,
                #enum_name,
                #variant_count,
            > {
                static INNER: ::miniroute::__private::Watch<
                    ::miniroute::__private::CriticalSectionRawMutex,
                    #enum_name,
                    #variant_count,
                > = ::miniroute::__private::Watch::new();
                &INNER
            }

            /// Spawn all route lifecycle tasks. Call once at startup.
            pub fn spawn_routes(spawner: &::miniroute::__private::Spawner) {
                #(#spawn_calls)*
            }

            /// Activate the initial route. Call after `spawn_routes()`.
            pub fn start(initial: #enum_name) {
                Self::router().sender().send(initial);
            }
        }

        ::miniroute::__private::impl_defmt_format!(#enum_name { #(#variant_defmt_entries),* });

        #(#route_of_impls)*
    };

    Ok(output)
}
