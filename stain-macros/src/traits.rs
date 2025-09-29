//! The parsed traits and their respective
//! [ToTokens](quote::ToTokens) implementations.

use std::ops::Deref;

use heck::{ToPascalCase, ToSnakeCase};
use proc_macro_error::{abort, abort_call_site, abort_if_dirty, emit_error};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    AssocType, Attribute, Ident, ItemTrait, LitStr, Meta, Token, TraitBound, TraitBoundModifier,
    TraitItem, Type, TypeParamBound,
    parse::{Parse, ParseStream},
};

use crate::{CreateArguments, validate_trait};

pub(crate) struct GenericPlugin {
    item_trait: ItemTrait,

    extra_attrs: Box<[Attribute]>,
    aliases: Box<[GenericAlias]>,
}

impl GenericPlugin {
    fn into_tokens(self, arguments: CreateArguments) -> TokenStream {
        let call_site = Span::call_site();
        let module_path = arguments.module_path;

        let mut item_trait = self.item_trait;
        let trait_ident = item_trait.ident.clone();
        let trait_ident_snake = trait_ident.to_string().to_snake_case();

        if arguments.concrete {
            let Ok(path) = syn::parse_str::<syn::Path>("::stain::AsAny") else {
                abort_call_site!("Unexpected error... couldn't build path to AsAny.");
            };

            let trait_bound = TypeParamBound::Trait(TraitBound {
                lifetimes: None,
                modifier: TraitBoundModifier::None,
                paren_token: None,
                path,
            });

            item_trait.supertraits.push(trait_bound);
        }

        let maybe_auto_impl = if arguments.concrete {
            Some(quote! {
                impl ::stain::AsAny for $target {
                    fn as_any(&self) -> &dyn ::std::any::Any {
                        self
                    }

                    fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                        self
                    }
                }
            })
        } else {
            None
        };

        let extra_attrs = self.extra_attrs;
        let mut token_stream = quote! {
            #(#extra_attrs)*
            #item_trait
        };

        for alias in self.aliases {
            let alias_name = alias.name.to_pascal_case();
            let newtype_entry =
                format_ident!("{}{}Entry", trait_ident, alias_name, span = call_site);

            let alias_name_snek = alias.name.deref();

            let macro_ident = format_ident!("{}_{}_stain", trait_ident_snake, alias_name_snek);

            let associated = alias.associated;
            let generics = alias.generics;

            let concrete_trait = quote! {
                #trait_ident<#(#generics,)* #(#associated),*>
            };

            let dyn_alias = quote! {
                dyn #trait_ident<#(#generics,)* #(#associated),*> + Send + Sync + 'static
            };

            token_stream.extend(quote! {
                pub struct #newtype_entry(pub ::stain::Entry<Self>);

                impl #newtype_entry {
                    pub const fn create_entry<T>(
                        lazy_boxed: &'static ::std::sync::LazyLock<::stain::parking_lot::RwLock<Box<#dyn_alias>>>,
                        lazy_type: &'static ::std::sync::LazyLock<::std::any::TypeId>,
                        name: &'static str,
                        ordering: u32,
                    ) -> Self where T: #concrete_trait {
                        Self(
                            ::stain::Entry::<Self>::create_entry(
                                lazy_boxed,
                                lazy_type,
                                name,
                                ordering,
                            )
                        )
                    }
                }

                impl Clone for #newtype_entry {
                    fn clone(&self) -> Self {
                        Self(self.0.clone())
                    }
                }

                impl ::stain::IntoEntry for #newtype_entry {
                    type Target = #dyn_alias;

                    fn into_entry(self) -> ::stain::Entry<Self> {
                        self.0
                    }
                }

                ::stain::inventory::collect!(#newtype_entry);

                #[allow(unused_macros)]
                macro_rules! #macro_ident {
                    ($target:ty, $ordering:literal) => {
                        #maybe_auto_impl

                        #[allow(non_upper_case_globals)]
                        const _: () = {
                            static INIT: fn() -> $target = <$target as Default>::default;

                            static LAZY_BOXED: ::std::sync::LazyLock<::stain::parking_lot::RwLock<Box<#dyn_alias>>> =
                                ::std::sync::LazyLock::new(|| ::stain::parking_lot::RwLock::new(Box::new(INIT())));

                            static LAZY_TYPE: ::std::sync::LazyLock<::std::any::TypeId> =
                                ::std::sync::LazyLock::new(|| ::std::any::TypeId::of::<$target>());

                            ::stain::inventory::submit!(#module_path::#newtype_entry::create_entry::<$target>(
                                &LAZY_BOXED,
                                &LAZY_TYPE,
                                stringify!($target),
                                $ordering,
                            ));
                        };
                    };
                    ($target:ty) => {
                        #macro_ident!($target, 4294967295u32);
                    };
                }

                #[allow(unused_imports)]
                pub(crate) use #macro_ident;
            });
        }

        token_stream
    }
}

pub(crate) struct ExactPlugin {
    item_trait: ItemTrait,

    extra_attrs: Box<[Attribute]>,
}

impl ExactPlugin {
    fn into_tokens(self, arguments: CreateArguments) -> TokenStream {
        let call_site = Span::call_site();
        let module_path = arguments.module_path;

        let mut item_trait = self.item_trait;
        let trait_ident = item_trait.ident.clone();
        let trait_ident_snake = trait_ident.to_string().to_snake_case();

        let newtype_entry = format_ident!("{}Entry", trait_ident, span = call_site);
        let macro_ident = format_ident!("{}_stain", trait_ident_snake);

        let dyn_alias = quote! {
            dyn #trait_ident + Send + Sync + 'static
        };

        if arguments.concrete {
            let Ok(path) = syn::parse_str::<syn::Path>("::stain::AsAny") else {
                abort_call_site!("Unexpected error... couldn't build path to AsAny.");
            };

            let trait_bound = TypeParamBound::Trait(TraitBound {
                lifetimes: None,
                modifier: TraitBoundModifier::None,
                paren_token: None,
                path,
            });

            item_trait.supertraits.push(trait_bound);
        }

        let maybe_auto_impl = if arguments.concrete {
            Some(quote! {
                impl ::stain::AsAny for $target {
                    fn as_any(&self) -> &dyn ::std::any::Any {
                        self
                    }

                    fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                        self
                    }
                }
            })
        } else {
            None
        };

        let extra_attrs = self.extra_attrs;

        quote! {
            #(#extra_attrs)*
            #item_trait

            pub struct #newtype_entry(pub ::stain::Entry<Self>);

            impl #newtype_entry {
                pub const fn create_entry<T>(
                    lazy_boxed: &'static ::std::sync::LazyLock<::stain::parking_lot::RwLock<Box<#dyn_alias>>>,
                    lazy_type: &'static ::std::sync::LazyLock<::std::any::TypeId>,
                    name: &'static str,
                    ordering: u32,
                ) -> Self where T: #trait_ident {
                    Self(
                        ::stain::Entry::<Self>::create_entry(
                            lazy_boxed,
                            lazy_type,
                            name,
                            ordering,
                        )
                    )
                }
            }

            impl Clone for #newtype_entry {
                fn clone(&self) -> Self {
                    Self(self.0.clone())
                }
            }

            impl ::stain::IntoEntry for #newtype_entry {
                type Target = #dyn_alias;

                fn into_entry(self) -> ::stain::Entry<Self> {
                    self.0
                }
            }

            ::stain::inventory::collect!(#newtype_entry);

            #[allow(unused_macros)]
            macro_rules! #macro_ident {
                ($target:ty, $ordering:literal) => {
                    #maybe_auto_impl

                    #[allow(non_upper_case_globals)]
                    const _: () = {
                        static INIT: fn() -> $target = <$target as Default>::default;

                        static LAZY_BOXED: ::std::sync::LazyLock<::stain::parking_lot::RwLock<Box<#dyn_alias>>> =
                            ::std::sync::LazyLock::new(|| ::stain::parking_lot::RwLock::new(Box::new(INIT())));

                        static LAZY_TYPE: ::std::sync::LazyLock<::std::any::TypeId> =
                            ::std::sync::LazyLock::new(|| ::std::any::TypeId::of::<$target>());

                        ::stain::inventory::submit!(#module_path::#newtype_entry::create_entry::<$target>(
                            &LAZY_BOXED,
                            &LAZY_TYPE,
                            stringify!($target),
                            $ordering,
                        ));
                    };
                };
                ($target:ty) => {
                    #macro_ident!($target, 4294967295u32);
                };
            }

            #[allow(unused_imports)]
            pub(crate) use #macro_ident;
        }
    }
}

pub(crate) enum PluginTrait {
    Generic(GenericPlugin),
    Exact(ExactPlugin),
}

struct GenericAlias {
    span: Span,

    name: Box<str>,
    associated: Box<[AssocType]>,
    generics: Box<[Type]>,
}

impl Parse for GenericAlias {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let mut associated = Vec::<AssocType>::new();
        let mut generics = Vec::<Type>::new();

        let name = {
            let name = input.parse::<LitStr>()?;

            let before_name = name.value();
            let after_name = before_name.to_snake_case();

            if before_name != after_name {
                abort!(name.span(), "Alias name should be in snake case...");
            }

            after_name
        };

        input.parse::<Token![,]>()?;

        loop {
            let Some(assoc) = input.parse::<Option<Ident>>()? else {
                break;
            };

            let Some(eq) = input.parse::<Option<Token![=]>>()? else {
                break;
            };

            let ty = input.parse::<Type>()?;

            let assoc = AssocType {
                ident: assoc,
                generics: None,
                eq_token: eq,
                ty,
            };

            associated.push(assoc);

            input.parse::<Option<Token![,]>>()?;
        }

        while !input.is_empty() {
            let ty = input.parse::<Type>()?;
            input.parse::<Option<Token![,]>>()?;

            generics.push(ty);
        }

        Ok(Self {
            name: name.into_boxed_str(),
            associated: associated.into_boxed_slice(),
            generics: generics.into_boxed_slice(),
            span,
        })
    }
}

impl Parse for PluginTrait {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut aliases = Vec::<GenericAlias>::new();
        let mut extra_attrs = Vec::<Attribute>::new();

        for attr in input.call(Attribute::parse_outer).unwrap_or(Vec::new()) {
            if let Meta::List(ref list) = attr.meta {
                let ident_str = list.path.require_ident()?.to_string();

                match ident_str.as_str() {
                    "alias" => {
                        let alias = syn::parse2::<GenericAlias>(list.tokens.clone())?;
                        aliases.push(alias);

                        continue;
                    }
                    _ => {}
                }
            }

            extra_attrs.push(attr);
        }

        let item_trait = input.parse::<ItemTrait>()?;
        validate_trait(&item_trait);

        let associated = item_trait
            .items
            .iter()
            .filter_map(|item| {
                if let TraitItem::Type(assoc) = item {
                    Some(assoc.clone())
                } else {
                    None
                }
            })
            .collect::<Box<[_]>>();

        let generics = item_trait
            .generics
            .params
            .iter()
            .cloned()
            .collect::<Box<[_]>>();

        if !associated.is_empty() || !generics.is_empty() {
            if aliases.is_empty() {
                abort_call_site!("Use `alias` attribute to specify associated types and generics.");
            }

            for alias in aliases.iter() {
                for assoc in alias.associated.iter() {
                    if let None = associated.iter().find(|in_trait| in_trait.ident == assoc.ident) {
                        emit_error!(alias.span, "Associated type not found in trait...");
                    }
                }

                if associated.len() != alias.associated.len() {
                    emit_error!(alias.span, "Missing associated types...");
                }

                if generics.len() != alias.generics.len() {
                    emit_error!(alias.span, "Missing generic arguments...");
                }
            }

            abort_if_dirty();

            Ok(Self::Generic(GenericPlugin {
                extra_attrs: extra_attrs.into_boxed_slice(),
                aliases: aliases.into_boxed_slice(),
                item_trait,
            }))
        } else {
            Ok(Self::Exact(ExactPlugin {
                extra_attrs: extra_attrs.into_boxed_slice(),
                item_trait,
            }))
        }
    }
}

impl PluginTrait {
    pub(crate) fn into_tokens(self, arguments: CreateArguments) -> TokenStream {
        match self {
            PluginTrait::Exact(exact) => exact.into_tokens(arguments),
            PluginTrait::Generic(generic) => generic.into_tokens(arguments),
        }
    }
}
