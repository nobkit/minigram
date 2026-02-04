use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Error, Fields, Ident, Result, Token, Variant, parse2};

struct RouteArgs {
    router: Ident,
    hooks: bool,
}

impl syn::parse::Parse for RouteArgs {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut router: Option<Ident> = None;
        let mut hooks = false;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            if ident == "router" {
                input.parse::<Token![=]>()?;
                router = Some(input.parse()?);
            } else if ident == "hooks" {
                hooks = true;
            } else {
                return Err(Error::new_spanned(ident, "unknown attribute argument"));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let router = router.ok_or_else(|| {
            Error::new(
                Span::call_site(),
                "missing required `router = TypeName` argument",
            )
        })?;

        Ok(RouteArgs { router, hooks })
    }
}

struct TaskInfo {
    task_fn: Ident,
    variant: Ident,
}

fn parse_task_attr(variant: &Variant) -> Result<Option<TaskInfo>> {
    let variant_ident = variant.ident.clone();

    for attr in &variant.attrs {
        if attr.path().is_ident("task") {
            let task_fn: Ident = attr.parse_args()?;
            return Ok(Some(TaskInfo {
                task_fn,
                variant: variant_ident,
            }));
        }
    }
    Ok(None)
}

fn filter_task_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("task"))
        .cloned()
        .collect()
}

pub fn route_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    match route_impl_inner(attr, item) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn route_impl_inner(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let args: RouteArgs = parse2(attr)?;
    let input: DeriveInput = parse2(item)?;

    let enum_name = &input.ident;
    let enum_name_lower = format_ident!("__{}_main", enum_name.to_string().to_lowercase());
    let builder_name = format_ident!("{}Builder", enum_name);
    let vis = &input.vis;
    let router = &args.router;

    let data_enum = match &input.data {
        Data::Enum(e) => e,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "route can only be applied to enums",
            ));
        }
    };

    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) || variant.discriminant.is_some() {
            return Err(Error::new_spanned(
                variant,
                "route variants must be fieldless with no explicit discriminants",
            ));
        }
    }

    let mut tasks: Vec<TaskInfo> = Vec::new();
    for variant in &data_enum.variants {
        if let Some(task) = parse_task_attr(variant)? {
            tasks.push(task);
        }
    }

    let variant_count = data_enum.variants.len();

    let cleaned_variants: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let fields = &v.fields;
            let attrs = filter_task_attrs(&v.attrs);
            match fields {
                Fields::Unit => quote! { #(#attrs)* #ident },
                Fields::Unnamed(f) => quote! { #(#attrs)* #ident #f },
                Fields::Named(f) => quote! { #(#attrs)* #ident #f },
            }
        })
        .collect();

    let enum_attrs: Vec<_> = input.attrs.iter().collect();

    let variant_idents: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            quote! { #enum_name::#ident }
        })
        .collect();

    let spawn_calls: Vec<_> = tasks
        .iter()
        .map(|task| {
            let task_fn = &task.task_fn;
            let variant = &task.variant;
            quote! { spawner.spawn(#task_fn(#enum_name::#variant)).unwrap(); }
        })
        .collect();

    let hooks_impl = if !args.hooks {
        quote! {
            impl ::miniroute::__private::RouteHooks for #enum_name {}
        }
    } else {
        quote! {}
    };

    let variant_defmt_entries: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| {
            let ident = &v.ident;
            let name = ident.to_string();
            quote! { #ident = #name }
        })
        .collect();

    let variant_count_u16 = variant_count as u16;

    let output = quote! {
        #(#enum_attrs)*
        #[repr(u16)]
        #[derive(Clone, Copy, PartialEq)]
        #vis enum #enum_name {
            #(#cleaned_variants),*
        }

        impl #enum_name {
            pub const VARIANT_COUNT: u16 = #variant_count_u16;

            pub fn from_u16(value: u16) -> ::core::option::Option<Self> {
                if value < Self::VARIANT_COUNT {
                    // SAFETY: value is in range, enum is repr(u16), variants are contiguous from 0
                    ::core::option::Option::Some(unsafe { ::core::mem::transmute(value) })
                } else {
                    ::core::option::Option::None
                }
            }

            pub fn as_u16(self) -> u16 {
                self as u16
            }

            pub const ROUTE: #router = #router::#enum_name;

            fn lifecycle() -> &'static ::miniroute::__private::Watch<
                ::miniroute::__private::CriticalSectionRawMutex,
                ::miniroute::__private::RouteCommand,
                #variant_count,
            > {
                static INNER: ::miniroute::__private::Watch<
                    ::miniroute::__private::CriticalSectionRawMutex,
                    ::miniroute::__private::RouteCommand,
                    #variant_count,
                > = ::miniroute::__private::Watch::new();
                &INNER
            }

            fn ack() -> &'static ::miniroute::__private::Channel<
                ::miniroute::__private::CriticalSectionRawMutex,
                #enum_name,
                #variant_count,
            > {
                static INNER: ::miniroute::__private::Channel<
                    ::miniroute::__private::CriticalSectionRawMutex,
                    #enum_name,
                    #variant_count,
                > = ::miniroute::__private::Channel::new();
                &INNER
            }

            fn navigate_request() -> &'static ::miniroute::__private::Signal<
                ::miniroute::__private::CriticalSectionRawMutex,
                #router,
            > {
                static INNER: ::miniroute::__private::Signal<
                    ::miniroute::__private::CriticalSectionRawMutex,
                    #router,
                > = ::miniroute::__private::Signal::new();
                &INNER
            }

            pub fn navigate(&self, destination: #router) {
                Self::navigate_request().signal(destination);
            }

            pub fn task<W: AsyncFnMut() -> ()>(self, work: W) -> #builder_name<W> {
                #builder_name {
                    variant: self,
                    work,
                    setup: ::miniroute::__private::Noop,
                    cleanup: ::miniroute::__private::Noop,
                }
            }

            pub fn spawn_tasks(spawner: &::miniroute::__private::Spawner) {
                spawner.spawn(#enum_name_lower()).unwrap();
                #(#spawn_calls)*
            }

            async fn run<T, S, C>(variant: Self, mut work: T, mut setup: S, mut cleanup: C)
            where
                T: AsyncFnMut() -> (),
                S: ::miniroute::__private::Callback,
                C: ::miniroute::__private::Callback,
            {
                let lifecycle = Self::lifecycle();
                let mut lifecycle_rx = lifecycle.receiver().unwrap();

                let ack = Self::ack();
                let ack_tx = ack.sender();

                loop {
                    lifecycle_rx
                        .get_and(|&command| command == ::miniroute::__private::RouteCommand::Start)
                        .await;

                    setup.call().await;

                    loop {
                        match ::miniroute::__private::select(
                            lifecycle_rx.get_and(|&command| command == ::miniroute::__private::RouteCommand::Stop),
                            work(),
                        )
                        .await
                        {
                            ::miniroute::__private::Either::First(_) => break,
                            _ => {}
                        };
                        ::miniroute::__private::yield_now().await;
                    }

                    cleanup.call().await;

                    ack_tx.send(variant).await;
                }
            }
        }

        #vis struct #builder_name<W, S = ::miniroute::__private::Noop, C = ::miniroute::__private::Noop> {
            variant: #enum_name,
            work: W,
            setup: S,
            cleanup: C,
        }

        impl<W, S, C> #builder_name<W, S, C>
        where
            W: AsyncFnMut() -> (),
            S: ::miniroute::__private::Callback,
            C: ::miniroute::__private::Callback,
        {
            pub fn setup<S2: AsyncFnMut() -> ()>(self, setup: S2) -> #builder_name<W, S2, C> {
                #builder_name {
                    variant: self.variant,
                    work: self.work,
                    setup,
                    cleanup: self.cleanup,
                }
            }

            pub fn cleanup<C2: AsyncFnMut() -> ()>(self, cleanup: C2) -> #builder_name<W, S, C2> {
                #builder_name {
                    variant: self.variant,
                    work: self.work,
                    setup: self.setup,
                    cleanup,
                }
            }

            pub async fn run(self) {
                #enum_name::run(self.variant, self.work, self.setup, self.cleanup).await
            }
        }

        #hooks_impl

        ::miniroute::__private::impl_defmt_format!(#enum_name { #(#variant_defmt_entries),* });

        #[embassy_executor::task]
        async fn #enum_name_lower() {
            use ::miniroute::__private::WaitResult::*;
            use ::miniroute::__private::WithTimeout;

            let lifecycle = #enum_name::lifecycle();
            let lifecycle_tx = lifecycle.sender();

            let ack = #enum_name::ack();
            let ack_rx = ack.receiver();

            let router = #router::router();
            let mut router_rx = router.subscriber().unwrap();
            let router_tx = router.immediate_publisher();

            loop {
                match router_rx.next_message().await {
                    Message(r) if r == #enum_name::ROUTE => {
                        <#enum_name as ::miniroute::__private::RouteHooks>::setup().await;
                        lifecycle_tx.send(::miniroute::__private::RouteCommand::Start);

                        let deferred_destination = #enum_name::navigate_request().wait().await;

                        let mut remaining: ::miniroute::__private::Vec<#enum_name, #variant_count> =
                            ::miniroute::__private::Vec::from_array([#(#variant_idents),*]);

                        lifecycle_tx.send(::miniroute::__private::RouteCommand::Stop);

                        while !remaining.is_empty() {
                            match ack_rx.receive()
                                .with_timeout(::miniroute::__private::Duration::from_secs(1))
                                .await
                            {
                                Ok(id) => {
                                    if let Some(pos) = remaining.iter().position(|&t| t == id) {
                                        remaining.swap_remove(pos);
                                    } else {
                                        ::miniroute::__private::log_unknown_task_ack();
                                    }
                                }
                                Err(_) => {
                                    ::miniroute::__private::log_tasks_timeout();
                                    break;
                                }
                            }
                        }

                        #enum_name::navigate_request().reset();
                        <#enum_name as ::miniroute::__private::RouteHooks>::cleanup().await;
                        router_tx.publish_immediate(deferred_destination);
                    }
                    Message(_) => {}
                    Lagged(lagged) => {
                        ::miniroute::__private::log_router_lagged(lagged as u64);
                    }
                }
            }
        }
    };

    Ok(output)
}
