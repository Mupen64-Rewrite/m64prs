use proc_macro::TokenStream;
use syn::parse_macro_input;

mod derive_typed_action_group;
mod forward_wrapper;
mod glib_callback;

/// Forwards methods from the inner class of a subtype.
///
/// # Usage
/// Basic usage (visibility defaults to private):
/// ```rust,ignore
/// #[forward_wrapper(super::MyObject)]
/// impl MyObject {
///     // ...
/// }
/// ```
///
/// Specify visibility:
/// ```rust,ignore
/// #[forward_wrapper(super::MyObject, vis = pub(crate))]
/// impl MyObject {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn forward_wrapper(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item2 = item.clone();
    let wrapper_type = parse_macro_input!(attr as forward_wrapper::Args);
    let impl_block = parse_macro_input!(item as syn::ItemImpl);

    let gen: TokenStream = match forward_wrapper::generate(wrapper_type, impl_block) {
        Ok(gen) => gen.into(),
        Err(err) => return err.into_compile_error().into(),
    };

    item2.extend(gen);
    item2
}

#[proc_macro_derive(TypedActionGroup, attributes(action))]
pub fn derive_typed_action_group(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::DeriveInput);

    match derive_typed_action_group::generate(item) {
        Ok(gen) => gen.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn glib_callback(input: TokenStream) -> TokenStream {
    let closure = parse_macro_input!(input as syn::ExprClosure);
    match glib_callback::generate(closure) {
        Ok(gen) => gen.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
