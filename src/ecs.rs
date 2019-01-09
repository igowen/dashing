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

use std::cell::RefCell;

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

// disable compilation of parallel.rs for now
// pub mod parallel;
pub mod traits;

mod bitset;

//use crate::bitset::*;
pub use crate::traits::*;

/// `Entity` is an opaque identifier that can be used to look up associated components in a
/// `World`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: usize,
    pub generation: usize,
}

/*
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

impl<'a, T: 'a> ComponentStorage<'a, T> for BlockStorage<T>
where
    T: Default,
{
    fn get(&self, entity: Entity) -> Option<&'a T> {
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
*/

/*
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
    T: 'a + Default,
{
    fn get(&self, entity: Entity) -> Option<&T> {
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
*/

pub struct DumbVecIter<'a, T: 'a>(std::slice::Iter<'a, Option<T>>);
impl<'a, T: 'a> Iterator for DumbVecIter<'a, T> {
    type Item = Option<&'a T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| v.as_ref())
    }
}

pub struct DumbVecIterMut<'a, T: 'a>(std::slice::IterMut<'a, Option<T>>);
impl<'a, T: 'a> Iterator for DumbVecIterMut<'a, T> {
    type Item = Option<&'a mut T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| v.as_mut())
    }
}

#[derive(Clone, Debug, Default)]
pub struct DumbVecStorage<T>(Vec<Option<T>>);

impl<'a, T> ComponentStorage<'a, T> for DumbVecStorage<T>
where
    T: 'a,
{
    type Iter = DumbVecIter<'a, T>;
    type IterMut = DumbVecIterMut<'a, T>;
    fn get(&self, entity: Entity) -> Option<&T> {
        if entity.id < self.0.len() {
            self.0[entity.id].as_ref()
        } else {
            None
        }
    }
    fn set(&mut self, entity: Entity, item: Option<T>) {
        if entity.id >= self.0.len() {
            let n = entity.id - self.0.len() + 1;
            self.0.reserve(n);
            for _ in 0..n {
                self.0.push(None);
            }
        }
        self.0[entity.id] = item;
    }
    fn reserve(&mut self, n: usize) {
        self.0.reserve(n);
    }
    fn size(&self) -> usize {
        self.0.len()
    }
    fn iter(&'a self) -> Self::Iter {
        DumbVecIter(self.0.iter())
    }
    fn iter_mut(&'a mut self) -> Self::IterMut {
        DumbVecIterMut(self.0.iter_mut())
    }
}

/*
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
*/

#[macro_export]
macro_rules! define_world {
    (@define_resource_struct $v:vis (
                             {$($component:ident : ($($component_storage:ident) :: +; $component_type:ty))*}
                             {$($resource:ident : $resource_type:ty)*})) => {
        #[derive(Clone, Debug, Default)]
        $v struct Resources {
            $(
                $component: RefCell<$($component_storage)::*<$component_type>>,
            )*

            $(
                $resource: RefCell<$resource_type>,
            )*
        }
    };

    (@define_world_struct $v:vis
                          ($($component:ident : $type:ty)*)) => {
        /// Encapsulation of a set of component and resource types. Also provides a means for
        /// constructing new entities.
        #[derive(Default)]
        $v struct World {
            resources: Resources,
            num_entities: usize,
            free_list: Vec<Entity>,
        }

        impl $crate::ResourceProvider for World {
            type Resources = Resources;
            fn get_resources(&mut self) -> &Self::Resources {
                &self.resources
            }
        }

        impl<'a> $crate::WorldInterface<'a> for World {
            type EntityBuilder = EntityBuilder<'a>;
            type ComponentSet = ComponentSet;
            type AvailableTypes = tlist!($($type,)*);

            fn new_entity(&'a mut self) -> Self::EntityBuilder {
                EntityBuilder {
                    components: ComponentSet{
                    $(
                        $component: None,
                    )*
                    },
                    world: self,
                }
            }

            fn build_entity(&mut self, components: Self::ComponentSet) -> Entity {
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
                    // Should never panic, since having a mutable reference to `self` implies that
                    // there are no extant immutable references.
                    self.resources.$component.borrow_mut().set(entity, components.$component);
                )*
                entity
            }

            fn delete_entity(&mut self, entity: Entity) {
                use $crate::ComponentStorage;
                if entity.id < self.num_entities {
                    $(
                        self.resources.$component.borrow_mut().set(entity, None);
                    )*
                    self.free_list.push(entity);
                }
            }

            fn run_system<S, T, U, V>(&'a mut self, _system: &mut S)
            where
                S: System<Dependencies = T>,
                T: typelist::IntoTypeList<Type = U>,
                U: typelist::TypeList,
                Self::AvailableTypes: typelist::ConsumeMultiple<U, V> {

                //system.run(self);
            }
        }
    };

    (@define_builder_struct $v:vis $($field:ident:$type:ty)*) => {
        #[derive(Default)]
        /// ComponentSet is roughly equivalent to a tuple containing Option<T> for all types the
        /// World stores.
        $v struct ComponentSet {
            $(
                $field: Option<$type>,
            )*
        }
        /// Builder pattern for creating new entities.
        $v struct EntityBuilder<'a> {
            components: ComponentSet,
            world: &'a mut World,
        }
        impl<'a> EntityBuilder<'a> {
            /// Finalize this entity and all of its components by storing them in the `World`.
            $v fn build(self) -> Entity {
                use $crate::WorldInterface;
                self.world.build_entity(self.components)
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

    // Entry point
    ($v:vis world {
        components {
            $($component:ident : $($component_storage:ident) :: + < $component_type:ty >),* $(,)*
        }
        resources {
            $($resource:ident : $resource_type:ty),* $(,)*
        }
    }) => {
        define_world!{@define_world_struct $v
                                           ($($component: $component_type)*)}
        define_world!{@define_builder_struct $v $($component:$component_type)*}
        $(
            define_world!{@impl_build_with $component $component_type}
        )*
        define_world!{@define_resource_struct $v (
                                              {$($component:($($component_storage)::*; $component_type))*}
                                              {$($resource : $resource_type)*})}
    };
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

define_world!(
    pub world {
        components {
            test1: DumbVecStorage<f64>,
        }
        resources {
            test2: String,
        }
    }
);

#[cfg(test)]
mod tests {
    use crate::bitset::*;
    use crate::typelist::*;
    use crate::*;
    #[test]
    fn can_provide() {
        define_world!(
            pub world {
                components {
                    test1: DumbVecStorage<f64>,
                }
                resources {
                    test2: String,
                }
            }
        );
        let mut w = World::default();
        struct Poop {}
        impl System for Poop {
            type Dependencies = (f64,);
            fn run(&mut self, dependencies: Self::Dependencies) {}
        }
        let mut poop = Poop {};
        w.run_system(&mut poop);
        //<World as CanProvide<(f64, u32, f64)>>::test();
    }
    /*
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
            World {
                position: crate::BlockStorage<Position>,
                junk: crate::BlockStorage<Junk>
            }
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
        w.delete_entity(entity_to_delete);

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
            World {
                position: crate::BlockStorage<A>,
                junk: crate::BlockStorage<B>
            }
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
    */
}
