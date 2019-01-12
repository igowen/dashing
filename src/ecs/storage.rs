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
    /// **This *must* output a value for every entity it knows about, in `id` order.**
    fn iter(&'a self) -> Self::Iter;
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
    type IterMut = BasicVecIterMut<'a, T>;
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
        BasicVecIter(self.0.iter())
    }
    fn iter_mut(&'a mut self) -> Self::IterMut {
        BasicVecIterMut(self.0.iter_mut())
    }
}

/// Iterator type for `BasicVecStorage`.
pub struct BasicVecIter<'a, T: 'a>(std::slice::Iter<'a, Option<T>>);
impl<'a, T: 'a> Iterator for BasicVecIter<'a, T> {
    type Item = Option<&'a T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| v.as_ref())
    }
}

/// Mutable iterator for `BasicVecStorage`.
pub struct BasicVecIterMut<'a, T: 'a>(std::slice::IterMut<'a, Option<T>>);
impl<'a, T: 'a> Iterator for BasicVecIterMut<'a, T> {
    type Item = Option<&'a mut T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|v| v.as_mut())
    }
}
