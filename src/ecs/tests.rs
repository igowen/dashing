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

            (&data,).for_each(|_, (d,)| {
                self.total += d.x;
            });

            (&data, &mut more_data).for_each(|_, (d, md)| {
                md.y *= d.x;
            });

            (&data, &void).for_each(|_, (d, _v)| {
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
