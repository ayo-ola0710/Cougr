use crate::hooks::{HookRegistry, OnAddHook, OnRemoveHook};
use crate::resource::{Resource, ResourceTrait};
use crate::scheduler::{ScheduleError, ScheduleStage, SimpleScheduler, SystemConfig, SystemGroup};
use crate::simple_world::SimpleWorld;
use crate::system::{AppSystem, SystemContext};
use alloc::vec::Vec;
use soroban_sdk::{Env, Symbol};

/// A plugin that configures systems, hooks, and initial world state.
///
/// Plugins provide a modular way to compose game functionality.
/// Each plugin gets access to a `GameApp` builder during `build()`.
///
/// # Example
/// ```no_run
/// # use cougr_core::plugin::{GameApp, Plugin};
/// # use cougr_core::simple_world::SimpleWorld;
/// # use soroban_sdk::Env;
/// # fn gravity_system(_world: &mut SimpleWorld, _env: &Env) {}
/// # fn collision_system(_world: &mut SimpleWorld, _env: &Env) {}
/// struct PhysicsPlugin;
///
/// impl Plugin for PhysicsPlugin {
///     fn name(&self) -> &'static str { "physics" }
///     fn build(&self, app: &mut GameApp) {
///         app.add_system("gravity", gravity_system);
///         app.add_system("collision", collision_system);
///     }
/// }
/// ```
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn build(&self, app: &mut GameApp);
}

/// Composable plugin group abstraction for `GameApp`.
pub trait PluginGroup {
    fn build(self, app: &mut GameApp);
}

impl<P: Plugin> PluginGroup for P {
    fn build(self, app: &mut GameApp) {
        app.add_plugin(self);
    }
}

impl<A: PluginGroup, B: PluginGroup> PluginGroup for (A, B) {
    fn build(self, app: &mut GameApp) {
        self.0.build(app);
        self.1.build(app);
    }
}

impl<A: PluginGroup, B: PluginGroup, C: PluginGroup> PluginGroup for (A, B, C) {
    fn build(self, app: &mut GameApp) {
        self.0.build(app);
        self.1.build(app);
        self.2.build(app);
    }
}

/// Soroban-first application builder and runtime.
///
/// `GameApp` is the recommended entrypoint for new Cougr projects:
/// it owns the `SimpleWorld`, the validated `SimpleScheduler`, and hook
/// registration in one place. `PluginApp` remains available as an alias for
/// backward compatibility with existing projects.
///
/// # Example
/// ```no_run
/// # use cougr_core::plugin::{GameApp, Plugin};
/// # use cougr_core::scheduler::{ScheduleStage, SystemConfig};
/// # use cougr_core::simple_world::SimpleWorld;
/// # use soroban_sdk::Env;
/// # struct PhysicsPlugin;
/// # struct ScoringPlugin;
/// # fn physics_system(_world: &mut SimpleWorld, _env: &Env) {}
/// # fn scoring_system(_world: &mut SimpleWorld, _env: &Env) {}
/// # impl Plugin for PhysicsPlugin {
/// #     fn name(&self) -> &'static str { "physics" }
/// #     fn build(&self, app: &mut GameApp) { app.add_system("physics", physics_system); }
/// # }
/// # impl Plugin for ScoringPlugin {
/// #     fn name(&self) -> &'static str { "scoring" }
/// #     fn build(&self, app: &mut GameApp) {
/// #         app.add_system_with_config(
/// #             "scoring",
/// #             scoring_system,
/// #             SystemConfig::new().in_stage(ScheduleStage::PostUpdate),
/// #         );
/// #     }
/// # }
/// let env = Env::default();
/// let mut app = GameApp::new(&env);
/// app.add_plugin(PhysicsPlugin);
/// app.add_plugin(ScoringPlugin);
/// app.run(&env).unwrap();
/// let world = app.into_world();
/// assert_eq!(world.version(), 0);
/// ```
pub struct GameApp {
    world: SimpleWorld,
    scheduler: SimpleScheduler,
    hooks: HookRegistry,
    resources: Vec<Resource>,
    plugins_registered: Vec<&'static str>,
    startup_ran: bool,
}

impl GameApp {
    pub fn new(env: &Env) -> Self {
        Self {
            world: SimpleWorld::new(env),
            scheduler: SimpleScheduler::new(),
            hooks: HookRegistry::new(),
            resources: Vec::new(),
            plugins_registered: Vec::new(),
            startup_ran: false,
        }
    }

    pub fn with_world(world: SimpleWorld) -> Self {
        Self {
            world,
            scheduler: SimpleScheduler::new(),
            hooks: HookRegistry::new(),
            resources: Vec::new(),
            plugins_registered: Vec::new(),
            startup_ran: false,
        }
    }

    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        let name = Plugin::name(&plugin);
        if !self.has_plugin(name) {
            self.plugins_registered.push(name);
            Plugin::build(&plugin, self);
        }
        self
    }

    pub fn add_plugins<G: PluginGroup>(&mut self, group: G) -> &mut Self {
        group.build(self);
        self
    }

    /// Add a world/env system to the default `Update` stage.
    pub fn add_system<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut SimpleWorld, &Env) + 'static,
    {
        self.scheduler.add_system(name, system);
        self
    }

    /// Add a world/env system with explicit scheduling rules.
    pub fn add_system_with_config<F>(
        &mut self,
        name: &'static str,
        system: F,
        config: SystemConfig,
    ) -> &mut Self
    where
        F: FnMut(&mut SimpleWorld, &Env) + 'static,
    {
        self.scheduler.add_system_with_config(name, system, config);
        self
    }

    /// Add a world/env system directly to a specific stage.
    pub fn add_system_in_stage<F>(
        &mut self,
        stage: ScheduleStage,
        name: &'static str,
        system: F,
    ) -> &mut Self
    where
        F: FnMut(&mut SimpleWorld, &Env) + 'static,
    {
        self.scheduler.add_system_in_stage(stage, name, system);
        self
    }

    /// Add a context-aware system to the default `Update` stage.
    pub fn add_context_system<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>) + 'static,
    {
        self.scheduler.add_context_system(name, system);
        self
    }

    /// Add a context-aware system with explicit scheduling rules.
    pub fn add_context_system_with_config<F>(
        &mut self,
        name: &'static str,
        system: F,
        config: SystemConfig,
    ) -> &mut Self
    where
        F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>) + 'static,
    {
        self.scheduler
            .add_context_system_with_config(name, system, config);
        self
    }

    /// Add a context-aware system directly to a specific stage.
    pub fn add_context_system_in_stage<F>(
        &mut self,
        stage: ScheduleStage,
        name: &'static str,
        system: F,
    ) -> &mut Self
    where
        F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>) + 'static,
    {
        self.scheduler
            .add_context_system_in_stage(stage, name, system);
        self
    }

    /// Add any pre-built runtime system to the default `Update` stage.
    pub fn add_simple_system<S>(&mut self, name: &'static str, system: S) -> &mut Self
    where
        S: AppSystem + 'static,
    {
        self.scheduler.add_simple_system(name, system);
        self
    }

    /// Add any pre-built runtime system with explicit scheduling rules.
    pub fn add_simple_system_with_config<S>(
        &mut self,
        name: &'static str,
        system: S,
        config: SystemConfig,
    ) -> &mut Self
    where
        S: AppSystem + 'static,
    {
        self.scheduler
            .add_simple_system_with_config(name, system, config);
        self
    }

    /// Add any pre-built runtime system directly to a specific stage.
    pub fn add_simple_system_in_stage<S>(
        &mut self,
        stage: ScheduleStage,
        name: &'static str,
        system: S,
    ) -> &mut Self
    where
        S: AppSystem + 'static,
    {
        self.scheduler
            .add_simple_system_in_stage(stage, name, system);
        self
    }

    /// Add one or more runtime systems using declarative specs.
    pub fn add_systems<G>(&mut self, systems: G) -> &mut Self
    where
        G: SystemGroup,
    {
        self.scheduler.add_systems(systems);
        self
    }

    /// Add one or more runtime systems while forcing them into a stage.
    pub fn add_systems_in_stage<G>(&mut self, stage: ScheduleStage, systems: G) -> &mut Self
    where
        G: SystemGroup,
    {
        self.scheduler.add_systems_in_stage(stage, systems);
        self
    }

    /// Convenience wrapper to register a startup-only system.
    pub fn add_startup_system<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut SimpleWorld, &Env) + 'static,
    {
        self.add_system_with_config(
            name,
            system,
            SystemConfig::new().in_stage(ScheduleStage::Startup),
        )
    }

    pub fn add_hook_on_add(&mut self, component_type: Symbol, hook: OnAddHook) -> &mut Self {
        self.hooks.on_add(component_type, hook);
        self
    }

    pub fn add_hook_on_remove(&mut self, component_type: Symbol, hook: OnRemoveHook) -> &mut Self {
        self.hooks.on_remove(component_type, hook);
        self
    }

    pub fn insert_resource<R: ResourceTrait>(&mut self, env: &Env, resource: &R) -> &mut Self {
        let resource_type = R::resource_type();
        let encoded = Resource::new(resource_type.clone(), resource.serialize(env));

        for index in 0..self.resources.len() {
            if let Some(existing) = self.resources.get(index) {
                if existing.resource_type == resource_type {
                    self.resources[index] = encoded;
                    return self;
                }
            }
        }

        self.resources.push(encoded);
        self
    }

    pub fn get_resource<R: ResourceTrait>(&self, env: &Env) -> Option<R> {
        let resource_type = R::resource_type();
        for index in 0..self.resources.len() {
            if let Some(resource) = self.resources.get(index) {
                if resource.resource_type == resource_type {
                    return R::deserialize(env, &resource.data);
                }
            }
        }
        None
    }

    pub fn remove_resource<R: ResourceTrait>(&mut self) -> Option<Resource> {
        let resource_type = R::resource_type();
        let mut found = None;
        let mut retained = Vec::new();

        for index in 0..self.resources.len() {
            if let Some(resource) = self.resources.get(index) {
                if resource.resource_type == resource_type {
                    found = Some(resource.clone());
                } else {
                    retained.push(resource.clone());
                }
            }
        }

        if found.is_some() {
            self.resources = retained;
        }

        found
    }

    pub fn world(&self) -> &SimpleWorld {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut SimpleWorld {
        &mut self.world
    }

    pub fn scheduler(&self) -> &SimpleScheduler {
        &self.scheduler
    }

    pub fn hooks(&self) -> &HookRegistry {
        &self.hooks
    }

    pub fn resources(&self) -> &Vec<Resource> {
        &self.resources
    }

    pub fn run_startup(&mut self, env: &Env) -> Result<(), ScheduleError> {
        if !self.startup_ran {
            self.scheduler
                .run_stage(ScheduleStage::Startup, &mut self.world, env)?;
            self.startup_ran = true;
        }
        Ok(())
    }

    /// Run one gameplay tick.
    pub fn run(&mut self, env: &Env) -> Result<(), ScheduleError> {
        self.run_startup(env)?;
        self.scheduler
            .run_stage(ScheduleStage::PreUpdate, &mut self.world, env)?;
        self.scheduler
            .run_stage(ScheduleStage::Update, &mut self.world, env)?;
        self.scheduler
            .run_stage(ScheduleStage::PostUpdate, &mut self.world, env)?;
        self.scheduler
            .run_stage(ScheduleStage::Cleanup, &mut self.world, env)?;
        Ok(())
    }

    pub fn run_stage(&mut self, stage: ScheduleStage, env: &Env) -> Result<(), ScheduleError> {
        if stage == ScheduleStage::Startup {
            return self.run_startup(env);
        }
        self.scheduler.run_stage(stage, &mut self.world, env)
    }

    pub fn configure_system(
        &mut self,
        name: &str,
        config: SystemConfig,
    ) -> Result<&mut Self, ScheduleError> {
        self.scheduler.configure_system(name, config)?;
        Ok(self)
    }

    pub fn into_world(self) -> SimpleWorld {
        self.world
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins_registered.len()
    }

    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins_registered.contains(&name)
    }

    pub fn system_count(&self) -> usize {
        self.scheduler.system_count()
    }
}

/// Backwards-compatible alias for the previous app entrypoint name.
pub type PluginApp = GameApp;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::GameState as RuntimeGameState;
    use soroban_sdk::{symbol_short, Bytes, Env};

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &'static str {
            "test_plugin"
        }

        fn build(&self, app: &mut GameApp) {
            app.add_system("test_system", test_system_fn);
        }
    }

    struct HookPlugin;

    impl Plugin for HookPlugin {
        fn name(&self) -> &'static str {
            "hook_plugin"
        }

        fn build(&self, app: &mut GameApp) {
            app.add_hook_on_add(symbol_short!("pos"), noop_add_hook);
        }
    }

    fn test_system_fn(world: &mut SimpleWorld, env: &Env) {
        let e = world.spawn_entity();
        let data = Bytes::from_array(env, &[0xFF]);
        world.add_component(e, symbol_short!("marker"), data);
    }

    fn noop_add_hook(
        _entity_id: crate::simple_world::EntityId,
        _component_type: &Symbol,
        _data: &Bytes,
    ) {
    }

    #[test]
    fn test_game_app_new() {
        let env = Env::default();
        let app = GameApp::new(&env);
        assert_eq!(app.plugin_count(), 0);
        assert_eq!(app.system_count(), 0);
    }

    #[test]
    fn test_add_plugin() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_plugin(TestPlugin);

        assert_eq!(app.plugin_count(), 1);
        assert!(app.has_plugin("test_plugin"));
        assert_eq!(app.system_count(), 1);
    }

    #[test]
    fn test_duplicate_plugin_skipped() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_plugin(TestPlugin);
        app.add_plugin(TestPlugin);

        assert_eq!(app.plugin_count(), 1);
        assert_eq!(app.system_count(), 1);
    }

    #[test]
    fn test_plugin_configures_hooks() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_plugin(HookPlugin);

        assert_eq!(app.plugin_count(), 1);
        assert_eq!(app.hooks().add_hook_count(), 1);
    }

    #[test]
    fn test_run_executes_systems() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_plugin(TestPlugin);
        app.run(&env).unwrap();

        assert!(app.world().has_component(1, &symbol_short!("marker")));
    }

    #[test]
    fn test_into_world() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_plugin(TestPlugin);
        app.run(&env).unwrap();

        let world = app.into_world();
        assert!(world.has_component(1, &symbol_short!("marker")));
    }

    #[test]
    fn test_with_world() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        world.add_component(e1, symbol_short!("pre"), data);

        let app = GameApp::with_world(world);
        assert!(app.world().has_component(e1, &symbol_short!("pre")));
    }

    #[test]
    fn test_add_system_directly() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_system("direct", test_system_fn);

        assert_eq!(app.system_count(), 1);
        assert_eq!(app.plugin_count(), 0);
    }

    #[test]
    fn test_multiple_plugins() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_plugin(TestPlugin);
        app.add_plugin(HookPlugin);

        assert_eq!(app.plugin_count(), 2);
        assert!(app.has_plugin("test_plugin"));
        assert!(app.has_plugin("hook_plugin"));
    }

    #[test]
    fn test_startup_system_runs_once() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_startup_system("startup", test_system_fn);

        app.run(&env).unwrap();
        app.run(&env).unwrap();

        assert!(app.world().has_component(1, &symbol_short!("marker")));
        assert_eq!(app.world().component_count(&symbol_short!("marker")), 1);
    }

    #[test]
    fn test_add_plugins_group_and_resources() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_plugins((TestPlugin, HookPlugin));

        let mut state = RuntimeGameState::new();
        state.increment_score(42);
        app.insert_resource(&env, &state);

        let loaded = app.get_resource::<RuntimeGameState>(&env).unwrap();
        assert_eq!(loaded.score, 42);
        assert_eq!(app.plugin_count(), 2);
        assert_eq!(app.resources().len(), 1);
    }

    #[test]
    fn test_add_system_in_stage_and_context_variants() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_system_in_stage(ScheduleStage::PreUpdate, "spawn_a", test_system_fn);
        app.add_context_system_in_stage(ScheduleStage::PostUpdate, "spawn_b", |context| {
            let entity = context.world_mut().spawn_entity();
            let data = Bytes::from_array(context.env(), &[0x0A]);
            context
                .world_mut()
                .add_component(entity, symbol_short!("ctx"), data);
        });

        app.run(&env).unwrap();

        assert!(app.world().has_component(1, &symbol_short!("marker")));
        assert!(app.world().has_component(2, &symbol_short!("ctx")));
    }

    #[test]
    fn test_add_simple_system_and_configure_system() {
        let env = Env::default();
        let mut app = GameApp::new(&env);
        app.add_simple_system(
            "spawn_ctx",
            crate::system::context_system(|context| {
                let entity = context.world_mut().spawn_entity();
                let data = Bytes::from_array(context.env(), &[0xAB]);
                context
                    .world_mut()
                    .add_component(entity, symbol_short!("simp"), data);
            }),
        );
        app.configure_system(
            "spawn_ctx",
            SystemConfig::new().in_stage(ScheduleStage::Cleanup),
        )
        .unwrap();

        app.run(&env).unwrap();

        assert!(app.world().has_component(1, &symbol_short!("simp")));
    }
}
