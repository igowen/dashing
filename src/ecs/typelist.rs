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

#[macro_export]
macro_rules! tlist {
    ($t:ty $(,)*) => { $crate::ecs::typelist::TypeCons<$t, $crate::ecs::typelist::Nil> };
    ($t:ty, $($ts:ty),+ $(,)*) => {
        $crate::ecs::typelist::TypeCons<$t, tlist!($($ts,)*)>
    };
}

// Recursive macro to implement IntoTypeList for tuples up length 32
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
