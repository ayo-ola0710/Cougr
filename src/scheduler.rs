use crate::simple_world::SimpleWorld;
use crate::system::System;
use crate::world::World;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use soroban_sdk::Env;

/// A scheduler that executes `System` trait objects in registration order.
///
/// Works with the full `World` type and boxed `System` trait objects.
///
/// # Example
/// ```
/// use cougr_core::scheduler::SystemScheduler;
/// use cougr_core::system::MovementSystem;
/// use cougr_core::world::World;
///
/// let mut scheduler = SystemScheduler::new();
/// scheduler.add_system(MovementSystem);
/// let mut world = World::new();
/// scheduler.run_all(&mut world);
/// assert_eq!(scheduler.system_count(), 1);
/// ```
pub struct SystemScheduler {
    systems: Vec<(String, Box<dyn System<In = (), Out = ()>>)>,
}

impl SystemScheduler {
    /// Create an empty scheduler.
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system to the end of the execution list.
    pub fn add_system<S: System<In = (), Out = ()> + 'static>(&mut self, system: S) {
        let name = core::any::type_name::<S>();
        self.systems.push((String::from(name), Box::new(system)));
    }

    /// Add a named system to the execution list.
    pub fn add_named_system<S: System<In = (), Out = ()> + 'static>(
        &mut self,
        name: &str,
        system: S,
    ) {
        self.systems.push((String::from(name), Box::new(system)));
    }

    /// Execute all registered systems in order.
    pub fn run_all(&mut self, world: &mut World) {
        for (_name, system) in &mut self.systems {
            system.run(world, ());
        }
    }

    /// Returns the number of registered systems.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Returns the names of all registered systems in execution order.
    pub fn system_names(&self) -> Vec<&str> {
        self.systems.iter().map(|(name, _)| name.as_str()).collect()
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// A lightweight scheduler for `SimpleWorld` using plain function pointers.
///
/// Designed for the common pattern in Soroban game contracts where systems
/// are standalone functions taking `(&mut SimpleWorld, &Env)`.
///
/// # Example
/// ```
/// use cougr_core::scheduler::SimpleScheduler;
/// use cougr_core::simple_world::SimpleWorld;
/// use soroban_sdk::{Bytes, Env};
///
/// fn physics_system(world: &mut SimpleWorld, env: &Env) {
///     let entity = world.spawn_entity();
///     world.add_component(entity, soroban_sdk::Symbol::new(env, "physics"), Bytes::new(env));
/// }
/// fn scoring_system(world: &mut SimpleWorld, env: &Env) {
///     let entity = world.spawn_entity();
///     world.add_component(entity, soroban_sdk::Symbol::new(env, "scoring"), Bytes::new(env));
/// }
///
/// let env = Env::default();
/// let mut world = SimpleWorld::new(&env);
/// let mut scheduler = SimpleScheduler::new();
/// scheduler.add_system("physics", physics_system);
/// scheduler.add_system("scoring", scoring_system);
/// scheduler.run_all(&mut world, &env);
/// assert_eq!(scheduler.system_count(), 2);
/// ```
pub struct SimpleScheduler {
    systems: Vec<SimpleSystemEntry>,
}

type SimpleSystemEntry = (&'static str, fn(&mut SimpleWorld, &Env));

impl SimpleScheduler {
    /// Create an empty scheduler.
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a named system function to the execution list.
    pub fn add_system(&mut self, name: &'static str, system: fn(&mut SimpleWorld, &Env)) {
        self.systems.push((name, system));
    }

    /// Execute all registered systems in order.
    pub fn run_all(&mut self, world: &mut SimpleWorld, env: &Env) {
        for (_name, system) in &self.systems {
            system(world, env);
        }
    }

    /// Returns the number of registered systems.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Returns the names of all registered systems in execution order.
    pub fn system_names(&self) -> Vec<&'static str> {
        self.systems.iter().map(|(name, _)| *name).collect()
    }
}

impl Default for SimpleScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::MovementSystem;
    use soroban_sdk::{symbol_short, Bytes, Env};

    #[test]
    fn test_system_scheduler_empty() {
        let mut scheduler = SystemScheduler::new();
        let mut world = World::new();
        scheduler.run_all(&mut world);
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn test_system_scheduler_add_and_run() {
        let mut scheduler = SystemScheduler::new();
        scheduler.add_system(MovementSystem);
        assert_eq!(scheduler.system_count(), 1);

        let mut world = World::new();
        scheduler.run_all(&mut world);
    }

    #[test]
    fn test_system_scheduler_named() {
        let mut scheduler = SystemScheduler::new();
        scheduler.add_named_system("movement", MovementSystem);
        let names = scheduler.system_names();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "movement");
    }

    #[test]
    fn test_simple_scheduler_empty() {
        let mut scheduler = SimpleScheduler::new();
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        scheduler.run_all(&mut world, &env);
        assert_eq!(scheduler.system_count(), 0);
    }

    fn test_system_a(world: &mut SimpleWorld, env: &Env) {
        // Add a marker component to entity 1 to prove this system ran
        let e1 = world.spawn_entity();
        let data = Bytes::from_array(env, &[0xAA]);
        world.add_component(e1, symbol_short!("sys_a"), data);
    }

    fn test_system_b(world: &mut SimpleWorld, env: &Env) {
        let e2 = world.spawn_entity();
        let data = Bytes::from_array(env, &[0xBB]);
        world.add_component(e2, symbol_short!("sys_b"), data);
    }

    #[test]
    fn test_simple_scheduler_execution_order() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system("system_a", test_system_a);
        scheduler.add_system("system_b", test_system_b);
        assert_eq!(scheduler.system_count(), 2);

        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        scheduler.run_all(&mut world, &env);

        // system_a spawns entity 1 with "sys_a"
        assert!(world.has_component(1, &symbol_short!("sys_a")));
        // system_b spawns entity 2 with "sys_b"
        assert!(world.has_component(2, &symbol_short!("sys_b")));
    }

    #[test]
    fn test_simple_scheduler_names() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system("physics", test_system_a);
        scheduler.add_system("scoring", test_system_b);

        let names = scheduler.system_names();
        assert_eq!(names, alloc::vec!["physics", "scoring"]);
    }
}
