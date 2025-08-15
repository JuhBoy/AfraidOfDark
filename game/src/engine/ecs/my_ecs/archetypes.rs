use std::{any::TypeId, cmp::Ordering, marker::PhantomData, ops::Range};

use crate::engine::ecs::my_ecs::{
    components::{ComponentMetaData, ComponentStorage},
    systems::TQuery,
    utils::GroupMask,
};

// Manager =======================
//
pub struct ArchetypeLayout {
    pub components: &'static [ComponentData],
    pub set_len: usize,
}
pub struct Archetype {
    pub groups_mask: GroupMask, 
    pub groups: Vec<RuntimeGroup>,
}
impl Archetype {
    pub fn match_mask(&self, mask: GroupMask) -> bool {
        self.groups_mask.is_match(&mask)
    }

    pub fn groups_len(&self) -> usize {
        self.groups.len()
    }
}
pub struct RuntimeGroup {
    mask: GroupMask, // components storage index included in this group
    len: u32, // len of entities included in this group
}
impl Default for RuntimeGroup {
    fn default() -> Self {
        Self {
            mask: GroupMask::new(None),
            len: 0,
        }
    }
}
pub struct ArchetypesManager {
    pub layouts: Vec<ArchetypeLayout>,
    pub archetypes: Vec<Archetype>,
}

impl ArchetypeLayout {}

impl ArchetypesManager {
    pub fn new() -> Self {
        Self {
            layouts: vec![],
            archetypes: vec![],
        }
    }

    pub fn get_archetyp<Q>()
    where
        Q: TQuery,
    {
    }

    #[must_use]
    pub fn flush_archetypes(&mut self, component_storage: &mut ComponentStorage) -> usize {
        assert_eq!(self.archetypes.len(), 0);

        let mut i = 0;

        while i < self.layouts.len() {
            let archetype_length = self.layouts[i].set_len;
            let mut groups: Vec<RuntimeGroup> = Vec::with_capacity(archetype_length);
            let mut archetype_mask = GroupMask::new(None);

            for j in 0..archetype_length {
                let archetype_id = i + j;
                let current = &mut self.layouts[archetype_id];
                let mut group = RuntimeGroup::default();

                current.components.iter().for_each(|f| {
                    let meta = f.metadata.allocate_buffer(component_storage);

                    if let Some(group_meta) = meta {
                        assert!(group_meta.index <= GroupMask::max_bit_shift());

                        let group_id: u8 = group_meta.index as u8;

                        group.mask.set(group_id);
                        archetype_mask.set(group_id);
                    } else {
                        panic!("[ECS] couldn't find metadata for group");
                    }
                });

                groups.push(group);
            }

            i += archetype_length;

            self.archetypes.push(Archetype {
                groups,
                groups_mask: archetype_mask,
            });
        }

        self.archetypes.len()
    }

    #[must_use]
    pub fn register(&mut self, components: &'static [ComponentData]) -> bool {
        let mut parent_index: Option<usize> = None;
        let mut insert_index: usize = self.layouts.len();

        for (index, layout) in self.layouts.iter().enumerate() {
            // if the components are completly different => all good !
            if layout.components.iter().all(|c| !components.contains(c)) {
                continue;
            }

            match components.len().cmp(&layout.components.len()) {
                Ordering::Less => {
                    let intersects: bool = components.iter().all(|c| layout.components.contains(c));

                    if !intersects {
                        println!("[ECS] Subset of Archetype cannot varies in type")
                    }

                    if parent_index.is_none() {
                        parent_index = Some(index);
                    }

                    insert_index = index;
                    break;
                }
                Ordering::Equal => {
                    // they are equals but intersects, we can't accept it as a sub/super Set
                    return false;
                }
                Ordering::Greater => {
                    let intersects: bool = layout.components.iter().all(|c| components.contains(c));

                    if !intersects {
                        panic!("[ECS] Superset of Archetype requires to match every components")
                    }

                    if parent_index.is_none() {
                        parent_index = Some(index);
                    }

                    insert_index = index + 1;
                }
            };
        }

        // only the parent (lowest components length) has the set_len property
        let mut set_len: usize = 1;
        if let Some(parent_id) = parent_index {
            let parent = self.layouts.get_mut(parent_id).unwrap();

            match insert_index.cmp(&parent_id) {
                Ordering::Less => panic!("[ECS] invalid insertion"),
                Ordering::Greater => parent.set_len += 1, // increase the parent set
                Ordering::Equal => {
                    // replace the current parent
                    set_len = parent.set_len + 1;
                    parent.set_len = 1;
                }
            }
        };

        self.layouts.insert(
            insert_index,
            ArchetypeLayout {
                components,
                set_len,
            },
        );

        true
    }
}

// Abstract Definition =======================
//
pub trait TAbstractComponentData: 'static {
    #[must_use]
    fn get_type(&self) -> TypeId;
    fn get_name(&self) -> &'static str;
    fn allocate_buffer(&self, storage: &mut ComponentStorage) -> Option<ComponentMetaData>;
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

    fn get_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn allocate_buffer(&self, storage: &mut ComponentStorage) -> Option<ComponentMetaData> {
        storage.allocate::<T>()
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
impl Ord for ComponentData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.metadata.get_name().cmp(other.metadata.get_name())
    }
}
impl PartialOrd for ComponentData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.metadata.get_name().cmp(other.metadata.get_name()))
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
