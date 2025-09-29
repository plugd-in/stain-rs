//! Utils for help creating the macros...

use proc_macro_error::{abort_call_site, abort_if_dirty, emit_error};
use proc_macro2::Span;
use syn::{
    FnArg, Ident, ItemTrait, PathSegment, ReturnType, TraitItem, TraitItemFn, TraitItemType, Type,
    TypeParamBound, WherePredicate,
};

/// Validates that an associated type is dyn-compatible.
fn validate_trait_type(item_type: &TraitItemType) {
    // Rule #7: Associated types cannot have generic parameters.
    if !item_type.generics.params.is_empty() {
        emit_error!(
            item_type.generics,
            "Associated types in a dyn-compatible trait cannot have generic parameters."
        );
    }
}

/// Validates that a trait method is dyn-compatible.
fn validate_trait_fn(method: &TraitItemFn) {
    // Check for a `where Self: Sized` clause. If it exists, this method
    // is not part of the dyn trait vtable, so we can skip the other checks.
    let mut is_opted_out = false;
    if let Some(where_clause) = &method.sig.generics.where_clause {
        for predicate in &where_clause.predicates {
            if let WherePredicate::Type(p) = predicate {
                if let Type::Path(type_path) = &p.bounded_ty {
                    if type_path.path.is_ident("Self") {
                        for bound in &p.bounds {
                            if let TypeParamBound::Trait(trait_bound) = bound {
                                if trait_bound.path.is_ident("Sized") {
                                    is_opted_out = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if is_opted_out {
                break;
            }
        }
    }

    if is_opted_out {
        return; // This method is not dyn-compatible, so skip it.
    }

    // Rule #6: The method cannot return `impl Trait`.
    if let ReturnType::Type(_, return_type) = &method.sig.output {
        if let Type::ImplTrait(_) = &**return_type {
            emit_error!(
                return_type,
                "Methods in a dyn-compatible trait cannot use `impl Trait` in return position.";

                help = "Consider returning a boxed trait object, e.g., `-> Box<dyn SomeTrait>`,";
                help = "or adding a `where Self: Sized` bound to the method.";
            );
        }
    }

    // Rule #5: The method cannot be `async`.
    if let Some(async_token) = &method.sig.asyncness {
        emit_error!(
            async_token,
            "Methods in a dyn-compatible trait cannot be `async`.";

            help = "Consider using `async-trait` or adding a `where Self: Sized` bound to the method.";
        );
    }

    // Rule #4: The method cannot have its own generic parameters.
    if !method.sig.generics.params.is_empty() {
        emit_error!(
            method.sig.generics,
            "Methods in a dyn-compatible trait cannot have generic parameters.";

            help = "Consider adding a `where Self: Sized` bound to the method.";
        );
    }

    // Rule #3: The method cannot take `self` by value.
    if let Some(FnArg::Receiver(receiver)) = method.sig.inputs.first() {
        if receiver.reference.is_none() {
            emit_error!(
                receiver,
                "Methods in a dyn-compatible trait cannot take `self` by value.";

                help = "Use `&self` or `&mut self` instead,";
                help = "or add a `where Self: Sized` bound to the method.";
            );
        }
    }
}

/// Ensures the given trait is dyn-compatible and provides helpful
/// error messages if it isn't.
///
/// This function checks for the following object-safety rules.
///
/// Trait-level rules:
/// 1. The trait must not have `Sized` as a supertrait.
/// 2. The trait cannot have lifetime parameters.
///
/// Item-level rules:
/// 3. Methods cannot take `self` by value.
/// 4. Methods cannot have generic parameters.
/// 5. Methods cannot be `async`.
/// 6. Methods cannot use `impl Trait` in return position (RPIT).
/// 7. Associated types cannot have generic parameters.
/// 8. Associated constants are not permitted for this system.
///
/// Note: A method can be exempted from rules 3-6 if it includes a `where Self: Sized` clause.
pub(crate) fn validate_trait(item_trait: &ItemTrait) {
    // Rule #1: The trait cannot require `Sized`.
    for supertrait in &item_trait.supertraits {
        if let TypeParamBound::Trait(trait_bound) = supertrait {
            if trait_bound.path.is_ident("Sized") {
                emit_error!(
                    supertrait,
                    "A trait cannot require `Sized` to be made into a dyn-compatible object."
                );
            }
        }
    }

    // Rule #2: The trait cannot have lifetime parameters.
    for lifetime in item_trait.generics.lifetimes() {
        emit_error!(lifetime, "Lifetimes aren't supported for stains.");
    }

    // Check all items defined within the trait.
    for item in &item_trait.items {
        match item {
            TraitItem::Fn(method) => validate_trait_fn(method),
            TraitItem::Type(item_type) => validate_trait_type(item_type),
            // Rule #7: Associated constants are not permitted for this system.
            TraitItem::Const(item_const) => {
                emit_error!(item_const, "Associated constants are not dyn-compatible.");
            }
            _ => {}
        }
    }

    abort_if_dirty();
}

pub(crate) fn module_path(span: Span, src_prefix: &str) -> Option<syn::Path> {
    let path = span.local_file()?;

    let Ok(path) = path.strip_prefix(src_prefix) else {
        abort_call_site! {
            "Unable to parse module path...";

            help = "Specify `src_prefix`...";
        };
    };

    // Remove the `.rs` extension.
    let path = path.with_extension("");

    let file_stem = path.file_stem().and_then(|s| s.to_str());
    let module_path = if file_stem == Some("lib") || file_stem == Some("main") {
        vec![Ident::new("crate", span)]
    } else {
        let mut module_path_parts = vec![Ident::new("crate", span)];

        for part in path.components() {
            module_path_parts.push(Ident::new(
                part.as_os_str().to_string_lossy().as_ref(),
                span,
            ));
        }

        if module_path_parts.last().map_or(false, |s| s == "mod") {
            module_path_parts.pop();
        }

        module_path_parts
    };

    let module_path = module_path.into_iter().map(PathSegment::from);

    Some(syn::Path {
        leading_colon: None,
        segments: module_path.collect(),
    })
}
