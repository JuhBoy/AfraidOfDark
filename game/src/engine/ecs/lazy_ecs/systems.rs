use glfw::ffi::GLFW_WAYLAND_PREFER_LIBDECOR;
use glfw::Key::W;

use super::archetypes::RuntimeGroup;
use super::entities::Entity;
use crate::engine::ecs::lazy_ecs::archetypes::{Archetype, ArchetypesManager};
use crate::engine::ecs::lazy_ecs::components::ComponentStorage;
use crate::engine::ecs::lazy_ecs::ecs::{ECSStats, ECS};
use crate::engine::ecs::lazy_ecs::queries::TQuery;
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;

pub enum SystemUpdateType {
    Update,
    FixedUpdate,
    LateUpdate,
}
pub struct SystemParams<'a> {
    pub world: &'a mut ECS,
    pub system_name: &'static str,
}

///////////////////////////////////

pub struct QuerySystems<'a> {
    pub storage: &'a ComponentStorage,
    pub archetypes: Ref<'a, ArchetypesManager>,
    pub stats: &'a RefCell<ECSStats>,
}

pub struct Query<'a, A>
where
    A: TQuery,
{
    pub query_systems: QuerySystems<'a>,
    _phantom: PhantomData<A>,
}

impl<'a, A> Query<'a, A>
where
    A: TQuery,
{
    pub fn new(world: &'a ECS) -> Self {
        Self {
            query_systems: QuerySystems {
                storage: &world.component_storage,
                archetypes: world.archetypes.borrow(),
                stats: &world.stats,
            },
            _phantom: PhantomData,
        }
    }

    pub fn iter(&'a self) -> QueryRef<'a, A> {
        QueryRef::new(&self.query_systems)
    }

    pub fn iter_mut(&'a self) -> QueryMut<'a, A> {
        QueryMut::new(&self.query_systems)
    }

    pub fn iter_with_entity(&'a self) -> QueryRef<'a, A> {
        QueryRef::new(&self.query_systems)
    }
}

pub struct QueryRef<'iter, A>
where
    A: TQuery,
{
    systems: &'iter QuerySystems<'iter>,
    pub entities: &'iter [Entity],
    pub group: Option<RuntimeGroup>,
    pub view: A::View<'iter>,
    pub _phantom: PhantomData<A>,

    next: usize,
}
impl<'iter, A> QueryRef<'iter, A>
where
    A: TQuery,
{
    fn new(world: &'iter QuerySystems<'iter>) -> Self {
        let view = A::borrow_storage(world.storage);
        let group_with_entities = A::entities(world.storage, &world.archetypes, &view);

        Self {
            systems: world,
            entities: group_with_entities.1,
            group: group_with_entities.0,
            view,
            _phantom: PhantomData::<A>,
            next: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.group.map_or(self.entities.len(), |f| f.len as usize)
    }

    pub fn is_empty(&self) -> bool {
        let len = self.len();
        len == 0
    }
}
impl<'a, A> Iterator for QueryRef<'a, A>
where
    A: TQuery,
{
    type Item = A::Item<'a>;

    fn next(&mut self) -> Option<A::Item<'a>> {
        let mut is_broken: bool = false;

        for i in self.next..self.len() {
            let entity = self.entities[i];
            self.next = i + 1;

            if !self.group.is_some() && !A::iter_predicate(entity, self.systems.storage, &self.view) {
                is_broken = true;
                continue;
            }

            let data = A::get_dense(entity, self.systems.storage, &self.view);

            return Some(data);
        }

        if is_broken {
            let mut borrow_stats = self.systems.stats.borrow_mut();
            borrow_stats.archetypes_broken += 1
        }

        None
    }
}

pub struct QueryMut<'a, A>
where
    A: TQuery,
{
    systems: &'a QuerySystems<'a>,
    entities: &'a [Entity],
    group: Option<RuntimeGroup>,
    view: A::View<'a>,
    _phantom: PhantomData<&'a A>,

    next: usize,
}
impl<'a, A> QueryMut<'a, A>
where
    A: TQuery,
{
    pub fn new(world: &'a QuerySystems<'a>) -> Self {
        let view = A::borrow_storage_mut(world.storage);
        let group_with_entities = A::entities(world.storage, &world.archetypes, &view);

        Self {
            systems: world,
            view,
            _phantom: PhantomData,
            entities: group_with_entities.1,
            group: group_with_entities.0,
            next: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.group.map_or(self.entities.len(), |f| f.len as usize)
    }

    pub fn is_empty(&self) -> bool {
        let len = self.len();
        len == 0
    }
}

impl<'a, A> Iterator for QueryMut<'a, A>
where
    A: TQuery,
{
    type Item = A::ItemMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut is_broken: bool = false;

        for i in self.next..self.len() {
            let entity = self.entities[i];
            self.next = i + 1;

            if !self.group.is_some() && !A::iter_predicate(entity, self.systems.storage, &self.view) {
                is_broken = true;
                continue;
            }

            let data = A::get_dense_mut(entity, self.systems.storage, &self.view);

            return Some(data);
        }

        if is_broken {
            self.systems.stats.borrow_mut().archetypes_broken += 1
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
    pub update_type: SystemUpdateType,
    pub action: fn(&mut SystemParams),
}

pub fn make_system(
    name: &'static str,
    update_type: SystemUpdateType,
    action: fn(&mut SystemParams),
) -> System {
    System {
        name,
        update_type,
        action,
        system_index: -1,
    }
}
