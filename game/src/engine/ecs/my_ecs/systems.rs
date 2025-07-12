use std::any::TypeId;
use crate::engine::ecs::my_ecs::entities::Entity;

pub enum SystemUpdate { Update, FixedUpdate, LateUpdate }
pub struct System {
  pub name: &'static str,

  pub update_type: SystemUpdate,
  pub action: fn(),
  pub query: Vec<TypeId>,
}
pub struct SystemQueryResponse {
  pub entities: Vec<Entity>,
}