use bevy_ecs::schedule::ScheduleLabel;

#[derive(Hash, Debug, Eq, PartialEq, Clone, Copy, ScheduleLabel)]
pub struct EcsUpdateSchedule;

#[derive(Hash, Debug, Eq, PartialEq, Clone, Copy, ScheduleLabel)]
pub struct EcsLateUpdateSchedule;