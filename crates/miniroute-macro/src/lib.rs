use proc_macro::TokenStream;

mod route;
mod router;

/// Defines a top-level router enum.
///
/// # Example
///
/// ```ignore
/// #[router]
/// pub enum Route {
///     #[to(Scale)]
///     Scale,
///     #[to(MainMenu)]
///     MainMenu,
/// }
/// ```
///
/// Each variant with `#[to(Type)]` will have its lifecycle task spawned via `Type::spawn_tasks()`.
#[proc_macro_attribute]
pub fn router(attr: TokenStream, item: TokenStream) -> TokenStream {
    router::router_impl(attr.into(), item.into()).into()
}

/// Defines a route with associated tasks.
///
/// # Example
///
/// ```ignore
/// #[route(router = Route)]
/// pub enum Scale {
///     #[task(timer_task)]
///     Timer,
///     #[task(weight_task)]
///     Weight,
/// }
/// ```
///
/// # Arguments
///
/// - `router = Type` (required): The router enum this route belongs to
/// - `hooks` (optional): User must implement `RouteHooks` manually
///
/// Each task function receives a context struct (`ScaleContext` in this example)
/// with `navigate()` and `run()` methods.
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::route_impl(attr.into(), item.into()).into()
}
