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

use crate::ecs::typelist::IntoTypeList;
use crate::ecs::*;

/// Trait that systems must implement.
pub trait System {
    /// The components and resources this system needs to run.
    type Dependencies: IntoTypeList;
    /// Run the system.
    fn run(&mut self, dependencies: Self::Dependencies);
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
    fn run_system<S, T, U, V>(&'a mut self, system: &mut S)
    where
        S: System<Dependencies = T>,
        T: typelist::IntoTypeList<Type = U>,
        U: typelist::TypeList,
        Self::AvailableTypes: typelist::ConsumeMultiple<U, V>;
}

/// Trait implemented by `EntityBuilder` types.
pub trait BuildWith<T> {
    /// Set the component of type `T`.
    fn with(self, data: T) -> Self;
}

/// Trait that all component storage types must implement.
pub trait ComponentStorage<'a, T: 'a> {
    /// Immutable iterator type.
    type Iter: Iterator<Item = Option<&'a T>>;
    /// Mutable iterator type.
    type IterMut: Iterator<Item = Option<&'a mut T>>;
    /// Get the component corresponding to the given entity, if it exists.
    fn get(&self, entity: Entity) -> Option<&T>;
    /// Set the component for the given entity.
    fn set(&mut self, entity: Entity, item: Option<T>);
    /// Reserve `n` slots without affecting the size of the storage. The default implementation is
    /// a no-op; only implement if it makes sense for your storage type.
    fn reserve(&mut self, _n: usize) {}
    /// Get the number of components currently stored.
    fn size(&self) -> usize;
    /// Iterate over the components in this storage.
    ///
    /// **This *must* output a value for every entity in the world.**
    fn iter(&'a self) -> Self::Iter;
    /// Mutably iterate over the components in this storage.
    ///
    /// **This *must* output a value for every entity in the world.**
    fn iter_mut(&'a mut self) -> Self::IterMut;
}

/// Get the `Resources` struct from a world generically.
pub trait ResourceProvider {
    /// The `Resources` struct type.
    type Resources;
    /// Get the resources struct.
    fn get_resources(&mut self) -> &Self::Resources;
}

/// Indicates that the implementor stores components of type `T`.
pub trait GetComponent<'a, T: 'a> {
    /// The backing storage type.
    type Storage: ComponentStorage<'a, T>;
    /// Get the storage.
    fn get(&self) -> std::cell::Ref<Self::Storage>;
    /// Get the storage mutably.
    fn get_mut(&self) -> std::cell::RefMut<Self::Storage>;
}

mod private {
    pub trait Sealed {}
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
