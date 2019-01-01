use crate::*;

/// Indicates that the implementor stores components of type `T`.
pub trait Get<'a, T> {
    /// The backing storage type.
    type Storage: ComponentStorage<'a, T>;
    /// Get the storage.
    fn get(&'a self) -> &'a Self::Storage;
    fn get_mut(&'a mut self) -> &'a mut Self::Storage;
}

/// Trait for converting tuples into tuples of `SystemOutput`s.
pub trait SystemOutputTuple: private::Sealed {
    type OutputTuple;
}

// Recursive macro to implement SystemOutputTuple for tuples up to length 32
macro_rules! impl_output_tuple {
    (@impl_internal $($t:ident,)+) => {
        impl<$($t),*> SystemOutputTuple for ($($t,)*) {
            type OutputTuple = ($(SystemOutput<$t>),*);
        }
        impl<$($t),*> private::Sealed for ($($t,)*) {}
    };

    // Base case
    (($($t:ident,)+);) => {
        impl_output_tuple!(@impl_internal $($t,)*);
    };

    // Produce the actual impl for the tuple represented by $t1, then move $t2 into the tuple and
    // recursively call impl_can_provide
    (($($t1:ident,)+); $t2:ident $(,)* $($t3:ident),*) => {
        impl_output_tuple!(@impl_internal $($t1,)*);
        impl_output_tuple!(($($t1),*, $t2,); $($t3),*);
    };

    // Entry point
    ($t1:ident, $($t:ident),+) => {
        impl_output_tuple!(($t1,); $($t),*);
    };
}

impl_output_tuple!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF, GG
);

pub trait CanProvide<T> {}

// Recursive macro to implement CanProvide for tuples up to length 32
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

impl<'a, WD> CanProvide<()> for WD where WD: WorldInterface<'a> {}

impl_can_provide!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, BB, CC, DD,
    EE, FF, GG
);
