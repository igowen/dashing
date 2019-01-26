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

//! Type-level metaprogramming traits that allow the library to validate certain invariants at compile
//! time.
//!
//! This is broadly adapted from Lloyd Chan's excellent article [Gentle Intro to Type-level
//! Recursion in Rust][1]. However, since it's only used to enforce invariants, the resulting
//! heterogeneous list type doesn't contain any actual storage; this means that, at least in
//! theory, the compiler should be able to optimize it out completely.
//!
//! The meat of the module is found in the [`TypeList`](trait.TypeList.html),
//! [`Consume`](trait.Consume.html), and [`ConsumeMultiple`](trait.ConsumeMultiple.html) traits.
//! `TypeList`s are `cons`-style singly linked lists expressed in the type system;
//! the trait is implemented by [`TypeCons`](struct.TypeCons.html) and [`Nil`](enum.Nil.html).
//! As mentioned above, the types are effectively empty, and since `Nil` is an empty enum, cannot
//! even be constructed.
//!
//! The way this is used in the library is to indicate what types are available for Systems to
//! access within a World.
//!
//! # Examples
//!
//! `TypeList`s are constructed from lisp-style `cons` cells, terminating with `Nil`.
//! ```
//! # use dashing::ecs::typelist::*;
//! type AvailableTypes = TypeCons<f64, TypeCons<u32, TypeCons<String, Nil>>>;
//! ```
//!
//! The [`tlist!`](../macro.tlist.html) macro is provided to make writing these types easier and
//! less verbose.
//! ```
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! type AvailableTypes = tlist![f64, u32, String];
//! ```
//!
//! In this example, `do_stuff()` will take an argument of type `f64`, `u32`, or `String`. `I` is a
//! type parameter used by `Consume`; it should be left up to the type checker to infer. It's kind
//! of a bummer that this has to leak into the public interface, but that's the way it is.
//! ```
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! type AvailableTypes = tlist![f64, u32, String];
//! fn do_stuff<T, I>(t: T) where AvailableTypes: Consume<T, I> {
//!     // Do something with `t`
//! }
//! do_stuff(25.0f64);
//! do_stuff(42u32);
//! do_stuff(String::from("Hello!"));
//! ```
//!
//! Calling `do_struff()` with types that are not in `AvailableTypes` will fail to type check.
//! ```compile_fail
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! struct Whatever {
//!     x: f32,
//!     y: f32,
//! }
//! type AvailableTypes = tlist![f64, u32, String];
//! fn do_stuff<T, I>(t: T) where AvailableTypes: Consume<T, I> {
//!     // Do something with `t`
//! }
//! do_stuff(Whatever { x: 1.0, y: 3.0 });
//! ```
//!
//! Unfortunately, the error messages you get from the type checker failing are not particularly
//! helpful. For instance, in the example above, you will get something like the following:
//!
//! ```text
//! error[E0277]: the trait bound `main::dashing::ecs::typelist::Nil: main::dashing::ecs::typelist::Consume<main::Whatever, _>` is not satisfied
//!   --> src/lib.rs:75:1
//!    |
//! 14 | do_stuff(Whatever { 1.0, 3.0 });
//!    | ^^^^^^^^ the trait `main::dashing::ecs::typelist::Consume<main::Whatever, _>` is not implemented for `main::dashing::ecs::typelist::Nil`
//!    |
//! ```
//!
//! Not the greatest indicator of what the actual problem is.
//!
//! There is also a trait, `ConsumeMultiple`, that takes a `TypeList` as its type parameter (along
//! with a similar "Index" type that you should let the compiler infer, like with `Consume`).
//!
//! ```
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! type AvailableTypes = tlist![f64, u32, String];
//! fn do_stuff<T, I>() where AvailableTypes: ConsumeMultiple<T, I> {
//!     // Do something
//! }
//! do_stuff::<tlist![f64, u32], _>();
//! ```
//!
//! This similarly will fail to type check if not all of the types are available in the source list.
//! ```compile_fail
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! type AvailableTypes = tlist![f64, u32, String];
//! fn do_stuff<T, I>() where AvailableTypes: ConsumeMultiple<T, I> {
//!     // Do something
//! }
//! do_stuff::<tlist![f64, u32, &str], _>();
//! ```
//!
//! Importantly, `Consume<T, I>` removes *all* instances of `T` from the source list; this allows
//! us to write generic functions over `T`, `U` such that `T != U` (!).
//!
//! ```
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! fn do_stuff<T, U, I>() where tlist![T, U]: ConsumeMultiple<tlist![T, U], I> {
//!     // Do something
//! }
//! do_stuff::<u32, f64, _>();
//! ```
//!
//! ```compile_fail
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! fn do_stuff<T, U, I>() where tlist![T, U]: ConsumeMultiple<tlist![T, U], I> {
//!     // Do something
//! }
//!
//! // Using the same type for `T` and `U` causes a compilation error along the lines of the
//! // following:
//! //
//! // error[E0282]: type annotations needed
//! //  --> src/ecs.rs:147:1
//! //   |
//! // 8 | do_stuff::<u32, u32, _>();
//! //   | ^^^^^^^^^^^^^^^^^^^^^^^ cannot infer type for `IHEAD`
//! do_stuff::<u32, u32, _>();
//! ```
//!
//! There is also a trait called `IntoTypeList` that allows easy conversion from tuples (up to
//! length 32) to `TypeList`.
//! ```
//! # #[macro_use] extern crate dashing;
//! # use dashing::ecs::typelist::*;
//! type AvailableTypes = tlist![f64, u32, String];
//! fn do_stuff<T, U, I>()
//! where
//!     T: IntoTypeList<Type=U>,
//!     // For some reason we still need to put a trait bound on `U`, even though the associated
//!     // type is constrained in `IntoTypeList`
//!     U: TypeList,
//!     AvailableTypes: ConsumeMultiple<U, I>
//! {
//!     // Do something
//! }
//! do_stuff::<(String, f64), _, _>();
//! ```
//!
//!
//! [1]: https://beachape.com/blog/2017/03/12/gentle-intro-to-type-level-recursion-in-Rust-from-zero-to-frunk-hlist-sculpting/
use std::marker::PhantomData;

mod private {
    pub trait Sealed {}
}

/// The empty list.
pub enum Nil {}
impl private::Sealed for Nil {}

/// A cons cell
pub struct TypeCons<H, T> {
    _head: PhantomData<*const H>,
    _tail: PhantomData<*const T>,
}
impl<H, T> private::Sealed for TypeCons<H, T> {}

/// Trait implemented by `Nil` and `TypeCons`
pub trait TypeList: private::Sealed {}

impl TypeList for Nil {}
impl<H, T> TypeList for TypeCons<H, T> {}

/// Index struct for `Consume` that indicates `T` hasn't been found in the list yet.
pub struct NotFound<T>(PhantomData<*const T>);
/// Index for `Consume` that indicates a type has been found.
pub enum Found {}

/// Generically append `T` to the end of a `TypeList`.
pub trait Append<T>: private::Sealed
where
    T: TypeList,
{
    /// `Self` with `T` appended.
    type Output: TypeList;
}

impl<T> Append<T> for Nil
where
    T: TypeList,
{
    type Output = T;
}

impl<H, T, U> Append<U> for TypeCons<H, T>
where
    T: Append<U>,
    U: TypeList,
{
    type Output = TypeCons<H, <T as Append<U>>::Output>;
}

/// Removes all instances of `T`, leaving `Self::Remainder`. `INDEX` must be inferred.
///
/// See the [module-level documentation](index.html#examples) for examples.
pub trait Consume<T, INDEX>: private::Sealed {
    /// The `TypeList` with all instances of `T` removed.
    type Remainder: TypeList;
}

impl<HEAD, TAIL, T, TINDEX> Consume<T, NotFound<TINDEX>> for TypeCons<HEAD, TAIL>
where
    TAIL: Consume<T, TINDEX>,
{
    type Remainder = TypeCons<HEAD, <TAIL as Consume<T, TINDEX>>::Remainder>;
}

impl<HEAD, TAIL: TypeList> Consume<HEAD, Found> for TypeCons<HEAD, TAIL> {
    type Remainder = TAIL;
}

/// Remove multiple elements, leaving `Self::Remainder`. `INDICES` must be inferred.
///
/// See the [module-level documentation](index.html#examples) for examples.
pub trait ConsumeMultiple<TLIST, INDICES>: private::Sealed {
    /// The `TypeList` with all of the elements of `T` removed.
    type Remainder;
}

impl<BASE: private::Sealed> ConsumeMultiple<Nil, Nil> for BASE {
    type Remainder = BASE;
}

impl<THEAD, TTAIL, SHEAD, STAIL, IHEAD, ITAIL>
    ConsumeMultiple<TypeCons<THEAD, TTAIL>, TypeCons<IHEAD, ITAIL>> for TypeCons<SHEAD, STAIL>
where
    TTAIL: TypeList,
    TypeCons<SHEAD, STAIL>: Consume<THEAD, IHEAD>,
    <TypeCons<SHEAD, STAIL> as Consume<THEAD, IHEAD>>::Remainder: ConsumeMultiple<TTAIL, ITAIL>,
{
    type Remainder =
        <<TypeCons<SHEAD, STAIL> as Consume<THEAD, IHEAD>>::Remainder as ConsumeMultiple<
            TTAIL,
            ITAIL,
        >>::Remainder;
}

/// Easy conversion into `TypeList`.
pub trait IntoTypeList: private::Sealed {
    /// The `TypeList` that is equivalent to this type.
    type Type: TypeList;
}

// TypeLists are trivially convertible to TypeLists.
impl<T> IntoTypeList for T
where
    T: TypeList,
{
    type Type = Self;
}

/// Helper macro for writing `TypeList`s.
#[macro_export]
macro_rules! tlist {
    ($t:ty $(,)*) => { $crate::ecs::typelist::TypeCons<$t, $crate::ecs::typelist::Nil> };
    ($t:ty, $($ts:ty),+ $(,)*) => {
        $crate::ecs::typelist::TypeCons<$t, tlist!($($ts,)*)>
    };
}

// Recursive macro to implement IntoTypeList for tuples up to length 32
macro_rules! impl_into_type_list {
    // Helpers for building type lists of generic types. We can't use `tlist!` because type
    // parameters don't parse as `ty`.
    (@type_cons $t:ident) => { TypeCons<$t, Nil> };
    (@type_cons $t:ident $($ts:ident)+) => {
        TypeCons<$t, impl_into_type_list!(@type_cons $($ts)*)>
    };

    (@impl_internal $($t:ident,)+) => {
        impl<$($t),*> private::Sealed for ($($t,)*) {}
        impl<$($t),*> IntoTypeList for ($($t,)*) {
            type Type = impl_into_type_list!(@type_cons $($t)*);
        }
    };

    // Base case
    (($($t:ident,)+);) => {
        impl_into_type_list!(@impl_internal $($t,)*);
    };

    // Produce the actual impl for the tuple represented by $t1, then move $t2 into the tuple and
    // recursively call impl_into_type_list
    (($($t1:ident,)+); $t2:ident $(,)* $($t3:ident),*) => {
        impl_into_type_list!(@impl_internal $($t1,)*);
        impl_into_type_list!(($($t1),*, $t2,); $($t3),*);
    };

    // Entry point
    ($t1:ident, $($t:ident),+) => {
        impl_into_type_list!(($t1,); $($t),*);
    };
}

impl_into_type_list!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF, GG
);
