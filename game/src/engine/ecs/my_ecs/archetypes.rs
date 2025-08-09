use std::{any::TypeId, cmp::Ordering, marker::PhantomData};

use crate::engine::ecs::my_ecs::systems::TQuery;

// Manager =======================
//
pub struct ArchetypeLayout {
    pub components: &'static [ComponentData],
    pub start_index: usize,
    pub super_set_len: usize,
}
pub struct ArchetypesManager {
    pub layouts: Vec<ArchetypeLayout>,
}
pub enum ArchetypeRegisterState {
    Subset,       //
    Equals,       // the exact same archetype has been registered
    SuperSet,     // the new Archetype is a superset of another Archertype
    Incompatible, // The Archetype is intersecting with others but is not preserving order.
}

impl ArchetypesManager {
    pub fn new() -> Self {
        Self { layouts: vec![] }
    }

    pub fn get_archetyp<Q>()
    where
        Q: TQuery,
    {
    }

    pub fn register(&mut self, components: &[ComponentData]) -> bool {
        let mut inserted: bool = false;

        for layout in self.layouts.iter() {
            // if the components are completly different => all good !
            if layout.components.iter().all(|c| !components.contains(c)) {
                continue;
            }

            match components.len().cmp(&layout.components.len()) {
                Ordering::Less => {
                    let surject = components.iter().all(|c| layout.components.contains(c)); 
                }
                Ordering::Equal => {}
                Ordering::Greater => {}
            };
        }

        inserted
    }
}

// Abstract Definition =======================
//
pub trait TAbstractComponentData: 'static {
    #[must_use]
    fn get_type(&self) -> TypeId;
}
pub struct AbstractComponentData<T>
where
    T: 'static,
{
    _phatom: PhantomData<T>,
}
impl<T> TAbstractComponentData for AbstractComponentData<T>
where
    T: 'static,
{
    fn get_type(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

pub struct ComponentData {
    #[allow(unused)]
    metadata: &'static dyn TAbstractComponentData,
}
impl ComponentData {
    pub const fn new<T>() -> Self
    where
        T: 'static,
    {
        Self {
            metadata: &AbstractComponentData::<T> {
                _phatom: PhantomData::<T>,
            },
        }
    }
}
impl Eq for ComponentData {}
impl PartialEq for ComponentData {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.get_type() == other.metadata.get_type()
    }
}

// Archetype definitions =========================
//
pub trait ArchetypeDefinition {
    const COMPONENTS: &'static [ComponentData];
}

impl<A, B> ArchetypeDefinition for (A, B)
where
    A: 'static,
    B: 'static,
{
    const COMPONENTS: &'static [ComponentData] =
        &[ComponentData::new::<A>(), ComponentData::new::<B>()];
}

impl<A, B, C> ArchetypeDefinition for (A, B, C)
where
    A: 'static,
    B: 'static,
    C: 'static,
{
    const COMPONENTS: &'static [ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
    ];
}

impl<A, B, C, D> ArchetypeDefinition for (A, B, C, D)
where
    A: 'static,
    B: 'static,
    C: 'static,
    D: 'static,
{
    const COMPONENTS: &'static [ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
        ComponentData::new::<D>(),
    ];
}
