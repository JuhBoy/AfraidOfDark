use crate::engine::ecs::my_ecs::components::{ComponentBufferSparseSet, StorageMetaData};
use crate::engine::ecs::my_ecs::ecs::ECS;
use crate::engine::ecs::my_ecs::entities;
use atomic_refcell::{AtomicRef, AtomicRefMut};
use core::slice::Iter;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;

use super::entities::Entity;

pub enum SystemUpdate {
    Update,
    FixedUpdate,
    LateUpdate,
}
pub struct SystemParams<'a> {
    pub world: &'a mut ECS,
    pub system_name: &'static str,
}

// Queries =============================
//
pub trait TQuery: 'static {
    type ViewMut<'a>;
    type View<'a>: 'a;
    type Item<'a>: 'a;
    type ItemMut<'a>: 'a;
    type Iter<'a>: 'a;

    fn borrow_storage<'a>(world: &'a ECS) -> Self::View<'a>;
    fn borrow_storage_mut(world: &ECS) -> Self::View<'_>;
    fn entities<'a>(world: &'a ECS, views: &Self::View<'_>) -> &'a [Entity];
    fn iter_predicat(entity: Entity, world: &ECS, view: &Self::View<'_>) -> bool;
    fn get_dense<'a>(entity: Entity, world: &'a ECS, view: &Self::View<'_>) -> Self::Item<'a>;
    fn get_dense_mut<'a>(
        entity: Entity,
        world: &'a ECS,
        view: &Self::View<'_>,
    ) -> Self::ItemMut<'a>;
}

pub struct StorageView<'a, T> {
    store: StorageMetaData,
    _phantom: PhantomData<&'a [T]>,
}

pub struct StorageViewMut<'a, T> {
    store: AtomicRefMut<'a, ComponentBufferSparseSet>,
    _phantom: PhantomData<&'a mut [T]>,
}

impl<A, B> TQuery for (A, B)
where
    A: 'static,
    B: 'static,
{
    type ViewMut<'a> = (StorageView<'a, A>, StorageView<'a, B>);
    type View<'a> = (StorageView<'a, A>, StorageView<'a, B>);
    type Item<'a> = (&'a A, &'a B);
    type ItemMut<'a> = (&'a mut A, &'a mut B);
    type Iter<'a> = Iter<'a, Self::Item<'a>>;

    fn borrow_storage(world: &ECS) -> Self::View<'_> {
        // @todo! maybe find a way to reduce the TypeId to a unique i64/32 identifier
        // @todo! using const compilation strategy like Attributes or something ?
        let a_storage = world.component_storage.get_statage_metadata::<A>();
        let b_storage = world.component_storage.get_statage_metadata::<B>();

        (
            StorageView {
                store: a_storage,
                _phantom: PhantomData,
            },
            StorageView {
                store: b_storage,
                _phantom: PhantomData,
            },
        )
    }

    fn borrow_storage_mut(world: &ECS) -> Self::View<'_> {
        let a = world.component_storage.get_statage_metadata::<A>();
        let b = world.component_storage.get_statage_metadata::<B>();

        (
            StorageView {
                store: a,
                _phantom: PhantomData,
            },
            StorageView {
                store: b,
                _phantom: PhantomData,
            },
        )
    }

    fn entities<'a>(world: &'a ECS, views: &Self::View<'_>) -> &'a [Entity] {
        let view_a = &views.0;
        let view_b = &views.1;

        let a_len = world
            .component_storage
            .get_storage_by_id(view_a.store.index)
            .entity_to_component
            .len();
        let b_len = world
            .component_storage
            .get_storage_by_id(view_b.store.index)
            .entity_to_component
            .len();

        let id = if a_len < b_len {
            view_a.store.index
        } else {
            view_b.store.index
        };

        let store = world.component_storage.get_storage_by_id(id);
        let entities = store.entity_to_component.as_slice().as_ptr();

        unsafe {
            let etts: &'a [Entity] = std::slice::from_raw_parts(entities, a_len.min(b_len));
            etts
        }
    }

    fn iter_predicat(entity: Entity, world: &ECS, view: &Self::View<'_>) -> bool {
        let store = world
            .component_storage
            .get_storage_by_id(view.0.store.index);
        let store2 = world
            .component_storage
            .get_storage_by_id(view.1.store.index);

        store.has(entity) && store2.has(entity)
    }

    fn get_dense<'a>(entity: Entity, world: &'a ECS, view: &Self::View<'_>) -> Self::Item<'a> {
        let store = world
            .component_storage
            .get_storage_by_id(view.0.store.index);
        let store2 = world
            .component_storage
            .get_storage_by_id(view.1.store.index);

        let component_a: *const A = store.get::<A>(entity).unwrap();
        let component_b: *const B = store2.get::<B>(entity).unwrap();

        unsafe {
            let upa: &'a A = &*component_a;
            let upb: &'a B = &*component_b;

            (upa, upb)
        }
    }

    fn get_dense_mut<'a>(
        entity: Entity,
        world: &'a ECS,
        view: &Self::View<'_>,
    ) -> Self::ItemMut<'a> {
        let mut store = world
            .component_storage
            .get_storage_mut_by_id(view.0.store.index);
        let mut store2 = world
            .component_storage
            .get_storage_mut_by_id(view.1.store.index);

        let component_a = store.get_mut::<A>(entity).unwrap() as *mut A;
        let component_b = store2.get_mut::<B>(entity).unwrap() as *mut B;

        unsafe { (&mut *component_a, &mut *component_b) }
    }
}

///////////////////////////////////

pub struct Query<'a, A>
where
    A: TQuery,
{
    pub world: &'a ECS,
    _phantom: PhantomData<A>,
}

impl<'a, A> Query<'a, A>
where
    A: TQuery,
{
    pub fn new(world: &'a ECS) -> Self {
        Self {
            world,
            _phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> QueryRef<'a, A> {
        QueryRef::new(self.world)
    }

    pub fn iter_mut(&mut self) -> QueryMut<'a, A> {
        QueryMut::new(self.world)
    }

    pub fn iter_with_entity(&self) -> QueryRef<'a, A> {
        QueryRef::new(self.world)
    }
}

pub struct QueryRef<'iter, A>
where
    A: TQuery,
{
    pub world: &'iter ECS,
    pub entities: &'iter [Entity],
    pub view: A::View<'iter>,
    pub _phantom: PhantomData<A>,

    next: usize,
}
impl<'iter, A> QueryRef<'iter, A>
where
    A: TQuery,
{
    fn new(world: &'iter ECS) -> Self {
        let view = A::borrow_storage(world);
        let entities = A::entities(world, &view);
        Self {
            world,
            entities,
            view,
            _phantom: PhantomData::<A>,
            next: 0,
        }
    }
}
impl<'a, A> Iterator for QueryRef<'a, A>
where
    A: TQuery,
{
    type Item = A::Item<'a>;

    fn next(&mut self) -> Option<A::Item<'a>> {
        for i in self.next..self.entities.len() {
            let entity = self.entities[i];
            self.next = i + 1;

            if !A::iter_predicat(entity, self.world, &self.view) {
                continue;
            }

            let data = A::get_dense(entity, self.world, &self.view);

            return Some(data);
        }

        None
    }
}

pub struct QueryMut<'a, A>
where
    A: TQuery,
{
    world: &'a ECS,
    entities: &'a [Entity],
    view: A::View<'a>,
    _phantom: PhantomData<&'a A>,

    next: usize,
}
impl<'a, A> QueryMut<'a, A>
where
    A: TQuery,
{
    pub fn new(world: &'a ECS) -> Self {
        let view = A::borrow_storage_mut(world);
        let entities = A::entities(world, &view);

        Self {
            world,
            view,
            _phantom: PhantomData,
            entities,
            next: 0,
        }
    }
}
impl<'a, A> Iterator for QueryMut<'a, A>
where
    A: TQuery,
{
    type Item = A::ItemMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.next..self.entities.len() {
            let entity = self.entities[i];
            self.next = i + 1;

            if !A::iter_predicat(entity, self.world, &self.view) {
                continue;
            }

            let data = A::get_dense_mut(entity, self.world, &self.view);

            return Some(data);
        }

        None
    }
}

///////////////////////////////////
// SYSTEMS
////////

pub trait TSystem {
    fn run(&self, params: SystemParams) -> bool;
}
impl TSystem for System {
    fn run(&self, mut params: SystemParams) -> bool {
        (self.action)(&mut params);
        true
    }
}

pub struct System {
    pub name: &'static str,

    pub system_index: i32,
    pub update_type: SystemUpdate,
    pub action: fn(&mut SystemParams),
}

pub fn make_system(
    name: &'static str,
    update_type: SystemUpdate,
    action: fn(&mut SystemParams),
) -> System {
    System {
        name,
        update_type,
        action,
        system_index: -1,
    }
}
