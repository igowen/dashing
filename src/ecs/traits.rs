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

use crate::typelist::IntoTypeList;
use crate::*;

/// Trait that systems must implement.
pub trait System {
    type Dependencies: IntoTypeList;
    /// Run the system.
    fn run(&mut self, dependencies: Self::Dependencies);
}

/// For systems that don't cause side effects or need to reason about entities or components
/// globally, it is highly recommended that you implement `PureFunctionalSystem`, which the
/// library is able to automatically parallelize.
pub trait PureFunctionalSystem<I, O: SystemOutputTuple> {
    /// Process one input.
    fn process(&self, data: &I) -> <O as SystemOutputTuple>::OutputTuple;
}

/// Trait implemented by the type output by the `define_world!` macro.
pub trait WorldInterface<'a>
where
    Self: Sized,
{
    /// The type returned by new_entity().
    type EntityBuilder: 'a;
    /// A type representing the union of every type supported by the `World`.
    type ComponentSet;
    /// A TypeList containing all available types.
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
    //fn run_pfs<I, O: SystemOutputTuple, S: PureFunctionalSystem<I, O>
    /*
    /// Prepare a system dispatch.
    fn new_dispatch(&'a self) -> DispatchBuilder<Self, Nil> {
        DispatchBuilder {
            world: self,
            systems: Default::default(),
            _used: PhantomData,
            _b: PhantomData,
        }
    }
    */
}

/// Trait implemented by `EntityBuilder` types.
pub trait BuildWith<T> {
    /// Set the component of type `T`.
    fn with(self, data: T) -> Self;
}

/// Trait that all component storage types must implement.
pub trait ComponentStorage<'a, T: 'a> {
    type Iter: Iterator<Item = Option<&'a T>>;
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
    fn iter(&'a self) -> Self::Iter;
    fn iter_mut(&'a mut self) -> Self::IterMut;
}

pub trait ResourceProvider {
    type Resources;
    fn get_resources(&mut self) -> &Self::Resources;
}

/// Indicates that the implementor stores components of type `T`.
pub trait GetComponent<'a, T: 'a> {
    /// The backing storage type.
    type Storage: ComponentStorage<'a, T>;
    /// Get the storage.
    fn get(&self) -> std::cell::Ref<Self::Storage>;
    fn get_mut(&self) -> std::cell::RefMut<Self::Storage>;
}

mod private {
    pub trait Sealed {}
}

/// Trait for converting tuples into tuples of `SystemOutput`s.
pub trait SystemOutputTuple: private::Sealed {
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

/*
pub trait CanProvide<'a, T> {}

// Recursive macro to implement CanProvide for tuples up to length 32
macro_rules! impl_can_provide {
    (@impl_internal $($t:ident,)+) => {
        impl<'a, WD, $($t),*> CanProvide<'a, ($($t,)*)> for WD
        where WD: $(Get<$t> +)* WorldInterface<'a> {}
    };

    // Base case
    (($($t:ident,)+);) => {
        impl_can_provide!(@impl_internal $($t,)*);
    };

    // Produce the actual impl for the tuple represented by $t1, then move $t2 into the tuple and
    // recursively call impl_can_provide
    (($($t1:ident,)+); $t2:ident $(,)* $($t3:ident),*) => {
        impl_can_provide!(@impl_internal $($t1,)*);
        impl_can_provide!(($($t1),*, $t2,); $($t3),*);
    };

    // Entry point
    ($t1:ident, $($t:ident),+) => {
        impl_can_provide!(($t1,); $($t),*);
    };
}

impl<'a, WD> CanProvide<'a, ()> for WD where WD: WorldInterface<'a> {}

impl_can_provide!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF, GG
);
*/
