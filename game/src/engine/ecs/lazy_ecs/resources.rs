use std::any::Any;

pub trait LazyResourceTrait {
    fn id() -> usize;
}

pub struct Resources {
    pub(crate) data: Vec<Option<Box<dyn Any>>>,
}

impl Resources {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn get<ResourceT: LazyResourceTrait + 'static>(&self) -> &ResourceT {
        let id = ResourceT::id();
        let resource: &ResourceT = self.data[id]
            .as_ref()
            .expect("resource container not allocated")
            .downcast_ref::<ResourceT>()
            .expect("failed to get resource ref");

        resource
    }

    pub fn get_mut<ResourceMT: LazyResourceTrait + 'static>(&mut self) -> &mut ResourceMT {
        let id = ResourceMT::id();

        let resource: &mut ResourceMT = self.data[id]
            .as_mut()
            .expect("resource container not allocated")
            .downcast_mut::<ResourceMT>()
            .expect("failed to get resource mut ref");

        resource
    }

    pub fn try_allocate<ResourceT: LazyResourceTrait>(&mut self, instance: ResourceT) -> bool
    where
        ResourceT: 'static,
    {
        let id = ResourceT::id();

        if id >= self.data.len() {
            let new_size = (id + 1).next_power_of_two();
            self.data.resize_with(new_size, || None);
        }
    
        if self.data[id].is_some() { 
            return false;
        }

        self.data[id] = Some(Box::new(instance));

        true
    }
}

#[macro_export]
macro_rules! make_lazy_resources_impl {
    ($index:expr;) => {};

    ($index:expr; $resource:ty $(, $rest:ty)*) => {
        impl $crate::engine::ecs::lazy_ecs::resources::LazyResourceTrait for $resource {
            fn id() -> usize {
                $index
            }
        }

        $crate::make_lazy_resources_impl!($index + 1; $($rest),*);
    };
}

#[macro_export]
macro_rules! make_lazy_resources {
    ($($resource:ty),* $(,)?) => {
        $crate::make_lazy_resources_impl!(0; $($resource),*);
    };
}
