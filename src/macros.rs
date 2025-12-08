/// Creates a storage module for a specific trait.
///
/// This macro generates the infrastructure required to collect plugin implementations.
/// It creates a module (named by the last argument) containing a `Store` struct.
///
/// # Supported Syntaxes
///
/// ## 1. Minimal Configuration
///
/// The simplest form requires only the Trait and the name of the module to generate.
/// The `ordering` defaults to `u64`.
///
/// ```rust
/// use stain::{create_stain, stain, Store};
///
/// pub trait Logger { fn log(&self); }
///
/// // Creates a module named `log_store`
/// create_stain! {
///     trait Logger;
///     store: pub mod log_store;
/// }
///
/// #[derive(Default)]
/// struct StdOutLogger;
/// impl Logger for StdOutLogger { fn log(&self) {} }
///
/// stain! {
///     store: log_store;
///     item: StdOutLogger;
///     ordering: 0; // Default ordering is u64
/// }
///
/// fn main() {
///     let store = log_store::Store::collect();
/// }
/// ```
///
/// ## 2. Custom Ordering Type
///
/// You can specify a custom type for ordering. It must implement `Ord + Clone`.
///
/// ```rust
/// use stain::{create_stain, stain, Store};
///
/// trait Job { fn run(&self); }
///
/// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
/// pub enum Priority { Low, High, Critical }
///
/// create_stain! {
///     trait Job;
///     ordering: Priority; // Use enum for ordering
///     store: mod job_store;
/// }
///
/// #[derive(Default)]
/// struct UrgentJob;
/// impl Job for UrgentJob { fn run(&self) {} }
///
/// stain! {
///     store: job_store;
///     item: UrgentJob;
///     ordering: Priority::Critical;
/// }
///
/// fn main() {
///     let store = job_store::Store::collect();
/// }
/// ```
///
/// ## 3. Generics and Associated Types (GATs)
///
/// `stain` supports generics on the trait and Generic Associated Types.
/// You must explicitly map them in the macro invocation.
///
/// ```rust
/// use stain::{create_stain, stain, Store};
///
/// pub trait Converter<Input> {
///     type Output;
///     fn convert(&self, i: Input) -> Self::Output;
/// }
///
/// create_stain! {
///     trait Converter;
///     // Declare the Generic used by the trait
///     type String;
///     // Bind the Associated Type to a concrete type for this store
///     trait type Output = usize;
///     store: pub(crate) mod converter_store;
/// }
///
/// #[derive(Default)]
/// struct StringLen;
/// impl Converter<String> for StringLen {
///     type Output = usize;
///     fn convert(&self, s: String) -> usize { s.len() }
/// }
///
/// stain! {
///     store: converter_store;
///     item: StringLen;
///     ordering: 0;
/// }
///
/// fn main() {
///     let store = converter_store::Store::collect();
/// }
/// ```
///
/// ## 4. Prefixes
///
/// If you have multiple stain stores in your binary, `linkme` might collision
/// on symbol names. You can add a `prefix` to namespace the linker symbols.
///
/// ```rust
/// use stain::{create_stain, Store};
///
/// pub(self) trait Hook {}
///
/// create_stain! {
///     trait Hook;
///     prefix: my_system_hooks;
///     store: pub(self) mod hook_store;
/// }
///
/// fn main() {
///     let store = hook_store::Store::collect();
/// }
/// ```
///
/// ## 5. Visibility
///
/// Visibility can be specified by adding a visibility to the store declaration.
/// Supported visibilities are `pub`, `pub(crate)`, `pub(super)`, `pub(self)`,
/// `pub(in self)`, and `` (empty, i.e. `pub(self)`).
///
/// All this does is add a visibility to the generated module and makes sure
/// that all the generated items in the module have the correct visibility, too.
#[macro_export]
macro_rules! create_stain {
    (
        // The trait for which the trait-object plugin store
        // should be generated.
        trait $trait:ident;
        // Some type that can be ordered via Ord, used to
        // enable ordered plugin execution.
        //
        // Customization is enabled so you can, for example,
        // use runtime values (e.g. enums) to address specific plugins.
        ordering: $ordering:ty;

        // Syntax for specifying trait generics.
        $(type $generic:ty;)*
        // Syntax for specifying Generic Associated Types (GATs).
        $(trait type $associated:ident = $associated_type:ty;)*

        // An optional prefix that acts as a namespace
        // for the [linkme] section.
        prefix$(: $prefix:ident)?;
        // The module declaration for the generated module
        // that will hold the generated store.
        store: pub mod $store:ident;
    ) => {
        $crate::paste! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ITEM >] = dyn $trait<
                $($generic,)*
                $($associated = $associated_type,)*
            > + Send + Sync;

            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ORDERING >] = $ordering;

            pub mod $store {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ITEM = super::[< __STAIN_ $store:upper _ITEM >];
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ORDERING = super::[<__STAIN_ $store:upper _ORDERING>];

                #[$crate::linkme::distributed_slice]
                #[linkme(crate = $crate::linkme)]
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                pub static [< __STAIN_ $($prefix:upper)? _ $store:upper >]: [$crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>];

                #[doc(hidden)]
                pub use [< __STAIN_ $($prefix:upper)? _ $store:upper >] as __STAIN_COLLECTION;

                pub struct Store {
                    entries: std::collections::BTreeMap<
                        __STAIN_ORDERING,
                        std::vec::Vec<&'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>>,
                    >,
                    type_map: std::collections::HashMap<
                        std::any::TypeId,
                        &'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>
                    >,
                }

                impl $crate::Store for Store {
                    // Define the associated types based on macro input
                    type Item = __STAIN_ITEM;
                    type Ordering = __STAIN_ORDERING;

                    fn collect() -> Self {
                        use std::ops::Deref;
                        use $crate::itertools::Itertools;

                        // Note: accessing the slice via the static name generated above
                        let type_map = [< __STAIN_ $($prefix:upper)? _ $store:upper >].deref()
                            .into_iter()
                            .map(|entry| (entry.type_id(), entry))
                            .collect::<std::collections::HashMap<
                                std::any::TypeId,
                                &'static $crate::Entry::<Self::Ordering, Self::Item>
                            >>();

                        let entries = type_map
                            .values()
                            .cloned()
                            .sorted()
                            .chunk_by(|entry| entry.ordering().clone())
                            .into_iter()
                            .map(|(ordering, entries)| (ordering, entries.collect()))
                            .collect();

                        Self {
                            entries,
                            type_map,
                        }
                    }

                    fn iter(&self) -> impl std::iter::Iterator<
                        Item = $crate::EntryRef<'_, Self::Ordering, Self::Item>
                    > {
                        self.entries
                            .values()
                            .map(|entries| entries.iter())
                            .flatten()
                            .map(|entry| *entry)
                            .map($crate::EntryRef::from)
                    }

                    fn ordering<'a>(&'a self, ordering: &Self::Ordering) -> Option<
                        impl std::iter::Iterator<
                            Item = $crate::EntryRef<'a, Self::Ordering, Self::Item>
                        > + 'a
                    > {
                        let entries = self.entries.get(ordering)?;
                        Some(
                            entries
                                .iter()
                                .map(|entry| *entry)
                                .map($crate::EntryRef::from)
                        )
                    }

                    fn concrete<T: std::any::Any + Send + Sync>(&self) -> Option<
                        $crate::ConcreteEntryRef<'_, T>
                    > {
                        self.type_map
                            .get(&std::any::TypeId::of::<T>())?
                            .concrete::<T>()
                    }
                }
            }
        }
    };

    (
        // The trait for which the trait-object plugin store
        // should be generated.
        trait $trait:ident;
        // Some type that can be ordered via Ord, used to
        // enable ordered plugin execution.
        //
        // Customization is enabled so you can, for example,
        // use runtime values (e.g. enums) to address specific plugins.
        ordering: $ordering:ty;

        // Syntax for specifying trait generics.
        $(type $generic:ty;)*
        // Syntax for specifying Generic Associated Types (GATs).
        $(trait type $associated:ident = $associated_type:ty;)*

        // An optional prefix that acts as a namespace
        // for the [linkme] section.
        prefix$(: $prefix:ident)?;
        // The module declaration for the generated module
        // that will hold the generated store.
        store: pub(crate) mod $store:ident;
    ) => {
        $crate::paste! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ITEM >] = dyn $trait<
                $($generic,)*
                $($associated = $associated_type,)*
            > + Send + Sync;

            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ORDERING >] = $ordering;

            pub(crate) mod $store {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ITEM = super::[< __STAIN_ $store:upper _ITEM >];
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ORDERING = super::[<__STAIN_ $store:upper _ORDERING>];

                #[$crate::linkme::distributed_slice]
                #[linkme(crate = $crate::linkme)]
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                pub(crate) static [< __STAIN_ $($prefix:upper)? _ $store:upper >]: [$crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>];

                #[doc(hidden)]
                pub(crate) use [< __STAIN_ $($prefix:upper)? _ $store:upper >] as __STAIN_COLLECTION;

                pub(crate) struct Store {
                    entries: std::collections::BTreeMap<
                        __STAIN_ORDERING,
                        std::vec::Vec<&'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>>,
                    >,
                    type_map: std::collections::HashMap<
                        std::any::TypeId,
                        &'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>
                    >,
                }

                impl $crate::Store for Store {
                    // Define the associated types based on macro input
                    type Item = __STAIN_ITEM;
                    type Ordering = __STAIN_ORDERING;

                    fn collect() -> Self {
                        use std::ops::Deref;
                        use $crate::itertools::Itertools;

                        // Note: accessing the slice via the static name generated above
                        let type_map = [< __STAIN_ $($prefix:upper)? _ $store:upper >].deref()
                            .into_iter()
                            .map(|entry| (entry.type_id(), entry))
                            .collect::<std::collections::HashMap<
                                std::any::TypeId,
                                &'static $crate::Entry::<Self::Ordering, Self::Item>
                            >>();

                        let entries = type_map
                            .values()
                            .cloned()
                            .sorted()
                            .chunk_by(|entry| entry.ordering().clone())
                            .into_iter()
                            .map(|(ordering, entries)| (ordering, entries.collect()))
                            .collect();

                        Self {
                            entries,
                            type_map,
                        }
                    }

                    fn iter(&self) -> impl std::iter::Iterator<
                        Item = $crate::EntryRef<'_, Self::Ordering, Self::Item>
                    > {
                        self.entries
                            .values()
                            .map(|entries| entries.iter())
                            .flatten()
                            .map(|entry| *entry)
                            .map($crate::EntryRef::from)
                    }

                    fn ordering<'a>(&'a self, ordering: &Self::Ordering) -> Option<
                        impl std::iter::Iterator<
                            Item = $crate::EntryRef<'a, Self::Ordering, Self::Item>
                        > + 'a
                    > {
                        let entries = self.entries.get(ordering)?;
                        Some(
                            entries
                                .iter()
                                .map(|entry| *entry)
                                .map($crate::EntryRef::from)
                        )
                    }

                    fn concrete<T: std::any::Any + Send + Sync>(&self) -> Option<
                        $crate::ConcreteEntryRef<'_, T>
                    > {
                        self.type_map
                            .get(&std::any::TypeId::of::<T>())?
                            .concrete::<T>()
                    }
                }
            }
        }
    };

    (
        // The trait for which the trait-object plugin store
        // should be generated.
        trait $trait:ident;
        // Some type that can be ordered via Ord, used to
        // enable ordered plugin execution.
        //
        // Customization is enabled so you can, for example,
        // use runtime values (e.g. enums) to address specific plugins.
        ordering: $ordering:ty;

        // Syntax for specifying trait generics.
        $(type $generic:ty;)*
        // Syntax for specifying Generic Associated Types (GATs).
        $(trait type $associated:ident = $associated_type:ty;)*

        // An optional prefix that acts as a namespace
        // for the [linkme] section.
        prefix$(: $prefix:ident)?;
        // The module declaration for the generated module
        // that will hold the generated store.
        store: pub(super) mod $store:ident;
    ) => {
        $crate::paste! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ITEM >] = dyn $trait<
                $($generic,)*
                $($associated = $associated_type,)*
            > + Send + Sync;

            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ORDERING >] = $ordering;

            pub(super) mod $store {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ITEM = super::[< __STAIN_ $store:upper _ITEM >];
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ORDERING = super::[<__STAIN_ $store:upper _ORDERING>];

                #[$crate::linkme::distributed_slice]
                #[linkme(crate = $crate::linkme)]
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                pub(in super::super) static [< __STAIN_ $($prefix:upper)? _ $store:upper >]: [$crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>];

                #[doc(hidden)]
                pub(in super::super) use [< __STAIN_ $($prefix:upper)? _ $store:upper >] as __STAIN_COLLECTION;

                pub(in super::super) struct Store {
                    entries: std::collections::BTreeMap<
                        __STAIN_ORDERING,
                        std::vec::Vec<&'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>>,
                    >,
                    type_map: std::collections::HashMap<
                        std::any::TypeId,
                        &'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>
                    >,
                }

                impl $crate::Store for Store {
                    // Define the associated types based on macro input
                    type Item = __STAIN_ITEM;
                    type Ordering = __STAIN_ORDERING;

                    fn collect() -> Self {
                        use std::ops::Deref;
                        use $crate::itertools::Itertools;

                        // Note: accessing the slice via the static name generated above
                        let type_map = [< __STAIN_ $($prefix:upper)? _ $store:upper >].deref()
                            .into_iter()
                            .map(|entry| (entry.type_id(), entry))
                            .collect::<std::collections::HashMap<
                                std::any::TypeId,
                                &'static $crate::Entry::<Self::Ordering, Self::Item>
                            >>();

                        let entries = type_map
                            .values()
                            .cloned()
                            .sorted()
                            .chunk_by(|entry| entry.ordering().clone())
                            .into_iter()
                            .map(|(ordering, entries)| (ordering, entries.collect()))
                            .collect();

                        Self {
                            entries,
                            type_map,
                        }
                    }

                    fn iter(&self) -> impl std::iter::Iterator<
                        Item = $crate::EntryRef<'_, Self::Ordering, Self::Item>
                    > {
                        self.entries
                            .values()
                            .map(|entries| entries.iter())
                            .flatten()
                            .map(|entry| *entry)
                            .map($crate::EntryRef::from)
                    }

                    fn ordering<'a>(&'a self, ordering: &Self::Ordering) -> Option<
                        impl std::iter::Iterator<
                            Item = $crate::EntryRef<'a, Self::Ordering, Self::Item>
                        > + 'a
                    > {
                        let entries = self.entries.get(ordering)?;
                        Some(
                            entries
                                .iter()
                                .map(|entry| *entry)
                                .map($crate::EntryRef::from)
                        )
                    }

                    fn concrete<T: std::any::Any + Send + Sync>(&self) -> Option<
                        $crate::ConcreteEntryRef<'_, T>
                    > {
                        self.type_map
                            .get(&std::any::TypeId::of::<T>())?
                            .concrete::<T>()
                    }
                }
            }
        }
    };

    (
        // The trait for which the trait-object plugin store
        // should be generated.
        trait $trait:ident;
        // Some type that can be ordered via Ord, used to
        // enable ordered plugin execution.
        //
        // Customization is enabled so you can, for example,
        // use runtime values (e.g. enums) to address specific plugins.
        ordering: $ordering:ty;

        // Syntax for specifying trait generics.
        $(type $generic:ty;)*
        // Syntax for specifying Generic Associated Types (GATs).
        $(trait type $associated:ident = $associated_type:ty;)*

        // An optional prefix that acts as a namespace
        // for the [linkme] section.
        prefix$(: $prefix:ident)?;
        // The module declaration for the generated module
        // that will hold the generated store.
        store: mod $store:ident;
    ) => {
        $crate::paste! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ITEM >] = dyn $trait<
                $($generic,)*
                $($associated = $associated_type,)*
            > + Send + Sync;

            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            type [< __STAIN_ $store:upper _ORDERING >] = $ordering;

            mod $store {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ITEM = super::[< __STAIN_ $store:upper _ITEM >];
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __STAIN_ORDERING = super::[<__STAIN_ $store:upper _ORDERING>];

                #[$crate::linkme::distributed_slice]
                #[linkme(crate = $crate::linkme)]
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                pub(super) static [< __STAIN_ $($prefix:upper)? _ $store:upper >]: [$crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>];

                #[doc(hidden)]
                pub(super) use [< __STAIN_ $($prefix:upper)? _ $store:upper >] as __STAIN_COLLECTION;

                pub(super) struct Store {
                    entries: std::collections::BTreeMap<
                        __STAIN_ORDERING,
                        std::vec::Vec<&'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>>,
                    >,
                    type_map: std::collections::HashMap<
                        std::any::TypeId,
                        &'static $crate::Entry::<__STAIN_ORDERING, __STAIN_ITEM>
                    >,
                }

                impl $crate::Store for Store {
                    // Define the associated types based on macro input
                    type Item = __STAIN_ITEM;
                    type Ordering = __STAIN_ORDERING;

                    fn collect() -> Self {
                        use std::ops::Deref;
                        use $crate::itertools::Itertools;

                        // Note: accessing the slice via the static name generated above
                        let type_map = [< __STAIN_ $($prefix:upper)? _ $store:upper >].deref()
                            .into_iter()
                            .map(|entry| (entry.type_id(), entry))
                            .collect::<std::collections::HashMap<
                                std::any::TypeId,
                                &'static $crate::Entry::<Self::Ordering, Self::Item>
                            >>();

                        let entries = type_map
                            .values()
                            .cloned()
                            .sorted()
                            .chunk_by(|entry| entry.ordering().clone())
                            .into_iter()
                            .map(|(ordering, entries)| (ordering, entries.collect()))
                            .collect();

                        Self {
                            entries,
                            type_map,
                        }
                    }

                    fn iter(&self) -> impl std::iter::Iterator<
                        Item = $crate::EntryRef<'_, Self::Ordering, Self::Item>
                    > {
                        self.entries
                            .values()
                            .map(|entries| entries.iter())
                            .flatten()
                            .map(|entry| *entry)
                            .map($crate::EntryRef::from)
                    }

                    fn ordering<'a>(&'a self, ordering: &Self::Ordering) -> Option<
                        impl std::iter::Iterator<
                            Item = $crate::EntryRef<'a, Self::Ordering, Self::Item>
                        > + 'a
                    > {
                        let entries = self.entries.get(ordering)?;
                        Some(
                            entries
                                .iter()
                                .map(|entry| *entry)
                                .map($crate::EntryRef::from)
                        )
                    }

                    fn concrete<T: std::any::Any + Send + Sync>(&self) -> Option<
                        $crate::ConcreteEntryRef<'_, T>
                    > {
                        self.type_map
                            .get(&std::any::TypeId::of::<T>())?
                            .concrete::<T>()
                    }
                }
            }
        }
    };

    (
        // The trait for which the trait-object plugin store
        // should be generated.
        trait $trait:ident;
        // Some type that can be ordered via Ord, used to
        // enable ordered plugin execution.
        //
        // Customization is enabled so you can, for example,
        // use runtime values (e.g. enums) to address specific plugins.
        ordering: $ordering:ty;

        // Syntax for specifying trait generics.
        $(type $generic:ty;)*
        // Syntax for specifying Generic Associated Types (GATs).
        $(trait type $associated:ident = $associated_type:ty;)*

        // An optional prefix that acts as a namespace
        // for the [linkme] section.
        prefix$(: $prefix:ident)?;
        // The module declaration for the generated module
        // that will hold the generated store.
        store: pub(self) mod $store:ident;
    ) => {
        $crate::create_stain {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: mod $store;
        }
    };

    (
        // The trait for which the trait-object plugin store
        // should be generated.
        trait $trait:ident;
        // Some type that can be ordered via Ord, used to
        // enable ordered plugin execution.
        //
        // Customization is enabled so you can, for example,
        // use runtime values (e.g. enums) to address specific plugins.
        ordering: $ordering:ty;

        // Syntax for specifying trait generics.
        $(type $generic:ty;)*
        // Syntax for specifying Generic Associated Types (GATs).
        $(trait type $associated:ident = $associated_type:ty;)*

        // An optional prefix that acts as a namespace
        // for the [linkme] section.
        prefix$(: $prefix:ident)?;
        // The module declaration for the generated module
        // that will hold the generated store.
        store: pub(in self) mod $store:ident;
    ) => {
        $crate::create_stain {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: mod $store;
        }
    };

    // Optional prefix...
    (
        trait $trait:ident;
        ordering: $ordering:ty;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: mod $store;
        }
    };

    // Optional prefix (pub)...
    (
        trait $trait:ident;
        ordering: $ordering:ty;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: pub mod $store;
        }
    };

    // Optional prefix (pub(crate))...
    (
        trait $trait:ident;
        ordering: $ordering:ty;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(crate) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: pub(crate) mod $store;
        }
    };

    // Optional prefix (pub(super))...
    (
        trait $trait:ident;
        ordering: $ordering:ty;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(super) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: pub(super) mod $store;
        }
    };

    // Optional prefix (pub(self))...
    (
        trait $trait:ident;
        ordering: $ordering:ty;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(self) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: mod $store;
        }
    };

    // Optional prefix (pub(in self))...
    (
        trait $trait:ident;
        ordering: $ordering:ty;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(in self) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: $ordering;

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: mod $store;
        }
    };

    // Optional ordering...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        prefix$(: $prefix:ident)?;
        store: mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: mod $store;
        }
    };

    // Optional ordering (pub)...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        prefix$(: $prefix:ident)?;
        store: pub mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: pub mod $store;
        }
    };

    // Optional ordering (pub(crate))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        prefix$(: $prefix:ident)?;
        store: pub(crate) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: pub(crate) mod $store;
        }
    };

    // Optional ordering (pub(super))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        prefix$(: $prefix:ident)?;
        store: pub(super) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: pub(super) mod $store;
        }
    };

    // Optional ordering (pub(self))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        prefix$(: $prefix:ident)?;
        store: pub(self) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: mod $store;
        }
    };

    // Optional ordering (pub(in self))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        prefix$(: $prefix:ident)?;
        store: pub(in self) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix$(: $prefix)?;
            store: mod $store;
        }
    };

    // Optional ordering and optional prefix...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: mod $store;
        }
    };

    // Optional ordering and optional prefix (pub)...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: pub mod $store;
        }
    };

    // Optional ordering and optional prefix (pub (crate))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(crate) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: pub(crate) mod $store;
        }
    };

    // Optional ordering and optional prefix (pub(super))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(super) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: pub(super) mod $store;
        }
    };

    // Optional ordering and optional prefix (pub(self))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(self) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: mod $store;
        }
    };

    // Optional ordering and optional prefix (pub(in self))...
    (
        trait $trait:ident;

        $(type $generic:ty;)*
        $(trait type $associated:ident = $associated_type:ty;)*

        store: pub(in self) mod $store:ident;
    ) => {
        $crate::create_stain! {
            trait $trait;
            ordering: u64; // Injected default

            $(type $generic;)*
            $(trait type $associated = $associated_type;)*

            prefix; // Injected empty prefix
            store: mod $store;
        }
    };
}

#[macro_export]
macro_rules! stain {
    (
        // The generated store. Used to get Store::Ordering
        // type for the static typing.
        store: $store:ident;
        // The concrete implementation/type to
        // stain/register in the collection.
        item: $item:ident;
        // The ordering to apply to this implementation.
        ordering: $order:expr;
    ) => {
        $crate::paste! {
            #[$crate::rustversion::before(1.91)]
            const _: () = {
                use std::any::Any;
                use std::sync::Arc;

                fn __stain_init() -> (
                    Arc<<$store::Store as $crate::Store>::Item>,
                    Arc<dyn Any + Send + Sync>,
                ) {
                    let instance: $item = Default::default();
                    let shared_instance = Arc::new(instance);

                    let trait_view = shared_instance.clone() as Arc<<$store::Store as $crate::Store>::Item>;
                    let any_view = shared_instance as Arc<dyn Any + Send + Sync>;

                    (trait_view, any_view)
                }

                #[$crate::linkme::distributed_slice($store::__STAIN_COLLECTION)]
                #[linkme(crate = $crate::linkme)]
                pub static _STAIN: $crate::Entry<
                    <$store::Store as $crate::Store>::Ordering,
                    <$store::Store as $crate::Store>::Item,
                > =
                $crate::Entry::<_,<$store::Store as $crate::Store>::Item>::new(
                    || std::any::TypeId::of::<$item>(),
                    $order,
                    stringify!($item),
                    __stain_init,
                );
            };

            #[$crate::rustversion::since(1.91)]
            const _: () = {
                use std::any::Any;
                use std::sync::Arc;

                fn __stain_init() -> (
                    Arc<<$store::Store as $crate::Store>::Item>,
                    Arc<dyn Any + Send + Sync>,
                ) {
                    let instance: $item = Default::default();
                    let shared_instance = Arc::new(instance);

                    let trait_view = shared_instance.clone() as Arc<<$store::Store as $crate::Store>::Item>;
                    let any_view = shared_instance as Arc<dyn Any + Send + Sync>;

                    (trait_view, any_view)
                }

                #[$crate::linkme::distributed_slice($store::__STAIN_COLLECTION)]
                #[linkme(crate = $crate::linkme)]
                pub static _STAIN: $crate::Entry<
                    <$store::Store as $crate::Store>::Ordering,
                    <$store::Store as $crate::Store>::Item,
                > =
                $crate::Entry::<_,<$store::Store as $crate::Store>::Item>::new(
                    std::any::TypeId::of::<$item>(),
                    $order,
                    stringify!($item),
                    __stain_init,
                );
            };
        }
    };
}
