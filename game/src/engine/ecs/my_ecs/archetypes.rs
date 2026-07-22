use super::entities::Entity;
use crate::engine::ecs::my_ecs::{
    components::{ComponentMetaData, ComponentStorage},
    ecs::GroupedEntity,
    utils::GroupMask,
};
use std::{any::TypeId, cmp::Ordering, marker::PhantomData};

// Manager =======================
//
#[derive(PartialEq, Eq)]
pub enum MatchType {
    Exact,
    Partial,
}
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

    pub fn contains(&self, mask: &GroupMask) -> bool {
        let is_superset: bool = self.groups_mask.is_superset_of(mask);
        is_superset
    }

    pub fn groups_len(&self) -> usize {
        self.groups.len()
    }
}
#[derive(Debug, Clone, Copy)]
pub struct RuntimeGroup {
    pub mask: GroupMask, // components storage index included in this group
    pub len: u32,        // len of entities included in this group
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

    pub fn get_supersets_with_archetype(
        &mut self,
        arch_id: usize,
        group_mask: &GroupMask,
        match_type: MatchType,
    ) -> &mut [RuntimeGroup] {
        let archetype = self
            .archetypes
            .get(arch_id)
            .expect("archetype index is invalid");
        let mut start = 0;
        let group_len = archetype.groups_len();

        for (i, group) in archetype.groups.iter().enumerate() {
            if match_type == MatchType::Exact && !group.mask.is_match(group_mask) {
                continue;
            }
            // NOTE(JuH): intersect should be enough but let's make sure that the archetype group contains all of the requested
            if match_type == MatchType::Partial && !group.mask.is_superset_of(group_mask) {
                continue;
            }

            start = i;
            break;
        }

        &mut self.archetypes[arch_id].groups[start..group_len]
    }

    pub fn get_supersets(
        &mut self,
        group_mask: &GroupMask,
        match_type: MatchType,
    ) -> Option<&mut [RuntimeGroup]> {
        let result = self
            .archetypes
            .iter_mut()
            .find(|arch| arch.groups_mask.is_superset_of(group_mask))?;

        let mut mb_start: Option<usize> = None;

        for (index, group) in result.groups.iter().enumerate() {
            if match_type == MatchType::Exact && !group.mask.is_match(group_mask) {
                continue;
            }
            // NOTE(JuH): intersect should be enough but let's make sure that the archetype group contains all of the requested
            if match_type == MatchType::Partial && !group.mask.is_superset_of(group_mask) {
                continue;
            }

            mb_start = Option::from(index);
            break;
        }

        let start = mb_start?;
        let end = result.groups.len();

        Option::from(&mut result.groups[start..end])
    }

    #[must_use]
    pub fn find_group_exact_match(&self, mask: &GroupMask) -> Option<(usize, RuntimeGroup)> {
        for (index, archetype) in self.archetypes.iter().enumerate() {
            if !archetype.contains(mask) {
                continue;
            }

            for group in archetype.groups.iter() {
                if !group.mask.is_match(mask) {
                    continue;
                }

                return Some((index, *group));
            }
        }

        None
    }

    #[must_use]
    pub fn find_group(&self, mask: &GroupMask) -> Option<RuntimeGroup> {
        for archetype in self.archetypes.iter() {
            if !archetype.contains(mask) {
                continue;
            }

            for group in archetype.groups.iter() {
                if !group.mask.eq(mask) {
                    continue;
                }

                return Some(*group);
            }
        }

        None
    }

    #[must_use]
    pub fn has_group(&self, mask: &GroupMask) -> bool {
        for archetype in self.archetypes.iter() {
            if !archetype.contains(mask) {
                continue;
            }

            for group in archetype.groups.iter() {
                if !group.mask.eq(mask) {
                    continue;
                }

                return true;
            }
        }

        false
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

                    insert_index = index + 1;
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

                    insert_index = index;
                    break;
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

    pub fn reset(&mut self) { 
        self.archetypes.clear();
        self.layouts.clear();
    }
}

// Abstract Definition =======================
//
pub trait TAbstractComponentData: 'static {
    fn get_type(&self) -> TypeId;
    fn get_name(&self) -> &'static str;
    fn allocate_buffer(&self, storage: &mut ComponentStorage) -> Option<ComponentMetaData>;
    fn get_storage_metadata(&self, storage: &ComponentStorage) -> ComponentMetaData;
    fn add_component(&mut self, storage: &ComponentStorage) -> bool;
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

    fn get_storage_metadata(&self, storage: &ComponentStorage) -> ComponentMetaData {
        storage.get_storage_metadata::<T>()
    }

    fn add_component(&mut self, _storage: &ComponentStorage) -> bool {
        // storage.add_component::<T>(entity, );
        true
    }
}

pub struct ComponentData {
    #[allow(unused)]
    pub metadata: &'static dyn TAbstractComponentData,
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

pub trait ComponentSet {
    #[must_use]
    fn insert(entity: &Entity, storage: &mut ComponentStorage, comps: Self) -> bool;
    fn group_mask(storage: &ComponentStorage) -> GroupMask;
    fn group(
        entity: &GroupedEntity,
        archetypes: &mut ArchetypesManager,
        storage: &mut ComponentStorage,
    ) -> bool;
    fn ungroup(
        entity: Entity,
        archetypes: &mut ArchetypesManager,
        storage: &mut ComponentStorage,
    ) -> bool;
    fn remove(entity: Entity, storage: &mut ComponentStorage) -> u32;
}

macro_rules! generate_component_set {
    ( ($( ($components:tt, $index:tt) ),+), $count:tt ) => {

      impl<$($components),*> ComponentSet for ($($components,)*)
      where
        $($components: 'static),*
      {
        fn insert(entity: &Entity, storage: &mut ComponentStorage, comps: Self) -> bool {

            $(
              let added = storage.add_component(*entity, comps.$index);
              if !added {
                return false;
              }
            )*

            return true;
        }

        fn group_mask(storage: &ComponentStorage) -> GroupMask {
            let mut mask: GroupMask = GroupMask::new(None);

            $(
                let store: ComponentMetaData = storage.get_storage_metadata::<$components>();
                mask.set(store.index as u8);
            )*

            mask
        }

        fn group(grouped_entity: &GroupedEntity,
          archetypes: &mut ArchetypesManager,
          storage: &mut ComponentStorage,
        ) -> bool {
            let entity = &grouped_entity.entity;
            let request_group = Self::group_mask(storage);

            // leave if the entity is already a superset of the requested groups
            if grouped_entity.group.is_superset_of(&request_group) {
                return false;
            }

            let merged_group = GroupMask::new(Some(grouped_entity.group.get_raw() | request_group.get_raw()));
            let is_updating = !grouped_entity.group.is_empty();

            let archetype = archetypes.find_group_exact_match(&merged_group);

            // it is completely ok to fail grouping when no matching runtime groups exist
            let Some((archetype_id, runtime_group)) = archetype else {
                {
                    let entity_id = entity.id;
                    println!("entity {entity_id} has no matching group");
                }
                return false;
            };

            // NOTE(JuH): updating path is way lower in performances because most of the computation is note expanded by the macro
            if is_updating {
                let supersets = archetypes.get_supersets_with_archetype(archetype_id, &runtime_group.mask, MatchType::Exact);
                let mut should_incr = false;

                for index in (0..supersets.len()).rev() {
                    let current_group_len: usize = supersets[index].len as usize;
                    let mut current_group: GroupMask = supersets[index].mask;

                    while !current_group.is_empty() {
                        let store_id = {
                            let id = current_group.least_one() as usize;
                            id
                        };
                        current_group.and(current_group.get_raw() - 1);

                        let mut store = storage.get_storage_mut_by_id(store_id);
                        let mut entity_pos: usize = store.get_entity_index(*entity).expect("entity is not in the group, this shouldn't be possible");

                        let curr_group_start: usize = {
                            let prev = if index == 0 { 0 } else { index - 1 };
                            let prev_group: Option<&RuntimeGroup> = supersets.get(prev);
                            prev_group.map_or(0 as usize, |g| g.len as usize)
                        };

                        // align all them at index 0 !!!

                        let is_outside = entity_pos >= current_group_len;
                        should_incr |= is_outside;

                        if entity_pos <= curr_group_start {
                            continue;
                        }

                        if entity_pos >= current_group_len {
                            let last_ett = store.get_entity(current_group_len).unwrap();
                            store.swap_untyped(*entity, last_ett);
                            entity_pos = current_group_len;
                        }

                        if entity_pos != curr_group_start {
                            let start_entity = store.get_entity(curr_group_start).unwrap();
                            store.swap_untyped(*entity, start_entity);
                        }
                    }

                    if should_incr {
                        let runtime_group = &mut supersets[index];
                        runtime_group.len += 1;
                    }
                }
            } else {
                // create tuple to store all storages
                let mut stores = ($(storage.get_storage_mut::<$components>(),)*);
                let mut swapping_entities: [Entity; $count] = [*entity; $count];

                let supersets = archetypes.get_supersets_with_archetype(archetype_id, &runtime_group.mask, MatchType::Exact);

                (0..supersets.len()).for_each(|superset_id| {
                    let superset = &mut supersets[superset_id];
                    let swap_index = superset.len as usize;

                    $({
                        let store = &mut stores.$index;

                        if let Some(store_ett) = store.get_entity(swap_index) {
                            let swaped = store.swap::<$components>(swapping_entities[$index], store_ett);
                            swapping_entities[$index] = store_ett;

                            if !swaped {
                                panic!("swap failed for component {:?}", TypeId::of::<$components>());
                            }
                        }
                    })*

                    superset.len += 1;
                });
            }

            true
        }

        fn ungroup(
            entity: Entity,
            archetypes: &mut ArchetypesManager,
            storage: &mut ComponentStorage,
        ) -> bool
        {
            let group_mask: GroupMask = Self::group_mask(storage);
            let groups_option = archetypes.get_supersets(&group_mask, MatchType::Partial);
            let mut ungrouped: bool = false;

            for group in groups_option.unwrap_or(&mut []) {
                if group.len == 0 {
                    continue;
                }
                if !group.mask.intersects(&group_mask) {
                    break;
                }
                let mut has_swaped: bool = false;
                let mut all_masks = group.mask;

                while !all_masks.is_empty() {
                    let store_id = all_masks.least_one() as usize;
                    all_masks.and(all_masks.get_raw() - 1);

                    let mut store = storage.get_storage_mut_by_id(store_id);
                    let entity_index = store.get_entity_index(entity);

                    if let Some(entity_index) = entity_index {
                        // NOTE(JuH): the entity is not part of this superset
                        if entity_index >= group.len as usize {
                            continue;
                        }

                        let swap_index: usize  = group.len as usize - 1;

                        if swap_index != entity_index {
                            let last_entity = store.get_entity(swap_index).unwrap();
                            has_swaped |= store.swap_untyped(entity, last_entity);

                            println!("swaped entity: {:?}[{}] <-> {:?}[{}] (store: {}, gid: {})", entity, entity_index, last_entity, swap_index, store_id, group.mask.get_raw());
                        } else {
                            has_swaped = true;
                        }
                    }
                }

                if has_swaped {
                    println!("reduce group len by 1 for gid: {}", group.mask.get_raw());
                    group.len -= 1;
                    ungrouped = true;
                }
            }

            ungrouped
        }

        fn remove(entity: Entity, storage: &mut ComponentStorage) -> u32 {
            let mut stores = ($(storage.get_storage_mut::<$components>(),)*);
            let mut removed_count: u32 = 0;

            $({
                let store = &mut stores.$index;
                let removed = store.remove(entity);

                if removed.is_some() { 
                    removed_count += 1;
                }
            })*

            removed_count
        }
      }
    };
}

generate_component_set!(((A, 0)), 1);
generate_component_set!(((A, 0), (B, 1)), 2);
generate_component_set!(((A, 0), (B, 1), (C, 2)), 3);
generate_component_set!(((A, 0), (B, 1), (C, 2), (D, 3)), 4);
generate_component_set!(((A, 0), (B, 1), (C, 2), (D, 3), (E, 4)), 5);
