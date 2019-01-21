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

use crate::ecs::bitset::*;
use crate::ecs::*;

/// Trait that all component storage types must implement.
pub trait ComponentStorage<'a, T: 'a> {
    /// Immutable iterator type.
    type Iter: Iterator<Item = Option<&'a T>>;
    /// Get the component corresponding to the given entity, if it exists.
    fn get(&self, entity: Entity) -> Option<&T>;
    /// Set the component for the given entity.
    fn set(&mut self, entity: Entity, item: Option<T>);
    /// Reserve `n` additional slots without affecting the size of the storage. The default
    /// implementation is a no-op; only implement if it makes sense for your storage type.
    fn reserve(&mut self, _n: usize) {}
    /// Get the number of components currently stored.
    fn size(&self) -> usize;
    /// Iterate over the components in this storage.
    ///
    /// **This *must* output a value for every entity it knows about, in `id` order.**
    fn iter(&'a self) -> Self::Iter;
}

/// Trait that component storage may optionally implement.
pub trait MutableComponentStorage<'a, T: 'a>: ComponentStorage<'a, T> {
    /// Mutable iterator type.
    type IterMut: Iterator<Item = Option<&'a mut T>>;
    /// Mutably iterate over the components in this storage.
    ///
    /// **This *must* output a value for every entity it knows about, in `id` order.**
    fn iter_mut(&'a mut self) -> Self::IterMut;
}

/// `ComponentStorage` that is just `Vec<Option<T>>`.
#[derive(Clone, Debug, Default)]
pub struct BasicVecStorage<T>(Vec<Option<T>>);

impl<'a, T> ComponentStorage<'a, T> for BasicVecStorage<T>
where
    T: 'a,
{
    type Iter = BasicVecIter<'a, T>;
    #[inline]
    fn get(&self, entity: Entity) -> Option<&T> {
        if entity.id < self.0.len() {
            self.0[entity.id].as_ref()
        } else {
            None
        }
    }
    #[inline]
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
    #[inline]
    fn reserve(&mut self, n: usize) {
        self.0.reserve(n);
    }
    #[inline]
    fn size(&self) -> usize {
        self.0.len()
    }
    #[inline]
    fn iter(&'a self) -> Self::Iter {
        BasicVecIter(self.0.iter())
    }
}

impl<'a, T: 'a> MutableComponentStorage<'a, T> for BasicVecStorage<T> {
    type IterMut = BasicVecIterMut<'a, T>;
    #[inline]
    fn iter_mut(&'a mut self) -> Self::IterMut {
        BasicVecIterMut(self.0.iter_mut())
    }
}

/// Iterator type for `BasicVecStorage`.
pub struct BasicVecIter<'a, T: 'a>(std::slice::Iter<'a, Option<T>>);
impl<'a, T: 'a> Iterator for BasicVecIter<'a, T> {
    type Item = Option<&'a T>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| v.as_ref())
    }
}

/// Mutable iterator for `BasicVecStorage`.
pub struct BasicVecIterMut<'a, T: 'a>(std::slice::IterMut<'a, Option<T>>);
impl<'a, T: 'a> Iterator for BasicVecIterMut<'a, T> {
    type Item = Option<&'a mut T>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| v.as_mut())
    }
}

/// Storage for zero-size types. It's technically possible to use this for anything that implements
/// `Default`, but you will always get the output of `default()` when you iterate over it. Also,
/// this storage does not implement `MutableComponentStorage`, since there would be no point in
/// iterating over it in a mutable fashion.
#[derive(Default)]
pub struct VoidStorage<T: Default> {
    storage: Vec<u32>,
    // Store an actual instance since we need to be able to return it by reference.
    instance: T,
}

impl<'a, T: 'a + Default> ComponentStorage<'a, T> for VoidStorage<T> {
    type Iter = VoidStorageIter<'a, T>;

    #[inline]
    fn get(&self, entity: Entity) -> Option<&T> {
        if entity.id / 32 < self.storage.len()
            && self.storage[entity.id / 32].get_bit(entity.id % 32)
        {
            Some(&self.instance)
        } else {
            None
        }
    }

    #[inline]
    fn set(&mut self, entity: Entity, item: Option<T>) {
        if entity.id / 32 >= self.storage.len() {
            let n = self.storage.len() - entity.id / 32 + 1;
            for _ in 0..n {
                self.storage.push(0);
            }
        }
        match item {
            Some(_) => {
                self.storage[entity.id / 32].set_bit(entity.id % 32);
            }
            None => {
                self.storage[entity.id / 32].clear_bit(entity.id % 32);
            }
        }
    }

    #[inline]
    fn reserve(&mut self, n: usize) {
        self.storage.reserve((n + 1) / 32);
    }

    #[inline]
    fn size(&self) -> usize {
        self.storage.len() * 32
    }

    #[inline]
    fn iter(&'a self) -> Self::Iter {
        VoidStorageIter::new(self.storage.iter(), &self.instance)
    }
}

/// Iterator for `VoidStorage<T>`.
pub struct VoidStorageIter<'a, T> {
    iter: std::slice::Iter<'a, u32>,
    cur_bits: u32,
    cur: usize,
    instance: &'a T,
}

impl<'a, T> VoidStorageIter<'a, T> {
    #[inline]
    fn new(mut iter: std::slice::Iter<'a, u32>, instance: &'a T) -> Self {
        match iter.next() {
            Some(v) => VoidStorageIter {
                iter: iter,
                cur_bits: *v,
                cur: 0,
                instance: instance,
            },
            None => VoidStorageIter {
                iter: iter,
                cur_bits: 0,
                cur: 31,
                instance: instance,
            },
        }
    }
}

impl<'a, T> Iterator for VoidStorageIter<'a, T> {
    type Item = Option<&'a T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == 31 {
            match self.iter.next() {
                Some(v) => {
                    self.cur_bits = *v;
                    self.cur = 1;
                    if self.cur_bits.get_bit(0) {
                        Some(Some(self.instance))
                    } else {
                        Some(None)
                    }
                }
                None => None,
            }
        } else {
            self.cur += 1;
            if self.cur_bits.get_bit(self.cur - 1) {
                Some(Some(self.instance))
            } else {
                Some(None)
            }
        }
    }
}
