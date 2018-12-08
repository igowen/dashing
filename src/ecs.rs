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

#![deny(missing_docs)]

/// `Entity` is an opaque identifier that can be used to look up associated components in a
/// `World`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Entity {
    id: usize,
}

trait BitSet {
    const SIZE: usize;
    fn get_bit(&self, i: usize) -> bool;
    fn set_bit(&mut self, i: usize);
    fn clear_bit(&mut self, i: usize);
}

macro_rules! bitset_impl {
    ($t:ty,$b:tt) => {
        impl BitSet for $t {
            const SIZE: usize = $b;
            #[inline]
            fn get_bit(&self, i: usize) -> bool {
                if i < Self::SIZE {
                    (self & (1 << i)) != 0
                } else {
                    false
                }
            }

            #[inline]
            fn set_bit(&mut self, i: usize) {
                if i < Self::SIZE {
                    *self |= 1 << i;
                }
            }

            #[inline]
            fn clear_bit(&mut self, i: usize) {
                if i < Self::SIZE {
                    *self &= !(1 << i);
                }
            }
        }
    };
}

bitset_impl!(u8, 8);
bitset_impl!(u16, 16);
bitset_impl!(u32, 32);
bitset_impl!(u64, 64);
bitset_impl!(u128, 128);

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

struct BlockIter<'a, T> {
    cur: usize,
    block: &'a Block<T>,
}

impl<'a, T> Block<T> {
    fn iter(&'a self) -> BlockIter<'a, T> {
        BlockIter {
            cur: 0,
            block: self,
        }
    }
}

impl<'a, T> Iterator for BlockIter<'a, T> {
    type Item = Option<&'a T>;
    fn next(&mut self) -> Option<Option<&'a T>> {
        if self.cur >= 32 {
            None
        } else {
            let i = self.cur;
            self.cur += 1;
            if self.block.bits.get_bit(i) {
                Some(Some(&self.block.data[i]))
            } else {
                Some(None)
            }
        }
    }
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
    _t: std::marker::PhantomData<T>,
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

/// Joinable data
pub trait Join<'a, T: 'a, W> {
    /// Iterator type.
    type Iter: Iterator<Item = Entity>;
    /// Iterate over the join.
    fn join(w: &'a W) -> Self::Iter;
}

/// Iterator for two joined `BlockStorage`s.
pub struct BlockJoinIter<'a, A: Default, B: Default> {
    a: &'a BlockStorage<A>,
    b: &'a BlockStorage<B>,
    curr: Entity,
}

impl<'a, A: Default, B: Default> Iterator for BlockJoinIter<'a, A, B> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        while self.curr.id < self.a.size()
            && (self.a.get(self.curr).is_none() || self.b.get(self.curr).is_none())
        {
            println!("skipping {:?}", self.curr);
            self.curr.id += 1;
        }
        if self.curr.id < self.a.size() {
            let i = self.curr;
            self.curr.id += 1;
            return Some(i);
        } else {
            return None;
        }
    }
}

impl<'a, A, B, W> Join<'a, (A, B), W> for (A, B)
where
    W: Get<'a, A, Storage = BlockStorage<A>> + Get<'a, B, Storage = BlockStorage<B>>,
    A: 'a + Default,
    B: 'a + Default,
{
    type Iter = BlockJoinIter<'a, A, B>;
    fn join(w: &'a W) -> Self::Iter {
        BlockJoinIter {
            a: <W as Get<'a, A>>::get(w),
            b: <W as Get<'a, B>>::get(w),
            curr: Default::default(),
        }
    }
}

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
    (@define_world_struct $($field:ident:$type:ty)*) => {
        #[derive(Default)]
        pub struct World {
            $(
                $field: $crate::BlockStorage<$type>,
            )*
            num_entities: usize,
        }
        impl<'a> $crate::WorldInterface<'a> for World {
            type EntityBuilder = EntityBuilder<'a>;
            type ComponentSet = ComponentSet;
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
            fn build(&mut self, components: Self::ComponentSet) {
                use $crate::ComponentStorage;
                let entity = Entity{id:self.num_entities};
                $(
                    self.$field.set(entity, components.$field);
                )*
                self.num_entities += 1;
            }
        }
    };
    (@define_builder_struct $($field:ident:$type:ty)*) => {
        #[derive(Default)]
        pub struct ComponentSet {
            $(
                $field: Option<$type>,
            )*
        }
        pub struct EntityBuilder<'a> {
            components: ComponentSet,
            world: &'a mut World,
        }
        impl<'a> EntityBuilder<'a> {
            pub fn build(self) {
                use $crate::WorldInterface;
                self.world.build(self.components);
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
    ($($field:ident:BlockStorage<$type:ty>),* $(,)*) => {
        define_world!{@define_world_struct $($field:$type)*}
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
pub trait WorldInterface<'a>
where
    Self: std::marker::Sized,
{
    /// The type returned by new_entity().
    type EntityBuilder: 'a;
    /// A type representing the union of every type supported by the `World`.
    type ComponentSet;
    /// Create a new entity.
    fn new_entity(&'a mut self) -> Self::EntityBuilder;
    /// Consume an `EntityBuilder` and store its components. Under normal circumstances, this
    /// should only be called by `EntityBuilder::build()`.
    fn build(&mut self, c: Self::ComponentSet);
}

/// `System`
pub trait System {
    /// Inputs to the system.
    type Input;
    /// Outputs of the system.
    type Output;
    /// Run the system.
    fn run(&mut self, data: Self::Input) -> Self::Output;
}

#[cfg(test)]
mod tests {
    #[test]
    fn join() {
        use crate::BuildWith;
        use crate::Entity;
        use crate::Join;
        use crate::WorldInterface;

        #[derive(Default)]
        pub struct Position {
            x: i32,
            y: i32,
        }

        #[derive(Default)]
        pub struct Junk {
            s: String,
        }

        define_world!(position: BlockStorage<Position>, junk: BlockStorage<Junk>);

        let mut w = World::default();
        w.new_entity()
            .with(Junk {
                s: String::from("Hi!"),
            })
            .with(Position { x: 25, y: -104 })
            .build();

        w.new_entity()
            .with(Junk {
                s: String::from("Hello!"),
            })
            .with(Position { x: 25, y: -104 })
            .build();
        w.new_entity().with(Position { x: 25, y: -104 }).build();

        w.new_entity()
            .with(Junk {
                s: String::from("Ooga Booga"),
            })
            .with(Position { x: 25, y: -104 })
            .build();

        w.new_entity()
            .with(Junk {
                s: String::from("Only junk"),
            })
            .build();

        let e: Vec<Entity> = <(Position, Junk)>::join(&w).collect();
        assert_eq!(e.len(), 3);
        assert_eq!(e[0], Entity { id: 0 });
        assert_eq!(e[1], Entity { id: 1 });
        assert_eq!(e[2], Entity { id: 3 });
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
}
