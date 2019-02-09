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

#[macro_use]
pub mod typelist;

/// Traits used in the ECS interface(s)
pub mod traits;

/// Component storage infrastructure
pub mod storage;

pub mod join;

mod bitset;

pub use crate::ecs::join::*;
pub use crate::ecs::storage::*;
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
///     pub(crate) world {
///         // Components must all go in collections that implement `ComponentStorage`. They are
///         // addressed by type, so you can only have one field per type.
///         components {
///             strings: BasicVecStorage<Data>,
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
        __define_world_internal!{@impl_storage_spec {$($component_type; $($component_storage)::*)*}}
        __define_world_internal!{@impl_get_component $({$component $component_type})*}
        __define_world_internal!{@impl_get_resource $({$resource $resource_type})*}
        __define_world_internal!{@define_world_struct
            $(#[$meta])* $v ($($component: $component_type)*)}
        __define_world_internal!{@define_builder_struct $v $($component:$component_type)*}
        $(
            __define_world_internal!{@impl_build_with $component $component_type}
        )*
        __define_world_internal!{@define_resource_struct $(#[$meta])* $v
            (
                {$($component:($($component_storage)::*; $component_type))*}
                {$($resource : $resource_type)*}
            )
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __define_world_internal {
    (@impl_storage_spec {$($component_type:ty; $($component_storage:ident)::+ )*}) => {
        $(
            impl<'a> $crate::ecs::StorageSpec<'a> for $component_type {
                type Storage = $($component_storage)::* <$component_type>;
                type Component = $component_type;
            }
        )*
    };

    (@impl_get_resource $({$resource:ident $resource_type:ty})*) => {
        $(
            impl GetResource<$resource_type> for World {
                fn get(&self) -> std::cell::Ref<$resource_type> {
                    self.resources.$resource.borrow()
                }
                fn get_mut(&self) -> std::cell::RefMut<$resource_type> {
                    self.resources.$resource.borrow_mut()
                }
                fn set(&self, t: $resource_type) {
                    self.resources.$resource.replace(t);
                }
            }
        )*
    };

    (@impl_get_component $({$component:ident $component_type:ty})*) => {
        $(
            impl<'a> GetComponent<'a, $component_type> for World {
                fn get(&self) -> std::cell::Ref<<$component_type as StorageSpec<'a>>::Storage> {
                    self.resources.$component.borrow()
                }
                fn get_mut(&self) -> std::cell::RefMut<<$component_type as StorageSpec<'a>>::Storage> {
                    self.resources.$component.borrow_mut()
                }
            }
        )*
    };

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

    #[derive(Debug, PartialEq)]
    pub struct Data {
        x: u32,
    }

    // `Default` impl that isn't the additive identity.
    impl Default for Data {
        fn default() -> Data {
            Data { x: 128 }
        }
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct MoreData {
        y: u32,
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct Void {}

    define_world!(
        #[derive(Default)]
        pub world {
            components {
                test1: BasicVecStorage<Data>,
                test2: BasicVecStorage<MoreData>,
                test3: VoidStorage<Void>,
            }
            resources {
                test_resource: String,
            }
        }
    );

    #[test]
    fn test_world() {
        let mut w = World::default();
        w.new_entity().with(Data { x: 1 }).build();
        w.new_entity().with(Data { x: 1 }).build();
        let md = w
            .new_entity()
            .with(Data { x: 2 })
            .with(MoreData { y: 10 })
            .build();
        w.new_entity().with(Data { x: 3 }).build();
        w.new_entity().with(Data { x: 5 }).build();
        w.new_entity().with(Data { x: 8 }).build();

        #[derive(Default)]
        struct TestSystem {
            total: u32,
            chosen: u32,
        }

        impl<'a> System<'a> for TestSystem {
            type Dependencies = (ReadComponent<'a, Data>, WriteComponent<'a, MoreData>);
            fn run(&'a mut self, (data, mut more_data): Self::Dependencies) {
                self.total = 0;
                self.chosen = 0;
                for item in data.iter() {
                    if let Some(d) = item {
                        self.total += d.x;
                    }
                }
                for item in more_data.iter_mut() {
                    if let Some(d) = item {
                        d.y *= 2;
                    }
                }
            }
        }
        let mut system = TestSystem::default();
        w.run_system(&mut system);
        assert_eq!(system.total, 20);
        assert_eq!(
            <World as GetComponent<'_, MoreData>>::get(&w).get(md),
            Some(&MoreData { y: 20 })
        );
    }

    #[test]
    fn test_join() {
        let mut w = World::default();
        w.new_entity().with(Data { x: 1 }).build();
        w.new_entity().with(Data { x: 1 }).build();
        let md = w
            .new_entity()
            .with(Data { x: 2 })
            .with(MoreData { y: 42 })
            .build();
        w.new_entity().with(Data { x: 3 }).build();
        w.new_entity().with(Data { x: 5 }).with(Void {}).build();
        w.new_entity().with(Data { x: 8 }).build();

        #[derive(Default)]
        struct TestSystem {
            total: u32,
            chosen: u32,
        }

        impl<'a> System<'a> for TestSystem {
            type Dependencies = (
                WriteComponent<'a, Data>,
                WriteComponent<'a, MoreData>,
                ReadComponent<'a, Void>,
            );
            fn run(&'a mut self, (data, mut more_data, void): Self::Dependencies) {
                self.total = 0;
                self.chosen = 0;

                (&data,).for_each(|(d,)| {
                    self.total += d.x;
                });

                (&data, &mut more_data).for_each(|(d, md)| {
                    md.y *= d.x;
                });

                (&data, &void).for_each(|(d, _v)| {
                    self.chosen = d.x;
                });
            }
        }

        let mut system = TestSystem::default();
        w.run_system(&mut system);

        assert_eq!(system.total, 20);
        assert_eq!(
            <World as GetComponent<'_, MoreData>>::get(&w).get(md),
            Some(&MoreData { y: 84 })
        );
        assert_eq!(system.chosen, 5);
    }
}
