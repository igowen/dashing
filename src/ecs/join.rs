// Copyright 2019 Google LLC
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

//! Support for iterating over a joined view of different components.
//!
//! # Soundness
//! This part of the library contains some unsafe code. However, I believe this is free of UB.
//!
//! Consider the following:
//! ```ignore
//! type Dependencies = (ReadComponent<'a, A>,
//!                      WriteComponent<'a, B>,
//!                      WriteComponent<'a, C>);
//! let (a, b, c,): Dependencies = ...;
//! (&a, &b, &mut c,).for_each(|(va, vb, vc) { ... });
//! ```
//!
//! Through ~trait magic~ this is converted to:
//! ```ignore
//! for e in ... {
//!     (&a, (&b, (&mut c, ()))).process(e, |v| f(v.flatten()));
//! }
//! ```
//!
//! `process()` builds up the nested `(&A, (&B, (&mut C, ())))` tuple recursively. Since it
//! requires `&mut self`, we know that we have exclusive access to the tuple of `Read`/`Write`
//! specifiers, and therefore exclusive access to the underlying component storage. Additionally,
//! we know that we can't have multiple mutable references to the same storage in the nested list,
//! because that list is constructed entirely in safe Rust.
//!
//! This call to `process` then recursively becomes:
//!
//! ```ignore
//! let v = a.get_raw(e)
//! (&b, (&mut c, ())).process(e, move |tail| f((&*v, tail).flatten()));
//! ```
//!
//! And so on, until we reach the `()` list terminator.
//!
//! Really, the only reason that we need to use raw pointers at all is that moving a safe reference
//! into a closure in the manner described above requires that the reference be borrowed for the
//! lifetime of the closure. However, the compiler cannot prove that the closure does not outlive
//! the recursive chain of `process()` calls, so this causes the borrow checker to report a conflict
//! (since `&mut self isn't borrowed for long enough). To remedy this, we move the reference into
//! the closure as a raw pointer. While technically unsafe, this doesn't violate any aliasing or
//! borrow rules; the borrows have been enforced in the safe layer, and the unsafe layer doesn't
//! create any aliases or references that outlive those borrows.
//!
//! Furthermore, it is impossible for client code (via the closure passed to `for_each`) to violate
//! the soundness of this approach in safe Rust, since the references passed to the closure are
//! only borrowed for the duration of that call.
//!
//! # Examples
//! ```
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::*;
//!
//! #[derive(Debug, PartialEq)]
//! pub struct Data {
//!     x: u32,
//! }
//!
//! // `Default` impl that isn't the additive identity.
//! impl Default for Data {
//!     fn default() -> Data {
//!         Data { x: 128 }
//!     }
//! }
//!
//! #[derive(Debug, Default, PartialEq)]
//! pub struct MoreData {
//!     y: u32,
//! }
//!
//! define_world!(
//!     #[derive(Default)]
//!     pub world {
//!         components {
//!             test1: BasicVecStorage<Data>,
//!             test2: BasicVecStorage<MoreData>,
//!         }
//!         resources {}
//!     }
//! );
//!
//! let mut w = World::default();
//! w.new_entity().with(Data { x: 1 }).build();
//! w.new_entity().with(Data { x: 1 }).build();
//! let md = w
//!     .new_entity()
//!     .with(Data { x: 2 })
//!     .with(MoreData { y: 42 })
//!     .build();
//! w.new_entity().with(Data { x: 3 }).build();
//! w.new_entity().with(Data { x: 5 }).build();
//! w.new_entity().with(Data { x: 8 }).build();
//!
//! /// `TestSystem` adds up the values in every `Data` component (storing the result in `total`),
//! /// and multiplies every `MoreData` by the `Data` in the same component.
//! #[derive(Default)]
//! struct TestSystem {
//!     total: u32,
//! }
//!
//! impl<'a> System<'a> for TestSystem {
//!     type Dependencies = (
//!         ReadComponent<'a, Data>,
//!         WriteComponent<'a, MoreData>,
//!     );
//!     fn run(&'a mut self, (data, mut more_data): Self::Dependencies) {
//!         self.total = 0;
//!
//!         (&data,).for_each(|(d,)| {
//!             self.total += d.x;
//!         });
//!
//!         (&data, &mut more_data).for_each(|(d, md)| {
//!             md.y *= d.x;
//!         });
//!     }
//! }
//!
//! let mut system = TestSystem::default();
//! w.run_system(&mut system);
//!
//! assert_eq!(system.total, 20);
//! assert_eq!(
//!     <World as GetComponent<'_, MoreData>>::get(&w).get(md),
//!     Some(&MoreData { y: 84 })
//! );
//! ```
//!
//! Components accessed via `ReadComponent` cannot be iterated over mutably:
//!
//! ```compile_fail
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::*;
//!
//! #[derive(Debug, PartialEq)]
//! pub struct Data {
//!     x: u32,
//! }
//!
//! define_world!(
//!     pub world {
//!         components {
//!             test1: BasicVecStorage<Data>,
//!         }
//!         resources {}
//!     }
//! );
//!
//! #[derive(Default)]
//! struct TestSystem {}
//!
//! impl<'a> System<'a> for TestSystem {
//!     type Dependencies = (
//!         ReadComponent<'a, Data>,
//!     );
//!     fn run(&'a mut self, (data,): Self::Dependencies) {
//!         (&mut data,).for_each(|(d,)| {
//!             // do something
//!         });
//!     }
//! }
//! ```

use crate::ecs::*;

/// Indicates that the type can be joined via the `Join` api.
pub trait Joinable {
    /// The type returned by this Joinable.
    type Output;
    /// Call `f` on the `Output` for the given entity, if it exists.
    fn process<F>(&mut self, e: Entity, f: F)
    where
        F: FnOnce(Self::Output);
    /// HACK: return the number of entities in the underlying storage.
    fn size(&self) -> usize;
}

impl<'a, 'b, H, T> Joinable for (&'a ReadComponent<'b, H>, T)
where
    H: StorageSpec<'b, Component = H>,
    T: Joinable,
{
    type Output = (&'a H, T::Output);
    fn process<F>(&mut self, e: Entity, f: F)
    where
        F: FnOnce(Self::Output),
    {
        let v = self.0.get_raw(e);
        if !v.is_null() {
            self.1.process(e, move |tail| f((unsafe { &*v }, tail)))
        }
    }
    fn size(&self) -> usize {
        self.0.size()
    }
}

impl<'a, 'b, H, T> Joinable for (&'a WriteComponent<'b, H>, T)
where
    H: StorageSpec<'b, Component = H>,
    T: Joinable,
{
    type Output = (&'a H, T::Output);
    fn process<F>(&mut self, e: Entity, f: F)
    where
        F: FnOnce(Self::Output),
    {
        let v = self.0.get_raw(e);
        if !v.is_null() {
            self.1.process(e, move |tail| f((unsafe { &*v }, tail)))
        }
    }
    fn size(&self) -> usize {
        self.0.size()
    }
}

impl<'a, 'b, H, T> Joinable for (&'a mut WriteComponent<'b, H>, T)
where
    H: StorageSpec<'b, Component = H>,
    H::Storage: MutableComponentStorage<'b>,
    T: Joinable,
{
    type Output = (&'a mut H, T::Output);
    fn process<F>(&mut self, e: Entity, f: F)
    where
        F: FnOnce(Self::Output),
    {
        let v = self.0.get_raw_mut(e);
        if !v.is_null() {
            self.1.process(e, move |tail| f((unsafe { &mut *v }, tail)))
        }
    }
    fn size(&self) -> usize {
        self.0.size()
    }
}

impl Joinable for () {
    type Output = ();
    fn process<F>(&mut self, _e: Entity, f: F)
    where
        F: FnOnce(()),
    {
        f(())
    }
    fn size(&self) -> usize {
        0
    }
}

/// Trait for joining different component types together.
pub trait Join {
    /// Output type of the join.
    type Output;
    /// Call `f` on each entity that contains all of the components in `Output`.
    fn for_each<F>(self, f: F)
    where
        F: FnMut(Self::Output);
}

impl<T> Join for T
where
    T: Nest,
    T::Nested: Joinable,
    <T::Nested as Joinable>::Output: Flatten,
{
    type Output = <<T::Nested as Joinable>::Output as Flatten>::Flattened;
    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Output),
    {
        let mut storage = self.nest();
        for i in 0..storage.size() {
            storage.process(
                Entity {
                    id: i,
                    generation: 0,
                },
                |v| f(v.flatten()),
            );
        }
    }
}
