//! Parsing arguments of main [create_stain](super::create_stain) macro.

use std::collections::HashMap;

use proc_macro_error::{abort, abort_call_site, abort_if_dirty, emit_error};
use proc_macro2::Span;
use syn::{
    Ident, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use crate::module_path;

#[derive(Clone, Copy)]
enum ArgumentType {
    String,
    Path,
    Bool,
}

const ARGUMENTS: &'static [(&'static str, ArgumentType)] = &[
    ("src_prefix", ArgumentType::String),
    ("module_path", ArgumentType::Path),
    ("concrete", ArgumentType::Bool),
];

pub(crate) struct CreateArguments {
    pub(crate) module_path: Option<Path>,
    pub(crate) concrete: bool,
}

impl Parse for CreateArguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut str_args = HashMap::<Box<str>, (Span, Box<str>)>::new();
        let mut bool_args = HashMap::<Box<str>, (Span, bool)>::new();
        let mut path_args = HashMap::<Box<str>, (Span, Path)>::new();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let str_ident = ident.to_string();

            let Some((_arg, ty)) = ARGUMENTS
                .iter()
                .find_map(|(arg, ty)| str_ident.eq(*arg).then_some((*arg, *ty)))
            else {
                let ident_span = ident.span();
                let msg = format!("Unsupported argument '{str_ident}'...");

                abort!(ident_span, msg);
            };

            match ty {
                ArgumentType::String => {
                    let eq = input.parse::<Token![=]>()?;
                    let value = input.parse::<LitStr>()?;

                    let span = ident
                        .span()
                        .join(eq.span)
                        .and_then(|span| span.join(value.span()));

                    let Some(span) = span else {
                        abort_call_site!("Unexpected issue parsing arguments...");
                    };

                    str_args.insert(
                        str_ident.into_boxed_str(),
                        (span, value.value().into_boxed_str()),
                    );
                }
                ArgumentType::Bool => {
                    bool_args.insert(str_ident.into_boxed_str(), (ident.span(), true));
                }
                ArgumentType::Path => {
                    let eq = input.parse::<Token![=]>()?;
                    let value = input.parse::<Path>()?;

                    let span = ident
                        .span()
                        .join(eq.span)
                        .and_then(|span| span.join(value.span()));

                    let Some(span) = span else {
                        abort_call_site!("Unexpected issue parsing arguments...");
                    };

                    path_args.insert(str_ident.into_boxed_str(), (span, value));
                }
            }

            input.parse::<Option<Token![,]>>()?;
        }

        if let Some((src_span, ..)) = str_args.get("src_prefix") {
            if let Some((path_span, ..)) = path_args.get("module_path") {
                emit_error!(
                    src_span,
                    "The arguments `src_prefix` and `module_path` are mutually exclusive..."
                );
                emit_error!(
                    path_span,
                    "The arguments `src_prefix` and `module_path` are mutually exclusive..."
                );

                abort_if_dirty();
            }
        }

        let module_path = if let Some((_, module_path)) = path_args.remove("module_path") {
            Some(module_path)
        } else {
            let src_prefix = str_args
                .remove("src_prefix")
                .map(|(_, prefix)| prefix)
                .unwrap_or(Box::from("src/"));

            module_path(proc_macro2::Span::call_site(), &src_prefix)
        };

        Ok(Self {
            concrete: bool_args
                .remove("concrete")
                .map(|(_, concrete)| concrete)
                .unwrap_or(false),
            module_path,
        })
    }
}
