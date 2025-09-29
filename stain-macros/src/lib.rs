//! The purpose of the [stain](self) crate is to make it easier
//! to define and register plugins.
//!
//! The goal is to be able to use [create_stain] on your trait
//! in order to generate the plugin store. This should also
//! generate a macro for registering plugins for the stained
//! trait.
//!
//! A goal of this plugin system is to work nicely with trait
//! generics and associated types. If generics or associated types
//! are detected, [create_stain] should allow the use of the
//! `alias` attribute which you should use to specify the Generic
//! parameters and associated types.
//!
//! Just as well, the generated registration macro should allow you
//! to specify an `ordering` for the registered plugin, which can
//! enable predictable runs.

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

use crate::{
    args::CreateArguments,
    traits::PluginTrait,
    utils::{module_path, validate_trait},
};

mod args;
mod traits;
mod utils;

#[proc_macro_attribute]
#[proc_macro_error]
/// Can be used on a trait declaration to create a plugin system
/// for the trait.
///
/// *Note:* If your code doesn't live in `src/`, then you must
/// specify the `src_prefix` to your code.
pub fn create_stain(args: TokenStream, input: TokenStream) -> TokenStream {
    let arguments = parse_macro_input!(args as CreateArguments);
    let plugin = parse_macro_input!(input as PluginTrait);

    plugin.into_tokens(arguments).into()
}
