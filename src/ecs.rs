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

//! Ecstatic is a library for implementing the Entity Component System architecture (ECS). It is
//! primarily designed for use in games, although there is nothing strictly game-specific in the
//! API.
//!
//! Design goals:
//! * Statically typed (no library-level runtime errors or stringly-typed functionality)
//! * Functional (as in programming) paradigm
//! * No unsafe code

#![recursion_limit = "72"]

use std::marker::PhantomData;

/// `typelist` contains some type-level metaprogramming that allows the library to validate certain
/// invariants at compile time.
///
/// This is broadly adapted from Lloyd Chan's excellent article [Gentle Intro to Type-level
/// Recursion in Rust][1]. However, since it's only used to enforce invariants, the resulting
/// heterogeneous list type doesn't contain any actual storage; this means that, at least in
/// theory, the compiler should be able to optimize it out completely.
///
/// [1]: https://beachape.com/blog/2017/03/12/gentle-intro-to-type-level-recursion-in-Rust-from-zero-to-frunk-hlist-sculpting/
#[macro_use]
pub mod typelist;
mod bitset;

use crate::typelist::*;
use crate::bitset::*;

/// `Entity` is an opaque identifier that can be used to look up associated components in a
/// `World`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Entity {
    id: usize,
    generation: usize,
}

/// Trait that all component storages must implement.
pub trait ComponentStorage<'a, T> {
    /// Get the component corresponding to the given entity, if it exists.
    fn get(&'a self, entity: Entity) -> Option<&'a T>;
    /// Set the component for the given entity.
    fn set(&mut self, entity: Entity, item: Option<T>);
    /// Reserve `n` slots without affecting the size of the storage. The default implementation is
    /// a no-op; only implement if it makes sense for your storage type.
    fn reserve(&mut self, _n: usize) {}
    /// Get the number of components currently stored.
    fn size(&self) -> usize;
}

struct Block<T> {
    data: [T; 32],
    bits: u32,
    size: usize,
}

/// `BlockStorage` stores its components in 32-item blocks.
#[derive(Default)]
pub struct BlockStorage<T>
where
    T: Default,
{
    blocks: Vec<Block<T>>,
}

impl<'a, T> ComponentStorage<'a, T> for BlockStorage<T>
where
    T: Default,
{
    fn get(&'a self, entity: Entity) -> Option<&'a T> {
        let index = entity.id;
        if self.blocks[index / 32].bits.get_bit(index) {
            Some(&self.blocks[index / 32].data[index % 32])
        } else {
            None
        }
    }

    fn set(&mut self, entity: Entity, item: Option<T>) {
        if entity.id == self.size() {
            if self.blocks.is_empty() || self.blocks[self.blocks.len() - 1].size == 32 {
                let mut block = Block {
                    data: Default::default(),
                    bits: 0,
                    size: 1,
                };
                if let Some(t) = item {
                    block.data[0] = t;
                    block.bits.set_bit(0);
                }
                self.blocks.push(block);
            } else {
                let last = self.blocks.len() - 1;
                let block = &mut self.blocks[last];
                let index = block.size;
                block.size += 1;
                if let Some(t) = item {
                    block.data[index] = t;
                    block.bits.set_bit(index);
                }
            }
        } else {
            if let Some(t) = item {
                self.blocks[entity.id / 32].data[entity.id % 32] = t;
                self.blocks[entity.id / 32].bits.set_bit(entity.id % 32);
            } else {
                self.blocks[entity.id / 32].bits.clear_bit(entity.id % 32);
            }
        }
    }
    fn reserve(&mut self, n: usize) {
        let mut remaining = n;
        let last = self.blocks.len() - 1;
        let last_block = &mut self.blocks[last];
        if n < 32 - last_block.size {
            last_block.size += n;
        } else {
            remaining -= 32 - last_block.size;
            last_block.size = 32;
            while remaining > 0 {
                let block = Block {
                    data: Default::default(),
                    bits: 0,
                    size: remaining.min(32),
                };
                remaining -= block.size;
                self.blocks.push(block);
            }
        }
    }
    fn size(&self) -> usize {
        if self.blocks.len() == 0 {
            0
        } else {
            (self.blocks.len() - 1) * 32 + self.blocks[self.blocks.len() - 1].size
        }
    }
}

/// Storage for zero-sized types. Marginally more compact than `BlockStorage`. It's technically
/// possible to instantiate this with non-ZSTs, but `get()` will always return the default
/// instance, so don't do that.
#[derive(Default)]
pub struct VoidStorage<T> {
    data: Vec<u32>,
    size: usize,
    instance: T,
    _t: PhantomData<T>,
}

impl<'a, T> ComponentStorage<'a, T> for VoidStorage<T>
where
    T: Default,
{
    fn get(&'a self, entity: Entity) -> Option<&'a T> {
        if entity.id < self.size && self.data[entity.id / 32].get_bit(entity.id % 32) {
            Some(&self.instance)
        } else {
            None
        }
    }
    fn set(&mut self, entity: Entity, item: Option<T>) {
        if entity.id == self.size {
            if self.size % 32 == 0 {
                self.data.push(0);
                self.size += 1;
            }
            if let Some(_) = item {
                self.data[self.size / 32].set_bit(self.size % 32);
            }
        }
    }
    fn size(&self) -> usize {
        self.size
    }
}

/// Iterator for two joined `BlockStorage`s.
pub struct BlockJoinIter<'a, A: Default, B: Default> {
    a: &'a BlockStorage<A>,
    b: &'a BlockStorage<B>,
    curr_block: usize,
    curr_iter: BitSetIter<u32>,
}

impl<'a, A: Default, B: Default> BlockJoinIter<'a, A, B> {
    fn new(a: &'a BlockStorage<A>, b: &'a BlockStorage<B>) -> Self {
        let iter = if a.blocks.len() > 0 {
            (a.blocks[0].bits & b.blocks[0].bits).iter()
        } else {
            0u32.iter()
        };

        BlockJoinIter {
            a: a,
            b: b,
            curr_block: 0,
            curr_iter: iter,
        }
    }
}

impl<'a, A: Default, B: Default> Iterator for BlockJoinIter<'a, A, B> {
    type Item = (&'a A, &'a B);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(i) = self.curr_iter.next() {
                return Some((
                    &self.a.blocks[self.curr_block].data[i],
                    &self.b.blocks[self.curr_block].data[i],
                ));
            } else {
                self.curr_block += 1;
                if self.curr_block < self.a.blocks.len() {
                    self.curr_iter = (self.a.blocks[self.curr_block].bits
                        & self.b.blocks[self.curr_block].bits)
                        .iter();
                } else {
                    return None;
                }
            }
        }
    }
}

/// Joinable data
pub trait Join<'a, T: 'a, W> {
    /// Iterator type.
    type Iter: Iterator<Item = T>;
    /// Iterate over the join.
    fn join(w: &'a W) -> Self::Iter;
}

impl<'a, A, B, W> Join<'a, (&'a A, &'a B), W> for (A, B)
where
    W: Get<'a, A, Storage = BlockStorage<A>> + Get<'a, B, Storage = BlockStorage<B>>,
    A: 'a + Default,
    B: 'a + Default,
{
    type Iter = BlockJoinIter<'a, A, B>;
    fn join(w: &'a W) -> Self::Iter {
        BlockJoinIter::new(<W as Get<'a, A>>::get(w), <W as Get<'a, B>>::get(w))
    }
}
/*
impl<'a, A, B, W> Join<'a, (&'a A, &'a B), W> for (A, B)
where
    W: Get<'a, A, Storage = BlockStorage<A>>,
    A: 'a + Default,
    B: 'a + Default,
{
}
*/

/// A trait that indicates that the implementor is able to store components of type `T`.
pub trait Get<'a, T> {
    /// The backing storage type.
    type Storage: ComponentStorage<'a, T>;
    /// Get the storage.
    fn get(&'a self) -> &'a Self::Storage;
}

/// Trait for `EntityBuilder` types.
pub trait BuildWith<T> {
    /// Set the component of type `T`.
    fn with(self, data: T) -> Self;
}
#[macro_export]
macro_rules! define_world {
    (@define_world_struct $($field:ident : ($($storage:ident) :: +; $type:ty))*) => {
        /// `World` encapsulates a set of component types and provides a means for constructing new
        /// entities.
        #[derive(Default)]
        pub struct World {
            $(
                $field: $($storage)::*<$type>,
            )*
            num_entities: usize,
            free_list: Vec<Entity>,
        }
        impl<'a> $crate::WorldInterface<'a> for World {
            type EntityBuilder = EntityBuilder<'a>;
            type ComponentSet = ComponentSet;
            type AvailableTypes = tlist!($($type,)*); //define_world!(@type_cons $($type)*);

            fn new_entity(&'a mut self) -> Self::EntityBuilder {
                EntityBuilder {
                    components: ComponentSet{
                    $(
                        $field: None,
                    )*
                    },
                    world: self,
                }
            }

            fn build(&mut self, components: Self::ComponentSet) -> Entity {
                use $crate::ComponentStorage;
                let mut entity;
                if let Some(e) = self.free_list.pop() {
                    entity = e;
                    entity.generation += 1;
                } else {
                    entity = Entity{
                        id:self.num_entities,
                        generation: 0,
                    };
                    self.num_entities += 1;
                }
                $(
                    self.$field.set(entity, components.$field);
                )*
                entity
            }

            fn delete(&mut self, entity: Entity) {
                use $crate::ComponentStorage;
                if entity.id < self.num_entities {
                    $(
                        self.$field.set(entity, None);
                    )*
                    self.free_list.push(entity);
                }
            }
        }
    };
    (@define_builder_struct $($field:ident:$type:ty)*) => {
        #[derive(Default)]
        /// ComponentSet is roughly equivalent to a tuple containing Option<T> for all types the
        /// World stores.
        pub struct ComponentSet {
            $(
                $field: Option<$type>,
            )*
        }
        /// Builder pattern for creating new entities.
        pub struct EntityBuilder<'a> {
            components: ComponentSet,
            world: &'a mut World,
        }
        impl<'a> EntityBuilder<'a> {
            /// Finalize this entity and all of its components by storing them in the `World`.
            pub fn build(self) -> Entity {
                use $crate::WorldInterface;
                self.world.build(self.components)
            }
        }
    };
    (@impl_build_with $field:ident $type:ty) => {
        impl<'a> $crate::BuildWith<$type> for EntityBuilder<'a> {
            fn with(mut self, data: $type) -> Self {
                self.components.$field = Some(data);
                self
            }
        }
    };
    (@impl_get $field:ident $type:ty) => {
        impl<'a> $crate::Get<'a, $type> for World {
            type Storage = $crate::BlockStorage<$type>;
            fn get(&'a self) -> &'a Self::Storage { &self.$field }
        }
    };
    ($($field:ident : $($storage:ident) :: + < $type:ty >),* $(,)*) => {
        define_world!{@define_world_struct $($field:($($storage)::*; $type))*}
        $(
            define_world!{@impl_get $field $type}
        )*
        define_world!{@define_builder_struct $($field:$type)*}
        $(
            define_world!{@impl_build_with $field $type}
        )*
    };
}

/// `World` is a container for a set of entities and components.
/// This is mostly here so users know what to expect from the output of the `define_world!` macro.
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
    fn build(&mut self, c: Self::ComponentSet) -> Entity;
    /// Delete an entity.
    fn delete(&mut self, e: Entity);
}

struct BoundSystem<'a, T, I, O, W>
where
    T: System<I, O>,
    W: for<'b> WorldInterface<'b> + CanProvide<I> + CanProvide<O>,
{
    world: &'a W,
    system: &'a mut T,
    _i: PhantomData<I>,
    _o: PhantomData<O>,
}

trait SystemBinding {
    fn run(&mut self);
}

impl<'a, T, I, O, W> SystemBinding for BoundSystem<'a, T, I, O, W>
where
    T: System<I, O>,
    W: for<'b> WorldInterface<'b> + CanProvide<I> + CanProvide<O>,
{
    fn run(&mut self) {
        self.system.run(self.world);
    }
}

/// Helper to build a set of parallel systems, subject to the following restrictions:
/// - Any number of systems may read the same input component; however
/// - No two systems may write the same output component
/// - If a component is read and written in the same dispatch, the inputs will always see the
/// original data (i.e., component writes are never visible within the same dispatch).
pub struct DispatchBuilder<'a, WD, OutputTypes>
where
    for<'b> WD: WorldInterface<'b>,
{
    //_world: PhantomData<WD>,
    world: &'a WD,
    systems: Vec<Box<dyn SystemBinding + 'a>>,
    _used: PhantomData<OutputTypes>,
}

impl<'a, WD, OutputTypes> DispatchBuilder<'a, WD, OutputTypes>
where
    for<'b> WD: WorldInterface<'b>,
{
    pub fn add<S, I, O>(
        mut self,
        system: &'a mut S,
    ) -> DispatchBuilder<'a, WD, <OutputTypes as Append<<O as IntoTypeList>::Type>>::Output>
    where
        S: System<I, O>,
        O: 'a + IntoTypeList + Append<<O as typelist::IntoTypeList>::Type>,
        I: 'a,
        WD: CanProvide<I> + CanProvide<O>,
        OutputTypes: Append<<O as typelist::IntoTypeList>::Type>,
    {
        let binding = BoundSystem {
            world: self.world,
            system: system,
            _i: Default::default(),
            _o: Default::default(),
        };

        self.systems.push(Box::new(binding));

        DispatchBuilder {
            world: self.world,
            systems: self.systems,
            _used: Default::default(),
        }
    }

    pub fn build<Index>(self)
    where
        for<'b> <WD as WorldInterface<'b>>::AvailableTypes: ConsumeMultiple<OutputTypes, Index>,
    {
        for mut system in self.systems {
            system.run();
        }
    }
}

pub trait CanProvide<T> {}

pub trait CanStore<T> {}

// Recursive macro to implement CanProvide for tuples up length 32
macro_rules! impl_can_provide {
    (@impl_internal $($t:ident,)+) => {
        impl<'a, WD, $($t),*> CanProvide<($($t,)*)> for WD where WD: $(Get<'a, $t> +)* WorldInterface<'a> {}
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

impl_can_provide!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF, GG
);

pub trait Reader<T> {}

pub struct ReaderIterator<T> {
    _t: PhantomData<T>,
}

impl<T> Iterator for ReaderIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        None
    }
}

pub trait Writer<T> {}

/// `System`
pub trait System<I, O> {
    /// Run the system.
    fn run<W: CanProvide<I> + CanProvide<O>>(&mut self, world: &W);
}

pub enum SystemOutput<T> {
    Ignore,
    Delete,
    Update(T),
}

impl<T> Default for SystemOutput<T> {
    fn default() -> Self {
        SystemOutput::Ignore
    }
}

pub trait WriteComponents<T>: Default {}

/// For systems that don't cause side effects or need to reason about entities or components
/// globally, it is highly recommended that you implement `PureFunctionalSystem`, which the
/// library is able to automatically parallelize.
pub trait PureFunctionalSystem<I, O> {
    /// Process one input.
    fn process<T: WriteComponents<O>>(&self, data: &I) -> T;
}

#[cfg(test)]
mod tests {
    use crate::BlockStorage;
    use crate::BuildWith;
    use crate::Entity;
    use crate::Join;
    use crate::WorldInterface;
    #[test]
    fn can_provide() {
        define_world!(
            test1: BlockStorage<f64>,
            test2: BlockStorage<&'static str>,
            test3: BlockStorage<u32>,
        );
        let _w = World::default();
        //<World as CanProvide<(f64, u32, f64)>>::test();
    }
    #[test]
    fn join_basic() {
        #[derive(Default, Debug, Eq, PartialEq)]
        pub struct Position {
            x: i32,
            y: i32,
        }

        #[derive(Default, Debug, Eq, PartialEq)]
        pub struct Junk {
            s: String,
        }

        define_world!(
            position: crate::BlockStorage<Position>,
            junk: crate::BlockStorage<Junk>
        );

        let mut w = World::default();
        w.new_entity()
            .with(Junk {
                s: String::from("Hi!"),
            })
            .with(Position { x: 25, y: -104 })
            .build();

        let entity_to_delete = w
            .new_entity()
            .with(Junk {
                s: String::from("Hello!"),
            })
            .with(Position { x: 40, y: 72 })
            .build();

        w.new_entity().with(Position { x: 723, y: -19458 }).build();

        w.new_entity()
            .with(Junk {
                s: String::from("¡Hola!"),
            })
            .with(Position { x: 492, y: 2894 })
            .build();

        w.new_entity()
            .with(Junk {
                s: String::from("Only junk"),
            })
            .build();

        // First round: join Position and Junk as entered
        let e1: Vec<(&Position, &Junk)> = <(Position, Junk)>::join(&w).collect();
        assert_eq!(e1.len(), 3);
        assert_eq!(e1[0].0, &Position { x: 25, y: -104 });
        assert_eq!(e1[0].1.s, "Hi!");
        assert_eq!(e1[1].0, &Position { x: 40, y: 72 });
        assert_eq!(e1[1].1.s, "Hello!");
        assert_eq!(e1[2].0, &Position { x: 492, y: 2894 });
        assert_eq!(e1[2].1.s, "¡Hola!");

        // Delete the second entity.
        w.delete(entity_to_delete);

        // Second round: make sure the deleted entity doesn't appear in the join.
        let e2: Vec<(&Position, &Junk)> = <(Position, Junk)>::join(&w).collect();
        assert_eq!(e2.len(), 2);
        assert_eq!(e2[0].0, &Position { x: 25, y: -104 });
        assert_eq!(e2[0].1.s, "Hi!");
        assert_eq!(e2[1].0, &Position { x: 492, y: 2894 });
        assert_eq!(e2[1].1.s, "¡Hola!");

        // Create a new entity with `Position` and `Junk`.
        let new_entity = w
            .new_entity()
            .with(Junk {
                s: String::from("Reused!"),
            })
            .with(Position { x: 70, y: 140 })
            .build();

        // We should get the same entity id as the deleted one, but with a newer generation.
        assert_eq!(new_entity.id, entity_to_delete.id);
        assert!(new_entity.generation > entity_to_delete.generation);

        // Round 3: the new entity should appear in the middle of the join because we reused the
        // second slot.
        let e3: Vec<(&Position, &Junk)> = <(Position, Junk)>::join(&w).collect();
        assert_eq!(e3.len(), 3);
        assert_eq!(e3[0].0, &Position { x: 25, y: -104 });
        assert_eq!(e3[0].1.s, "Hi!");
        assert_eq!(e3[1].0, &Position { x: 70, y: 140 });
        assert_eq!(e3[1].1.s, "Reused!");
        assert_eq!(e3[2].0, &Position { x: 492, y: 2894 });
        assert_eq!(e3[2].1.s, "¡Hola!");
    }

    #[test]
    fn join_multiple_blocks() {
        #[derive(Default, Debug, Eq, PartialEq, Copy, Clone)]
        pub struct A {
            x: i32,
        }

        #[derive(Default, Debug, Eq, PartialEq, Clone)]
        pub struct B {
            s: String,
        }

        define_world!(
            position: crate::BlockStorage<A>,
            junk: crate::BlockStorage<B>
        );

        let mut w = World::default();
        let data = (0..15000_i32)
            .map(|i| {
                (
                    A { x: i },
                    B {
                        s: format!("{}", i),
                    },
                )
            })
            .collect::<Vec<_>>();

        data.iter().for_each(|(a, b)| {
            w.new_entity().with(*a).with(b.clone()).build();
        });

        for ((a1, b1), (a2, b2)) in data.iter().zip(<(A, B)>::join(&w)) {
            assert_eq!(a1, a2);
            assert_eq!(b1, b2);
        }
    }

    #[test]
    fn bitset() {
        use crate::BitSet;
        let mut x: u32 = 0;
        for i in 0..32 {
            assert!(x.get_bit(i) == false);
        }
        x.set_bit(12);
        assert!(x.get_bit(12));
        for i in 0..32 {
            if i != 12 {
                assert!(x.get_bit(i) == false);
            }
        }
        x.clear_bit(12);
        for i in 0..32 {
            assert!(x.get_bit(i) == false);
        }
        x = 0xffffffff;
        for i in 0..32 {
            assert!(x.get_bit(i) == true);
        }
        x.clear_bit(14);
        assert!(x.get_bit(14) == false);
        for i in 0..32 {
            if i != 14 {
                assert!(x.get_bit(i) == true);
            }
        }
    }

    #[test]
    fn bitset_iter() {
        use crate::BitSet;
        let x: u16 = 0b1010011101101010;
        let idxs = x.iter().collect::<Vec<_>>();
        assert_eq!(idxs, vec![1, 3, 5, 6, 8, 9, 10, 13, 15]);
    }
}
