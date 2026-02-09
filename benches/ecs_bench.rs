//! Simple performance benchmarks for Cougr ECS operations.
//!
//! Uses `std::time::Instant` since `criterion` is not compatible with `#![no_std]`.
//! Run with: `cargo bench`

use std::time::Instant;

use cougr_core::archetype_world::ArchetypeWorld;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::{symbol_short, Bytes, Env};

fn bench_spawn_entities() {
    println!("\n--- Spawn Entities ---");
    for count in [100, 500] {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let start = Instant::now();
        for _ in 0..count {
            world.spawn_entity();
        }
        let elapsed = start.elapsed();
        println!(
            "  Spawn {count} entities: {elapsed:?} ({:.1} us/entity)",
            elapsed.as_micros() as f64 / count as f64
        );
    }
}

fn bench_add_components() {
    println!("\n--- Add Components ---");
    for count in [50, 200] {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entities: Vec<u32> = (0..count).map(|_| world.spawn_entity()).collect();

        let name = symbol_short!("pos");
        let data = Bytes::from_slice(&env, &[0u8; 16]);

        let start = Instant::now();
        for &eid in &entities {
            world.add_component(eid, name.clone(), data.clone());
        }
        let elapsed = start.elapsed();
        println!(
            "  Add component to {count} entities: {elapsed:?} ({:.1} us/entity)",
            elapsed.as_micros() as f64 / count as f64
        );
    }
}

fn bench_get_components() {
    println!("\n--- Get Components ---");
    for count in [50, 200] {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entities: Vec<u32> = (0..count).map(|_| world.spawn_entity()).collect();

        let name = symbol_short!("pos");
        let data = Bytes::from_slice(&env, &[0u8; 16]);
        for &eid in &entities {
            world.add_component(eid, name.clone(), data.clone());
        }

        let start = Instant::now();
        for &eid in &entities {
            let _ = world.get_component(eid, &name);
        }
        let elapsed = start.elapsed();
        println!(
            "  Get component from {count} entities: {elapsed:?} ({:.1} us/entity)",
            elapsed.as_micros() as f64 / count as f64
        );
    }
}

fn bench_remove_components() {
    println!("\n--- Remove Components ---");
    for count in [50, 200] {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entities: Vec<u32> = (0..count).map(|_| world.spawn_entity()).collect();

        let name = symbol_short!("pos");
        let data = Bytes::from_slice(&env, &[0u8; 16]);
        for &eid in &entities {
            world.add_component(eid, name.clone(), data.clone());
        }

        let start = Instant::now();
        for &eid in &entities {
            world.remove_component(eid, &name);
        }
        let elapsed = start.elapsed();
        println!(
            "  Remove component from {count} entities: {elapsed:?} ({:.1} us/entity)",
            elapsed.as_micros() as f64 / count as f64
        );
    }
}

fn bench_archetype_world() {
    println!("\n--- ArchetypeWorld ---");
    for count in [50, 200] {
        let env = Env::default();
        let mut world = ArchetypeWorld::new(&env);

        let name = symbol_short!("pos");
        let data = Bytes::from_slice(&env, &[0u8; 16]);

        let start = Instant::now();
        for _ in 0..count {
            let eid = world.spawn_entity();
            world.add_component(eid, name.clone(), data.clone(), &env);
        }
        let elapsed = start.elapsed();
        println!(
            "  Spawn+add to {count} entities: {elapsed:?} ({:.1} us/entity)",
            elapsed.as_micros() as f64 / count as f64
        );

        let start = Instant::now();
        for eid in 1..=count as u32 {
            let _ = world.get_component(eid, &name);
        }
        let elapsed = start.elapsed();
        println!(
            "  Get component from {count} entities: {elapsed:?} ({:.1} us/entity)",
            elapsed.as_micros() as f64 / count as f64
        );
    }
}

fn main() {
    println!("=== Cougr ECS Benchmarks ===");
    bench_spawn_entities();
    bench_add_components();
    bench_get_components();
    bench_remove_components();
    bench_archetype_world();
    println!("\n=== Done ===");
}
