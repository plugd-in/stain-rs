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

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};

use inventory::Collect;
use itertools::Itertools;

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

#[doc(hidden)]
pub use inventory;
#[doc(hidden)]
pub use parking_lot;
pub use stain_macros::*;

pub trait IntoEntry {
    type Target: ?Sized;

    fn into_entry(self) -> Entry<Self>
    where
        Self: Sized;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct Entry<T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    ordering: u32,
    type_id: &'static LazyLock<TypeId>,
    name: &'static str,
    boxed: &'static LazyLock<RwLock<Box<T::Target>>>,
}

impl<T> Clone for Entry<T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    fn clone(&self) -> Self {
        Self {
            ordering: self.ordering,
            type_id: self.type_id,
            name: self.name,
            boxed: self.boxed,
        }
    }
}

impl<T> Entry<T>
where
    T: IntoEntry,
    T::Target: Any + 'static,
{
    pub const fn create_entry(
        lazy_boxed: &'static LazyLock<RwLock<Box<T::Target>>>,
        lazy_type: &'static LazyLock<TypeId>,
        name: &'static str,
        ordering: u32,
    ) -> Self
    where
        T: Collect,
    {
        Self {
            ordering,
            name,
            type_id: lazy_type,
            boxed: lazy_boxed,
        }
    }

    fn read(&self) -> ReadEntry<'_, T> {
        ReadEntry::<'_, T> {
            name: self.name,
            boxed_read: self.boxed.read(),
        }
    }

    fn read_concrete<C>(&self) -> Option<ConcreteReadEntry<'_, C>>
    where
        C: 'static,
        T::Target: AsAny,
    {
        let mapped = RwLockReadGuard::try_map(self.boxed.read(), |boxed| {
            boxed.as_any().downcast_ref::<C>()
        })
        .ok()?;

        Some(ConcreteReadEntry::<'_, C> {
            name: self.name,
            read: mapped,
        })
    }

    fn write(&self) -> WriteEntry<'_, T> {
        WriteEntry::<'_, T> {
            name: self.name,
            boxed_write: self.boxed.write(),
        }
    }

    fn write_concrete<C>(&self) -> Option<ConcreteWriteEntry<'_, C>>
    where
        C: 'static,
        T::Target: AsAny,
    {
        let mapped = RwLockWriteGuard::try_map(self.boxed.write(), |boxed| {
            boxed.as_any_mut().downcast_mut::<C>()
        })
        .ok()?;

        Some(ConcreteWriteEntry::<'_, C> {
            name: self.name,
            write: mapped,
        })
    }
}

pub struct ReadEntry<'e, T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    name: &'static str,
    boxed_read: RwLockReadGuard<'e, Box<T::Target>>,
}

impl<T> ReadEntry<'_, T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    /// The name of the stained type.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> Deref for ReadEntry<'_, T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        &**self.boxed_read
    }
}

pub struct ConcreteReadEntry<'e, T> {
    name: &'static str,
    read: MappedRwLockReadGuard<'e, T>,
}

impl<T> ConcreteReadEntry<'_, T> {
    /// The name of the stained type.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> Deref for ConcreteReadEntry<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.read
    }
}

pub struct WriteEntry<'e, T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    name: &'static str,
    boxed_write: RwLockWriteGuard<'e, Box<T::Target>>,
}

impl<T> WriteEntry<'_, T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    /// The name of the stained type.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> Deref for WriteEntry<'_, T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        &**self.boxed_write
    }
}

impl<T> DerefMut for WriteEntry<'_, T>
where
    T: IntoEntry,
    T::Target: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut **self.boxed_write
    }
}

pub struct ConcreteWriteEntry<'e, T> {
    name: &'static str,
    write: MappedRwLockWriteGuard<'e, T>,
}

impl<T> ConcreteWriteEntry<'_, T> {
    /// The name of the stained type.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl<T> Deref for ConcreteWriteEntry<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.write
    }
}

impl<T> DerefMut for ConcreteWriteEntry<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.write
    }
}

#[derive(Clone)]
pub struct Store<T>
where
    T: 'static,
    T: IntoEntry,
{
    store: HashMap<TypeId, Entry<T>>,
}

impl<T> Store<T>
where
    T: IntoEntry,
{
    /// Collect all the types stained with `T`.
    pub fn collect() -> Self
    where
        T: Collect + Clone,
    {
        let entries = inventory::iter::<T>()
            .cloned()
            .map(|entry| entry.into_entry())
            .map(|entry| (**entry.type_id, entry))
            .collect();

        Self { store: entries }
    }

    /// Returns whether the given type, `C`, is stained with `T`.
    pub fn stained<C>(&self) -> bool
    where
        C: Any,
    {
        let type_id = TypeId::of::<C>();

        self.store.get(&type_id).is_some()
    }

    pub fn get<C>(&self) -> Option<ConcreteReadEntry<'_, C>>
    where
        C: Any,
        C: AsAny,
        T::Target: AsAny,
    {
        let type_id = TypeId::of::<C>();
        let entry = self.store.get(&type_id)?;

        entry.read_concrete()
    }

    pub fn get_mut<C>(&self) -> Option<ConcreteWriteEntry<'_, C>>
    where
        C: Any,
        C: AsAny,
        T::Target: AsAny,
    {
        let type_id = TypeId::of::<C>();
        let entry = self.store.get(&type_id)?;

        entry.write_concrete()
    }

    pub fn iter(&self) -> impl Iterator<Item = ReadEntry<'_, T>> {
        self.store
            .values()
            .sorted_by(|a, b| Ord::cmp(&a.ordering, &b.ordering))
            .map(|entry| entry.read())
    }

    pub fn iter_mut(&self) -> impl Iterator<Item = WriteEntry<'_, T>> {
        self.store
            .values()
            .sorted_by(|a, b| Ord::cmp(&a.ordering, &b.ordering))
            .map(|entry| entry.write())
    }
}
