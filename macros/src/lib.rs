use proc_macro::TokenStream;
use syn_utils::into_macro_output;

#[macro_use]
mod syn_utils;

mod timeout_impl;

#[proc_macro_attribute]
pub fn timeout(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> TokenStream {
    into_macro_output(timeout_impl::timeout(attr.into(), item.into()))
}

#[proc_macro_attribute]
pub fn should_timeout(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> TokenStream {
    into_macro_output(timeout_impl::should_timeout(attr.into(), item.into()))
}
