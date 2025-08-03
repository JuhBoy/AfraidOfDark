use std::{any::TypeId, marker::PhantomData};

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
