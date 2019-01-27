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

use std::cell::{Ref, RefMut, UnsafeCell};
use std::ops::{Deref, DerefMut};

/// Specifies how a component is stored.
///
/// This is automatically implemented for component types by `define_world!`; you shouldn't ever
/// need to implement it manually.
pub trait StorageSpec<'a> {
    /// The component type.
    type Component: 'a;
    /// The storage type for this Component.
    type Storage: ComponentStorage<'a, Component = Self::Component>;
}

/// Read-only view of a Component storage.
pub struct ReadComponent<'a, T: StorageSpec<'a>> {
    // TODO: This probably doesn't need to be crate public.
    pub(crate) storage: Ref<'a, T::Storage>,
}

/// Read/write view of a Component storage.
pub struct WriteComponent<'a, T: 'a + StorageSpec<'a>> {
    // TODO: This probably doesn't need to be crate public.
    pub(crate) storage: RefMut<'a, T::Storage>,
}

// ReadComponent is cloneable; WriteComponent is not.
impl<'a, T> Clone for ReadComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
{
    #[inline]
    fn clone(&self) -> Self {
        ReadComponent {
            storage: Ref::clone(&self.storage),
        }
    }
}

impl<'a, T> ReadComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
    T::Storage: ComponentStorage<'a>,
{
    /// Get a reference to the underlying `Storage`. This is an associated method because
    /// `ReadComponent` implements `Deref`.
    #[inline]
    pub fn get(v: &Self) -> &T::Storage {
        Deref::deref(&v.storage)
    }
}

impl<'a, T> Deref for ReadComponent<'a, T>
where
    T: 'a + StorageSpec<'a>,
    T::Storage: ComponentStorage<'a>,
{
    type Target = T::Storage;
    #[inline]
    fn deref(&self) -> &T::Storage {
        Deref::deref(&self.storage)
    }
}

impl<'a, T> WriteComponent<'a, T>
where
    T: StorageSpec<'a>,
    T::Storage: ComponentStorage<'a>,
{
    /// Get a reference to the underlying `Storage`. This is an associated method because
    /// `WriteComponent` implements `Deref`/`DerefMut`.
    #[inline]
    pub fn get_mut(v: &mut Self) -> &mut T::Storage {
        DerefMut::deref_mut(&mut v.storage)
    }
}

impl<'a, T> Deref for WriteComponent<'a, T>
where
    T: StorageSpec<'a>,
    T::Storage: ComponentStorage<'a>,
{
    type Target = T::Storage;
    #[inline]
    fn deref(&self) -> &T::Storage {
        Deref::deref(&self.storage)
    }
}

impl<'a, T> DerefMut for WriteComponent<'a, T>
where
    T: StorageSpec<'a>,
    T::Storage: ComponentStorage<'a>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut T::Storage {
        DerefMut::deref_mut(&mut self.storage)
    }
}

/// Trait that all component storage types must implement.
pub trait ComponentStorage<'a> {
    /// The individual type of the Component in this storage
    type Component: 'a;
    /// Immutable iterator type.
    type Iter: Iterator<Item = Option<&'a Self::Component>>;
    /// Get the component corresponding to the given entity, if it exists.
    fn get<'b>(&'b self, entity: Entity) -> Option<&'b Self::Component>;
    /// Get a raw pointer to the component corresponding to the given entity, if it exists. Must
    /// return `std::ptr::null()` if the component doesn't exist for the given entity.
    fn get_raw(&self, entity: Entity) -> *const Self::Component;
    /// Set the component for the given entity.
    fn set(&mut self, entity: Entity, item: Option<Self::Component>);
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

/// Trait that component storage may optionally implement if it supports in-place modification.
pub trait MutableComponentStorage<'a>: ComponentStorage<'a> {
    /// Get a mutable reference to the component corresponding to the given entity, if it exists.
    fn get_mut<'b>(&'b mut self, entity: Entity) -> Option<&'b mut Self::Component>;
    /// Get a mutable raw pointer to the component corresponding to the given entity, if it exists.
    /// Must return `std::ptr::null()` if the component doesn't exist for the given entity.
    fn get_raw_mut(&mut self, entity: Entity) -> *mut Self::Component;
    /// Mutable iterator type.
    type IterMut: Iterator<Item = Option<&'a mut <Self as ComponentStorage<'a>>::Component>>;
    /// Mutably iterate over the components in this storage.
    ///
    /// **This *must* output a value for every entity it knows about, in `id` order.**
    fn iter_mut(&'a mut self) -> Self::IterMut;
}

/// `ComponentStorage` that is just `Vec<Option<T>>`.
#[derive(Debug, Default)]
pub struct BasicVecStorage<T>(Vec<Option<UnsafeCell<T>>>);

impl<'a, T> ComponentStorage<'a> for BasicVecStorage<T>
where
    T: 'a,
{
    type Component = T;
    type Iter = std::iter::Map<
        std::slice::Iter<'a, Option<UnsafeCell<T>>>,
        fn(&'a Option<UnsafeCell<T>>) -> Option<&'a T>,
    >;
    #[inline]
    fn get<'b>(&'b self, entity: Entity) -> Option<&'b T> {
        if entity.id < self.0.len() {
            // This unsafe block should be sound, because the borrow of the returned reference is
            // tied to the borrow of `&self`.
            self.0[entity.id].as_ref().map(|v| unsafe { &*v.get() })
        } else {
            None
        }
    }
    #[inline]
    fn get_raw(&self, entity: Entity) -> *const T {
        self.0[entity.id]
            .as_ref()
            .map_or(std::ptr::null(), |v| v.get())
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
        self.0[entity.id] = item.map(|x| UnsafeCell::new(x));
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
        // This unsafe block should be sound, because the borrows of the returned references are
        // tied to the borrow of `&self`.
        self.0
            .iter()
            .map(|v| v.as_ref().map(|u| unsafe { &*(u.get()) }))
    }
}

impl<'a, T: 'a> MutableComponentStorage<'a> for BasicVecStorage<T> {
    type IterMut = std::iter::Map<
        std::slice::IterMut<'a, Option<UnsafeCell<T>>>,
        fn(&mut Option<UnsafeCell<T>>) -> Option<&'a mut T>,
    >;
    #[inline]
    fn iter_mut(&'a mut self) -> Self::IterMut {
        // This unsafe block should be sound, because the borrows of the returned references are
        // tied to the borrow of `&mut self`.
        self.0
            .iter_mut()
            .map(|v| v.as_ref().map(|u| unsafe { &mut *(u.get()) }))
    }
    #[inline]
    fn get_mut<'b>(&'b mut self, entity: Entity) -> Option<&'b mut T> {
        if entity.id < self.0.len() {
            // This unsafe block should be sound, because the borrow of the returned references is
            // tied to the borrow of `&mut self`.
            self.0[entity.id].as_ref().map(|v| unsafe { &mut *v.get() })
        } else {
            None
        }
    }
    #[inline]
    fn get_raw_mut(&mut self, entity: Entity) -> *mut T {
        if entity.id < self.0.len() {
            self.0[entity.id]
                .as_ref()
                .map_or(std::ptr::null_mut(), |v| unsafe { &mut *v.get() })
        } else {
            std::ptr::null_mut()
        }
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

impl<'a, T: 'a + Default> ComponentStorage<'a> for VoidStorage<T> {
    type Component = T;
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
    fn get_raw(&self, entity: Entity) -> *const T {
        if entity.id / 32 < self.storage.len()
            && self.storage[entity.id / 32].get_bit(entity.id % 32)
        {
            &self.instance as *const T
        } else {
            std::ptr::null()
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
