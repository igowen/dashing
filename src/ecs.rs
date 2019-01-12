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

/// Type-level metaprogramming traits that allow the library to validate certain invariants at compile
/// time.
///
/// This is broadly adapted from Lloyd Chan's excellent article [Gentle Intro to Type-level
/// Recursion in Rust][1]. However, since it's only used to enforce invariants, the resulting
/// heterogeneous list type doesn't contain any actual storage; this means that, at least in
/// theory, the compiler should be able to optimize it out completely.
///
/// The meat of the module is found in the [`TypeList`](trait.TypeList.html),
/// [`Consume`](trait.Consume.html), and [`ConsumeMultiple`](trait.ConsumeMultiple.html) traits.
/// `TypeList`s are `cons`-style singly linked lists expressed in the type system;
/// the trait is implemented by [`TypeCons`](struct.TypeCons.html) and [`Nil`](enum.Nil.html).
/// As mentioned above, the types are effectively empty, and since `Nil` is an empty enum, cannot
/// even be constructed.
///
/// The way this is used in the library is to indicate what types are available for Systems to
/// access within a World.
///
/// # Examples
///
/// `TypeList`s are constructed from lisp-style `cons` cells, terminating with `Nil`.
/// ```
/// # use dashing::ecs::typelist::*;
/// type AvailableTypes = TypeCons<f64, TypeCons<u32, TypeCons<String, Nil>>>;
/// ```
///
/// The [`tlist!`](../macro.tlist.html) macro is provided to make writing these types easier and
/// less verbose.
/// ```
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// type AvailableTypes = tlist![f64, u32, String];
/// ```
///
/// In this example, `do_stuff()` will take an argument of type `f64`, `u32`, or `String`. `I` is a
/// type parameter used by `Consume`; it should be left up to the type checker to infer. It's kind
/// of a bummer that this has to leak into the public interface, but that's the way it is.
/// ```
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// type AvailableTypes = tlist![f64, u32, String];
/// fn do_stuff<T, I>(t: T) where AvailableTypes: Consume<T, I> {
///     // Do something with `t`
/// }
/// do_stuff(25.0f64);
/// do_stuff(42u32);
/// do_stuff(String::from("Hello!"));
/// ```
///
/// Calling `do_struff()` with types that are not in `AvailableTypes` will fail to type check.
/// ```compile_fail
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// struct Whatever {
///     x: f32,
///     y: f32,
/// }
/// type AvailableTypes = tlist![f64, u32, String];
/// fn do_stuff<T, I>(t: T) where AvailableTypes: Consume<T, I> {
///     // Do something with `t`
/// }
/// do_stuff(Whatever { x: 1.0, y: 3.0 });
/// ```
///
/// Unfortunately, the error messages you get from the type checker failing are not particularly
/// helpful. For instance, in the example above, you will get something like the following:
///
/// ```text
/// error[E0277]: the trait bound `main::dashing::ecs::typelist::Nil: main::dashing::ecs::typelist::Consume<main::Whatever, _>` is not satisfied
///   --> src/lib.rs:75:1
///    |
/// 14 | do_stuff(Whatever { 1.0, 3.0 });
///    | ^^^^^^^^ the trait `main::dashing::ecs::typelist::Consume<main::Whatever, _>` is not implemented for `main::dashing::ecs::typelist::Nil`
///    |
/// ```
///
/// Not the greatest indicator of what the actual problem is.
///
/// There is also a trait, `ConsumeMultiple`, that takes a `TypeList` as its type parameter (along
/// with a similar "Index" type that you should let the compiler infer, like with `Consume`).
///
/// ```
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// type AvailableTypes = tlist![f64, u32, String];
/// fn do_stuff<T, I>() where AvailableTypes: ConsumeMultiple<T, I> {
///     // Do something
/// }
/// do_stuff::<tlist![f64, u32], _>();
/// ```
///
/// This similarly will fail to type check if not all of the types are available in the source list.
/// ```compile_fail
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// type AvailableTypes = tlist![f64, u32, String];
/// fn do_stuff<T, I>() where AvailableTypes: ConsumeMultiple<T, I> {
///     // Do something
/// }
/// do_stuff::<tlist![f64, u32, &str], _>();
/// ```
///
/// Importantly, `Consume<T, I>` removes *all* instances of `T` from the source list; this allows
/// us to write generic functions over `T`, `U` such that `T != U` (!).
///
/// ```
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// fn do_stuff<T, U, I>() where tlist![T, U]: ConsumeMultiple<tlist![T, U], I> {
///     // Do something
/// }
/// do_stuff::<u32, f64, _>();
/// ```
///
/// ```compile_fail
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// fn do_stuff<T, U, I>() where tlist![T, U]: ConsumeMultiple<tlist![T, U], I> {
///     // Do something
/// }
///
/// // Using the same type for `T` and `U` causes a compilation error along the lines of the
/// // following:
/// //
/// // error[E0282]: type annotations needed
/// //  --> src/ecs.rs:147:1
/// //   |
/// // 8 | do_stuff::<u32, u32, _>();
/// //   | ^^^^^^^^^^^^^^^^^^^^^^^ cannot infer type for `IHEAD`
/// do_stuff::<u32, u32, _>();
/// ```
///
/// There is also a trait called `IntoTypeList` that allows easy conversion from tuples (up to
/// length 32) to `TypeList`.
/// ```
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::typelist::*;
/// type AvailableTypes = tlist![f64, u32, String];
/// fn do_stuff<T, U, I>()
/// where
///     T: IntoTypeList<Type=U>,
///     // For some reason we still need to put a trait bound on `U`, even though the associated
///     // type is constrained in `IntoTypeList`
///     U: TypeList,
///     AvailableTypes: ConsumeMultiple<U, I>
/// {
///     // Do something
/// }
/// do_stuff::<(String, f64), _, _>();
/// ```
///
///
/// [1]: https://beachape.com/blog/2017/03/12/gentle-intro-to-type-level-recursion-in-Rust-from-zero-to-frunk-hlist-sculpting/
#[macro_use]
pub mod typelist;

/// Traits used in the ECS interface(s).
pub mod traits;

mod bitset;

pub use crate::ecs::traits::*;

/// `Entity` is an opaque identifier that can be used to look up associated components in a
/// `World`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Entity {
    /// The id of this entity within the world.
    pub id: usize,
    /// The generation of this entity.
    pub generation: usize,
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

/// Defines the set of data structures necessary for using dashing's ECS architecture.
///
/// Generates the following structs:
/// - `Resources`
///   - All of the components and resources
/// - `World`
///   - Wraps `Resources` and contains entity metadata
/// - `EntityBuilder`
///   - Helper for `World::new_entity()`
/// - `ComponentSet`
///   - Used by `EntityBuilder`. Basically just all of the components wrapped in an `Option`.
///
/// # Example
/// ```
/// # #[macro_use] extern crate dashing;
/// # use dashing::ecs::*;
/// #[derive(Default, Debug)]
/// struct Data {
///     info: String,
/// }
///
/// define_world!(
///     // You can apply trait derivations to the output structs. Whatever is specified here will
///     // apply to both the `World` struct and the `Resources` struct.
///     #[derive(Default, Debug)]
///     // The visibility specifier is optional. It applies to all of the types defined by the
///     // macro.
///     pub world {
///         // Components must all go in collections that implement `ComponentStorage`. They are
///         // addressed by type, so you can only have one field per type.
///         components {
///             strings: BasicVecStorage<String>,
///         }
///         // Resources are just stored bare, but the same restriction on unique fields per type
///         // applies (but only within resources -- you can have a resource of the same type as a
///         // component).
///         resources {
///             data: Data,
///         }
///     }
/// );
/// ```
#[macro_export(local_inner_macros)]
macro_rules! define_world {
    ($(#[$meta:meta])*
     $v:vis world {
        components {
            $($component:ident : $($component_storage:ident) :: + < $component_type:ty >),* $(,)*
        }
        resources {
            $($resource:ident : $resource_type:ty),* $(,)*
        }
    }) => {
        __define_world_internal!{@define_world_struct $(#[$meta])* $v
                                           ($($component: $component_type)*)}
        __define_world_internal!{@define_builder_struct $v $($component:$component_type)*}
        $(
            __define_world_internal!{@impl_build_with $component $component_type}
        )*
        __define_world_internal!{@define_resource_struct $(#[$meta])* $v (
                                              {$($component:($($component_storage)::*; $component_type))*}
                                              {$($resource : $resource_type)*})}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __define_world_internal {
    (@define_resource_struct $(#[$meta:meta])* $v:vis (
                             {$($component:ident : ($($component_storage:ident) :: +; $component_type:ty))*}
                             {$($resource:ident : $resource_type:ty)*})) => {
        $(#[$meta])*
        $v struct Resources {
            $(
                $component: std::cell::RefCell<$($component_storage)::*<$component_type>>,
            )*

            $(
                $resource: std::cell::RefCell<$resource_type>,
            )*
        }
    };

    (@define_world_struct $(#[$meta:meta])* $v:vis
                          ($($component:ident : $type:ty)*)) => {
        /// Encapsulation of a set of component and resource types. Also provides a means for
        /// constructing new entities.
        $(#[$meta])*
        $v struct World {
            resources: Resources,
            num_entities: usize,
            free_list: Vec<Entity>,
        }

        impl $crate::ecs::ResourceProvider for World {
            type Resources = Resources;
            fn get_resources(&mut self) -> &Self::Resources {
                &self.resources
            }
        }

        impl<'a> $crate::ecs::WorldInterface<'a> for World {
            type EntityBuilder = EntityBuilder<'a>;
            type ComponentSet = ComponentSet;
            type AvailableTypes = tlist!($($type),*);

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
                use $crate::ecs::ComponentStorage;
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
                use $crate::ecs::ComponentStorage;
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
                use $crate::ecs::WorldInterface;
                self.world.build_entity(self.components)
            }
        }
    };

    (@impl_build_with $field:ident $type:ty) => {
        impl<'a> $crate::ecs::BuildWith<$type> for EntityBuilder<'a> {
            fn with(mut self, data: $type) -> Self {
                self.components.$field = Some(data);
                self
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::ecs::*;
    #[test]
    fn can_provide() {
        define_world!(
            #[derive(Default)]
            pub world {
                components {
                    test1: BasicVecStorage<f64>,
                }
                resources {
                    test2: String,
                }
            }
        );
        let mut w = World::default();
        struct TestSystem {}
        impl System for TestSystem {
            type Dependencies = (f64,);
            fn run(&mut self, dependencies: Self::Dependencies) {}
        }
        let mut system = TestSystem {};
        w.run_system(&mut system);
    }
}
