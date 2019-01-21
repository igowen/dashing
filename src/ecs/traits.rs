// Copyright 2018 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::ecs::typelist::*;
use crate::ecs::*;

use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};

/// Specifies how a component is stored.
///
/// This is automatically implemented for component types by `define_world!`; you shouldn't ever
/// need to implement it manually.
pub trait StorageSpec<'a> {
    /// The component type.
    type Component: 'a;
    /// The storage type for this Component.
    type Storage: ComponentStorage<'a, Self::Component>;
}

/// Read-only view of a Component.
pub struct ReadComponent<'a, T: StorageSpec<'a>> {
    storage: Ref<'a, T::Storage>,
}

impl<'a, T> ReadComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
    T::Storage: ComponentStorage<'a, T>,
{
    /// Get a reference to the underlying `Storage`. This is an associated method because
    /// `ReadComponent` implements `Deref` and `ComponentStorage` also has a method called `get()`.
    pub fn get(v: &Self) -> Ref<T::Storage> {
        Ref::clone(&v.storage)
    }
}

impl<'a, T> Deref for ReadComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
    T::Storage: ComponentStorage<'a, T>,
{
    type Target = T::Storage;
    fn deref(&self) -> &T::Storage {
        Deref::deref(&self.storage)
    }
}

/// Write component
pub struct WriteComponent<'a, T: 'a + StorageSpec<'a>> {
    storage: RefMut<'a, T::Storage>,
}

impl<'a, T> WriteComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
    T::Storage: ComponentStorage<'a, T>,
{
    /// Get a reference to the underlying `Storage`. This is an associated method because
    /// `ReadComponent` implements `Deref`.
    ///
    /// Since `RefMut` cannot be cloned (write access must be exclusive), this consumes its
    /// argument.
    pub fn get_mut(v: Self) -> RefMut<'a, T::Storage> {
        v.storage
    }
}

impl<'a, T> Deref for WriteComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
    T::Storage: ComponentStorage<'a, T>,
{
    type Target = T::Storage;
    fn deref(&self) -> &T::Storage {
        Deref::deref(&self.storage)
    }
}

impl<'a, T> DerefMut for WriteComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
    T::Storage: ComponentStorage<'a, T>,
{
    fn deref_mut(&mut self) -> &mut T::Storage {
        DerefMut::deref_mut(&mut self.storage)
    }
}

/// Trait that allows us to convert flat tuple types to nested tuple types (e.g.,
/// `(A, B, C)` → `(A, (B, (C, ())))`).
///
/// Also provides an associated function to convert a nested tuple **by value** to the equivalent
/// flat tuple.
///
/// This trait is provided for tuples up to length 32.
pub trait Unflatten: private::Sealed {
    /// Equivalent nested tuple type.
    type Unflattened;
    /// Flatten the thing back out.
    fn reflatten(v: Self::Unflattened) -> Self;
}

// helper macro for `impl_unflatten!`.
macro_rules! unnest {
    (($layer:expr); ($($v:expr),*); ($u:ident, $($us:ident,)*)) => {
        unnest!(($layer . 1); ($($v,)* $layer.0); ($($us,)*))
    };
    (($layer:expr); ($($v:expr),*); ()) => { ($($v,)*) };
}

// Implement `Unflatten` for tuples up to length 32.
macro_rules! impl_unflatten {
    (@impl_internal $t: ident, $($ts:ident,)*) => {
        impl<$t, $($ts),*> Unflatten for ($t, $($ts,)*) {
            type Unflattened = impl_unflatten!(@nest $t, $($ts,)*);
            #[inline]
            fn reflatten(v: Self::Unflattened) -> Self {
                unnest!((v); (); ($t, $($ts,)*))
            }
        }
    };

    (@nest) => {
        ()
    };

    (@nest $t: ident, $($ts:ident,)*) => {
        ($t, impl_unflatten!(@nest $($ts,)*))
    };

    // Base case
    (($($t:ident,)+);) => {
        impl_unflatten!(@impl_internal $($t,)*);
    };

    // Produce the actual impl for the tuple represented by $t1, then move $t2 into the tuple and
    // recursively call impl_unflatten
    (($($t1:ident,)+); $t2:ident $(,)* $($t3:ident),*) => {
        impl_unflatten!(@impl_internal $($t1,)*);
        impl_unflatten!(($($t1),*, $t2,); $($t3),*);
    };

    // Entry point
    ($t1:ident, $($t:ident),+) => {
        impl_unflatten!(($t1,); $($t),*);
    };
}

impl_unflatten!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF, GG
);

/// Internal version of `ComponentProvider` that is implemented for nested tuples.
pub trait ComponentProviderRec<'a, T> {
    /// Get the components.
    fn fetch(&'a self) -> T;
}

/// Component provider for flat tuples.
pub trait ComponentProvider<'a, T: Unflatten> {
    /// Get the components.
    fn fetch(&'a self) -> T;
}

impl<'a, T, W> ComponentProvider<'a, T> for W
where
    T: Unflatten,
    W: WorldInterface<'a> + ComponentProviderRec<'a, T::Unflattened>,
{
    #[inline]
    fn fetch(&'a self) -> T {
        <T as Unflatten>::reflatten(<Self as ComponentProviderRec<'a, T::Unflattened>>::fetch(
            self,
        ))
    }
}

impl<'a, H, T, WD> ComponentProviderRec<'a, (ReadComponent<'a, H>, T)> for WD
where
    H: 'a + StorageSpec<'a>,
    H::Storage: ComponentStorage<'a, H>,
    WD: WorldInterface<'a> + ComponentProviderRec<'a, T> + GetComponent<'a, H>,
{
    #[inline]
    fn fetch(&'a self) -> (ReadComponent<'a, H>, T) {
        (
            ReadComponent {
                storage: <Self as GetComponent<'a, H>>::get(self),
            },
            <Self as ComponentProviderRec<T>>::fetch(self),
        )
    }
}

impl<'a, H, T, WD> ComponentProviderRec<'a, (WriteComponent<'a, H>, T)> for WD
where
    H: 'a + StorageSpec<'a>,
    H::Storage: ComponentStorage<'a, H>,
    WD: WorldInterface<'a> + ComponentProviderRec<'a, T> + GetComponent<'a, H>,
{
    #[inline]
    fn fetch(&'a self) -> (WriteComponent<'a, H>, T) {
        (
            WriteComponent {
                storage: <Self as GetComponent<'a, H>>::get_mut(self),
            },
            <Self as ComponentProviderRec<T>>::fetch(self),
        )
    }
}

impl<'a, WD> ComponentProviderRec<'a, ()> for WD {
    #[inline]
    fn fetch(&'a self) -> () {
        ()
    }
}

impl<'a, WD, T> ComponentProviderRec<'a, ReadComponent<'a, T>> for WD
where
    T: 'a + StorageSpec<'a>,
    WD: WorldInterface<'a> + GetComponent<'a, T>,
{
    fn fetch(&'a self) -> ReadComponent<T> {
        ReadComponent {
            storage: <Self as GetComponent<'a, T>>::get(self),
        }
    }
}

impl<'a, WD, T> ComponentProviderRec<'a, WriteComponent<'a, T>> for WD
where
    T: 'a + StorageSpec<'a>,
    WD: WorldInterface<'a> + GetComponent<'a, T>,
{
    fn fetch(&'a self) -> WriteComponent<T> {
        WriteComponent {
            storage: <Self as GetComponent<'a, T>>::get_mut(self),
        }
    }
}

/// Trait that systems must implement.
pub trait System<'a> {
    /// The components and resources this system needs to run.
    type Dependencies: Unflatten; // +IntoTypeList;
    /// Run the system.
    fn run(&'a mut self, dependencies: Self::Dependencies);
}
/// Output of `PureFunctionalSystem` for one component.
pub enum SystemOutput<T> {
    /// Ignore the component (neither update nor delete it).
    Ignore,
    /// Delete the component if it exists.
    Delete,
    /// Update the component with a new value.
    Update(T),
}

impl<T> Default for SystemOutput<T> {
    fn default() -> Self {
        SystemOutput::Ignore
    }
}

/// For systems that don't cause side effects or need to reason about entities or components
/// globally, it is highly recommended that you implement `PureFunctionalSystem`, which the
/// library will be able to automatically parallelize.
pub trait PureFunctionalSystem<I, O: SystemOutputTuple> {
    /// Process one input.
    fn process(&self, data: &I) -> <O as SystemOutputTuple>::OutputTuple;
}

/// Interface to the `World` struct generated via the `define_world!` macro.
pub trait WorldInterface<'a>
where
    Self: Sized,
{
    /// The type returned by `new_entity()`.
    type EntityBuilder: 'a;
    /// A type representing the union of every component type supported by the `World`.
    type ComponentSet;
    /// A `TypeList` containing all available types.
    type AvailableTypes;
    /// Create a new entity.
    fn new_entity(&'a mut self) -> Self::EntityBuilder;
    /// Consume an `EntityBuilder` and store its components. Under normal circumstances, this
    /// should only be called by `EntityBuilder::build()`.
    fn build_entity(&mut self, c: Self::ComponentSet) -> Entity;
    /// Delete an entity.
    fn delete_entity(&mut self, e: Entity);
    /// Run a system.
    fn run_system<'b, S, T /*, U, V*/>(&'a mut self, system: &'b mut S)
    where
        S: System<'b, Dependencies = T>,
        T: Unflatten,
        //U: typelist::TypeList,
        //Self::AvailableTypes: typelist::ConsumeMultiple<U, V>,
        Self: ComponentProviderRec<'a, T::Unflattened>,
    {
        system.run(<Self as ComponentProvider<'a, T>>::fetch(self));
    }
}

/// Trait implemented by `EntityBuilder` types.
pub trait BuildWith<T> {
    /// Set the component of type `T`.
    fn with(self, data: T) -> Self;
}

/// Get the `Resources` struct from a world generically.
pub trait ResourceProvider {
    /// The `Resources` struct type.
    type Resources;
    /// Get the resources struct.
    fn get_resources(&mut self) -> &Self::Resources;
}

/// Indicates that the implementor stores components of type `T`.
pub trait GetComponent<'a, T: 'a + StorageSpec<'a>> {
    /// Get the storage.
    fn get(&self) -> std::cell::Ref<T::Storage>;
    /// Get the storage mutably.
    fn get_mut(&self) -> std::cell::RefMut<T::Storage>;
}

mod private {
    pub trait Sealed {}
    impl Sealed for () {}
}

/// Trait for converting tuples into tuples of `SystemOutput`s.
pub trait SystemOutputTuple: private::Sealed {
    /// The output of the conversion.
    type OutputTuple;
}

// Recursive macro to implement SystemOutputTuple for tuples up to length 32
macro_rules! impl_output_tuple {
    (@impl_internal $($t:ident,)+) => {
        impl<$($t),*> SystemOutputTuple for ($($t,)*) {
            type OutputTuple = ($(SystemOutput<$t>),*);
        }
        impl<$($t),*> private::Sealed for ($($t,)*) {}
    };

    // Base case
    (($($t:ident,)+);) => {
        impl_output_tuple!(@impl_internal $($t,)*);
    };

    // Produce the actual impl for the tuple represented by $t1, then move $t2 into the tuple and
    // recursively call impl_can_provide
    (($($t1:ident,)+); $t2:ident $(,)* $($t3:ident),*) => {
        impl_output_tuple!(@impl_internal $($t1,)*);
        impl_output_tuple!(($($t1),*, $t2,); $($t3),*);
    };

    // Entry point
    ($t1:ident, $($t:ident),+) => {
        impl_output_tuple!(($t1,); $($t),*);
    };
}

impl_output_tuple!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF, GG
);
