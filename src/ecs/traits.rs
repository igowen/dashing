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

use crate::ecs::*;

/// Trait that allows us to convert flat tuple types to nested tuple types (e.g.,
/// `(A, B, C)` → `(A, (B, (C, ())))`).
///
/// Also provides an associated function to convert a nested tuple **by value** to the equivalent
/// flat tuple.
///
/// This trait is provided for tuples up to length 32.
pub trait Nest: private::Sealed {
    /// Equivalent nested tuple type.
    type Nested;
    /// Convert a nested tuple to its flat representation.
    fn flatten(v: Self::Nested) -> Self;
    /// Convert `self` to its nested representation.
    fn nest(self) -> Self::Nested;
}

/// Inverse of `Nest`.
pub trait Flatten: private::Sealed {
    /// Equivalent flat tuple type.
    type Flattened;
    /// Convert `self` to its flat representation.
    fn flatten(self) -> Self::Flattened;
    /// Convert a flat tuple to its nested representation.
    fn nest(v: Self::Flattened) -> Self;
}

// helper macros for `impl_nested!`.
macro_rules! unnest {
    (($layer:expr); ($($v:expr),*); ($u:ident, $($us:ident,)*)) => {
        unnest!(($layer . 1); ($($v,)* $layer . 0); ($($us,)*))
    };
    (($layer:expr); ($($v:expr),*); ()) => { ($($v,)*) };
}

impl Nest for () {
    type Nested = ();
    #[inline]
    fn flatten(_v: ()) -> () {
        ()
    }
    #[inline]
    fn nest(self) -> () {
        ()
    }
}

macro_rules! nest {
    ($v:ident,) => { () };
    ($v:ident, $n:tt, $($ns:tt,)*) => {
        ($v.$n, nest!($v, $($ns,)*))
    }
}

// Implement `Nest` for tuples up to length 32.
macro_rules! impl_nested {
    (@impl_internal {($t:ident, $n:tt), $(($ts:ident, $ns:tt),)*}) => {
        impl<$t, $($ts),*> Nest for ($t, $($ts,)*) where ($($ts,)*): Nest {
            type Nested = impl_nested!(@nest_type $t, $($ts,)*);
            #[inline]
            fn flatten(v: Self::Nested) -> Self {
                unnest!((v); (); ($t, $($ts,)*))
            }
            #[inline]
            fn nest(self) -> Self::Nested {
                nest!(self, $n, $($ns,)*)
            }
        }

        impl<$t, $($ts),*> Flatten for impl_nested!(@nest_type $t, $($ts,)*) {
            type Flattened = ($t, $($ts,)*);
            #[inline]
            fn flatten(self) -> Self::Flattened {
                unnest!((self); (); ($t, $($ts,)*))
            }
            #[inline]
            fn nest(v: Self::Flattened) -> Self {
                nest!(v, $n, $($ns,)*)
            }
        }
    };

    (@nest_type) => {
        ()
    };

    (@nest_type $t:ident, $($ts:ident,)*) => {
        ($t, impl_nested!(@nest_type $($ts,)*))
    };

    // Base case
    (@internal {$(($t:ident,$n:tt),)+}; {}; {}) => {
        impl_nested!(@impl_internal {$(($t, $n),)*});
    };

    // Produce the actual impl for the tuple represented by $t1, then move $t2 into the tuple and
    // recursively call impl_nested
    (@internal {$(($t1:ident,$n1:tt),)+};
               {$t2:ident, $($t3:ident,)*};
               {$n2:tt, $($n3:tt,)*}) => {
        impl_nested!(@impl_internal {$(($t1, $n1),)*});
        impl_nested!(@internal {$(($t1, $n1),)* ($t2, $n2),};
                               {$($t3,)*};
                               {$($n3,)*});
    };

    // Entry point
    (($t:ident, $($ts:ident,)+); ($n:tt, $($ns:tt,)+)) => {
        impl_nested!(@internal {($t, $n),}; {$($ts,)*}; {$($ns,)*});
    };
}

impl_nested!(
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF,);
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,)
);

/// Internal version of `ComponentProvider` that is implemented for nested tuples.
pub trait ComponentProviderRec<'a, T> {
    /// Get the components.
    fn fetch(&'a self) -> T;
}

/// Component provider for flat tuples.
pub trait ComponentProvider<'a, T: Nest> {
    /// Get the components.
    fn fetch(&'a self) -> T;
}

impl<'a, T, W> ComponentProvider<'a, T> for W
where
    T: Nest,
    W: WorldInterface<'a> + ComponentProviderRec<'a, T::Nested>,
{
    #[inline]
    fn fetch(&'a self) -> T {
        <T as Nest>::flatten(<Self as ComponentProviderRec<'a, T::Nested>>::fetch(self))
    }
}

impl<'a, H, T, WD> ComponentProviderRec<'a, (ReadComponent<'a, H>, T)> for WD
where
    H: 'a + StorageSpec<'a>,
    H::Storage: ComponentStorage<'a>,
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

impl<'a, H, T, WD> ComponentProviderRec<'a, (ReadResource<'a, H>, T)> for WD
where
    H: 'a,
    WD: WorldInterface<'a> + ComponentProviderRec<'a, T> + GetResource<H>,
{
    #[inline]
    fn fetch(&'a self) -> (ReadResource<'a, H>, T) {
        (
            ReadResource {
                resource: <Self as GetResource<H>>::get(self),
            },
            <Self as ComponentProviderRec<T>>::fetch(self),
        )
    }
}

impl<'a, H, T, WD> ComponentProviderRec<'a, (WriteResource<'a, H>, T)> for WD
where
    H: 'a,
    WD: WorldInterface<'a> + ComponentProviderRec<'a, T> + GetResource<H>,
{
    #[inline]
    fn fetch(&'a self) -> (WriteResource<'a, H>, T) {
        (
            WriteResource {
                resource: <Self as GetResource<H>>::get_mut(self),
            },
            <Self as ComponentProviderRec<T>>::fetch(self),
        )
    }
}

impl<'a, H, T, WD> ComponentProviderRec<'a, (WriteComponent<'a, H>, T)> for WD
where
    H: 'a + StorageSpec<'a>,
    H::Storage: ComponentStorage<'a>,
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
    type Dependencies: Nest; // +IntoTypeList;
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
pub trait PureFunctionalSystem {
    /// Input types
    type Inputs;
    /// Output types.
    type Outputs: SystemOutputTuple;
    /// Process one input.
    fn process(&self, data: &Self::Inputs) -> <Self::Outputs as SystemOutputTuple>::OutputTuple;
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
        T: Nest,
        //U: typelist::TypeList,
        //Self::AvailableTypes: typelist::ConsumeMultiple<U, V>,
        Self: ComponentProviderRec<'a, T::Nested>,
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
pub trait GetComponent<'a, T: StorageSpec<'a>> {
    /// Get the storage.
    fn get(&self) -> std::cell::Ref<T::Storage>;
    /// Get the storage mutably.
    fn get_mut(&self) -> std::cell::RefMut<T::Storage>;
}

/// Indicates that the implementor stores a resource of type `T`.
pub trait GetResource<T> {
    /// Get the resource.
    fn get(&self) -> std::cell::Ref<T>;
    /// Get the resource mutably.
    fn get_mut(&self) -> std::cell::RefMut<T>;
    /// Set the resource.
    fn set(&self, t: T);
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
            #[allow(unused_parens)]
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
    EE, FF
);
