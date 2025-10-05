use super::archetypes::RuntimeGroup;
use super::entities::Entity;
use crate::engine::ecs::my_ecs::ecs::ECS;
use crate::engine::ecs::my_ecs::queries::TQuery;
use std::marker::PhantomData;

pub enum SystemUpdate {
    Update,
    FixedUpdate,
    LateUpdate,
}
pub struct SystemParams<'a> {
    pub world: &'a mut ECS,
    pub system_name: &'static str,
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
    pub group: Option<RuntimeGroup>,
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
        let group_with_entities = A::entities(world, &view);

        Self {
            world,
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
        for i in self.next..self.entities.len() {
            let entity = self.entities[i];
            self.next = i + 1;

            if !A::iter_predicate(entity, self.world, &self.view) {
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
    group: Option<RuntimeGroup>,
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
        let group_with_entities = A::entities(world, &view);

        Self {
            world,
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
        for i in self.next..self.entities.len() {
            let entity = self.entities[i];
            self.next = i + 1;

            if !A::iter_predicate(entity, self.world, &self.view) {
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
