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
//! This part of the library contains some unsafe code, which could set of some alarm bells.
//! However, this usage should be safe and free of undefined behavior.
//!
//! Consider the following:
//! ```ignore
//! type Dependencies = (ReadComponent<'a, A>,
//!                      WriteComponent<'a, B>,
//!                      WriteComponent<'a, C>);
//! let (a, b, c,): Dependencies = ...;
//! (&a, &b, &mut c,).for_each(|(va, vb, vc)| { ... });
//! ```
//!
//! Through ~trait magic~ this is implemented as:
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
//! Moving a safe reference into a closure in the manner described above requires that the reference
//! be borrowed for `'static`. Similarly, if we were to store the capture ourselves in a struct,
//! that reference would need to outlive the enclosing scope's lifetime, in the general case. The
//! compiler apparently cannot prove that the closure does *not* outlive the recursive chain of
//! `process()` calls, so this causes the borrow checker to report a conflict. Since we are not
//! storing the closure anywhere, we know we don't actually need the reference to live longer than
//! the call, so we can move the reference into the closure as a raw pointer to work around the
//! borrow checker.
//!
//! While technically unsafe, this doesn't violate any aliasing or borrow rules; the borrows have
//! been checked in the safe layer, and the unsafe layer doesn't create any aliases or references
//! that outlive those borrows.
//!
//! Furthermore, it is impossible for client code (via the closure passed to `for_each`) to violate
//! the soundness of this approach in safe Rust, since the references passed to the closure are
//! only borrowed for the duration of that call.  `Joinable` is also a sealed trait, so it is not
//! possible for client code to violate this soundness by implementing this trait and doing
//! something funky with the closure.

use crate::ecs::*;

mod private {
    pub trait Sealed {}
    use crate::ecs::{ReadComponent, StorageSpec, WriteComponent};
    impl<'a, 'b, H, T> Sealed for (&'a ReadComponent<'b, H>, T) where H: StorageSpec<'b> {}
    impl<'a, 'b, H, T> Sealed for (&'a WriteComponent<'b, H>, T) where H: StorageSpec<'b> {}
    impl<'a, 'b, H, T> Sealed for (&'a mut WriteComponent<'b, H>, T) where H: StorageSpec<'b> {}
    impl Sealed for () {}
}

/// Indicates that the type can be joined via the `Join` api.
pub trait Joinable: private::Sealed {
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
    H: StorageSpec<'b, Component = H> + 'b,
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
        F: FnMut(Entity, Self::Output);
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
        F: FnMut(Entity, Self::Output),
    {
        let mut storage = self.nest();
        for i in 0..storage.size() {
            let e = Entity {
                id: i,
                generation: 0,
            };
            storage.process(e, |v| f(e, v.flatten()));
        }
    }
}
