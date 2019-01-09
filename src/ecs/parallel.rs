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

use crate::*;
use std::marker::PhantomData;

/// Helper to build a set of parallel systems.
///
/// Usage is subject to the following restrictions:
/// - Any number of systems may read the same input component; however
/// - No two systems may write the same output component
/// - If a component is read and written in the same dispatch, the inputs will always see the
/// original data (i.e., component writes are never visible within the same dispatch).
///
/// ```ignore
/// # use ecstatic::*;
/// # use ecstatic::traits::*;
/// # use ecstatic::typelist::*;
/// define_world!(World { test: BlockStorage<f64>, });
/// struct ExampleSystem;
/// // `ExampleSystem` outputs `f64`.
/// impl System<(), (f64,)> for ExampleSystem {
///     fn run<'a, W: WorldInterface<'a> + CanProvide<()> + CanProvide<(f64,)>>(
///         &mut self,
///         world: &W,
///     ) {
///         // do nothing
///     }
/// }
///
/// let w = World::default();
/// let mut s = ExampleSystem {};
/// let d = w.new_dispatch().add(&mut s);
/// d.build();
/// ```
///
/// The restriction on multiple outputs of the same component is enforced statically, so if fail to
/// respect this restriction compilation will fail:
///
/// ```compile_fail
/// # use ecstatic::*;
/// # use ecstatic::traits::*;
/// # use ecstatic::typelist::*;
/// define_world!(World { test: BlockStorage<f64>, });
/// struct ExampleSystem;
/// // `ExampleSystem` outputs `f64`.
/// impl System<(), (f64,)> for ExampleSystem {
///     fn run<'a, W: WorldInterface<'a> + CanProvide<()> + CanProvide<(f64,)>>(
///         &mut self,
///         world: &W,
///     ) {
///         // do nothing
///     }
/// }
///
/// let w = World::default();
/// let mut s = ExampleSystem {};
/// let mut s2 = ExampleSystem {};
/// let d = w.new_dispatch().add(&mut s).add(&mut s2);
/// d.build();
/// ```
///
/// Compilation will also fail if you try to write a type that is not supported by the `World`:
/// ```compile_fail
/// # use ecstatic::*;
/// # use ecstatic::traits::*;
/// # use ecstatic::typelist::*;
/// define_world!(World { test: BlockStorage<String>, });
/// struct ExampleSystem;
/// // `ExampleSystem` outputs `f64`.
/// impl System<(), (f64,)> for ExampleSystem {
///     fn run<'a, W: WorldInterface<'a> + CanProvide<()> + CanProvide<(f64,)>>(
///         &mut self,
///         world: &W,
///     ) {
///         // do nothing
///     }
/// }
///
/// let w = World::default();
/// let mut s = ExampleSystem {};
/// let mut s2 = ExampleSystem {};
/// let d = w.new_dispatch().add(&mut s).add(&mut s2);
/// d.build();
/// ```
///
/// However, **the error messages are not particularly helpful**, so it is highly recommended that
/// you try these examples so you know what to look for.
pub struct DispatchBuilder<'a, 'b, WD, OutputTypes>
where
    WD: WorldInterface<'b>,
{
    world: &'a WD,
    systems: Vec<Box<dyn SystemBinding + 'a>>,
    _used: PhantomData<OutputTypes>,
    _b: PhantomData<&'b u8>,
}

impl<'a, 'b, WD, OutputTypes> DispatchBuilder<'a, 'b, WD, OutputTypes>
where
    WD: WorldInterface<'b>,
    'b: 'a,
{
    /// Add a system to this dispatch.
    pub fn add<S, I, O>(
        mut self,
        system: &'a mut S,
    ) -> DispatchBuilder<'a, 'b, WD, <OutputTypes as Append<<O as IntoTypeList>::Type>>::Output>
    where
        S: System<I, O>,
        O: 'a + IntoTypeList, // + Append<<O as typelist::IntoTypeList>::Type>,
        I: 'a,
        WD: CanProvide<I> + CanProvide<O>,
        OutputTypes: Append<<O as typelist::IntoTypeList>::Type>,
    {
        let binding = BoundSystem {
            world: self.world,
            system: system,
            _i: PhantomData,
            _o: PhantomData,
            _b: PhantomData,
        };

        self.systems.push(Box::new(binding));

        DispatchBuilder {
            world: self.world,
            systems: self.systems,
            _used: PhantomData,
            _b: PhantomData,
        }
    }

    /// Finalize this dispatch.
    pub fn build<Index>(self)
    where
        <WD as WorldInterface<'b>>::AvailableTypes: ConsumeMultiple<OutputTypes, Index>,
    {
        for mut system in self.systems {
            system.run();
        }
    }
}

struct BoundSystem<'a, 'b, T, I, O, W>
where
    T: System<I, O>,
    W: WorldInterface<'b> + CanProvide<I> + CanProvide<O>,
{
    world: &'a W,
    system: &'a mut T,
    _i: PhantomData<I>,
    _o: PhantomData<O>,
    _b: PhantomData<&'b u8>,
}

trait SystemBinding {
    fn run(&mut self);
}

impl<'a, 'b, T, I, O, W> SystemBinding for BoundSystem<'a, 'b, T, I, O, W>
where
    T: System<I, O>,
    W: WorldInterface<'b> + CanProvide<I> + CanProvide<O>,
{
    fn run(&mut self) {
        self.system.run(self.world);
    }
}
