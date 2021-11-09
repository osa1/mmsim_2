use std::cmp::{max, min};
use std::fs::File;
use std::io::Write;

use clap::{App, Arg};

#[derive(Debug, Clone, Copy)]
enum GcStrategy {
    MarkCompact,
    Copying,
}

#[derive(Debug, Clone, Copy)]
enum Scheduler {
    Old,
    New,
}

#[derive(Debug, Clone)]
struct RuntimeConfig {
    gc_strategy: GcStrategy,
    scheduler: Scheduler,

    num_calls: u32,

    // bytes/call
    allocation_rate: u32,

    // Percentage of newly allocated objects surviving a call
    survival_rate: u32,

    growth_factor: f64,
    small_heap_delta: u64,
    max_hp_for_gc: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            gc_strategy: GcStrategy::MarkCompact,
            scheduler: Scheduler::New,
            num_calls: 100_000,
            allocation_rate: 100_000,
            survival_rate: 50,
            growth_factor: 1.5f64,
            small_heap_delta: 10 * 1024 * 1024,    // 10 MiB
            max_hp_for_gc: 2 * 1024 * 1024 * 1024, // 1 GiB
        }
    }
}

fn main() {
    let matches = App::new("mmsim")
        .arg(
            Arg::with_name("NUM_CALLS")
                .long("num-calls")
                .value_name("NUM_CALLS")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("ALLOCATION_RATE")
                .long("allocation-rate")
                .value_name("ALLOCATION_RATE")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("SURVIVAL_RATE")
                .long("survival-rate")
                .value_name("SURVIVAL_RATE")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("GROWTH_FACTOR")
                .long("growth-factor")
                .value_name("GROTH_FACTOR")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("SMALL_HEAP_DELTA")
                .long("small-heap-delta")
                .value_name("SMALL_HEAP_DELTA")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("MAX_HP_FOR_GC")
                .long("max-hp-for-gc")
                .value_name("MAX_HP_FOR_GC")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("GC_STRATEGY")
                .long("gc-strategy")
                .value_name("GC_STRATEGY")
                .takes_value(true)
                .possible_values(&["copying", "mark-compact"])
                .required(false),
        )
        .arg(
            Arg::with_name("SCHEDULER")
                .long("scheduler")
                .value_name("SCHEDULER")
                .takes_value(true)
                .possible_values(&["old", "new"])
                .required(false),
        )
        .get_matches();

    let mut runtime_config = RuntimeConfig::default();

    match matches.value_of("GC_STRATEGY") {
        Some("copying") => runtime_config.gc_strategy = GcStrategy::Copying,
        _ => runtime_config.gc_strategy = GcStrategy::MarkCompact,
    }

    match matches.value_of("SCHEDULER") {
        Some("old") => runtime_config.scheduler = Scheduler::Old,
        _ => runtime_config.scheduler = Scheduler::New,
    }

    if let Some(num_calls) = matches.value_of("NUM_CALLS") {
        match str::parse::<u32>(num_calls) {
            Ok(num_calls) => runtime_config.num_calls = num_calls,
            Err(err) => panic!(
                "NUM_CALLS argument must be a 32-bit unsigned integer: {:?}",
                err
            ),
        }
    }

    if let Some(allocation_rate) = matches.value_of("ALLOCATION_RATE") {
        match str::parse::<u32>(allocation_rate) {
            Ok(allocation_rate) => runtime_config.allocation_rate = allocation_rate,
            Err(err) => panic!(
                "ALLOCATION_RATE argument must be a 32-bit unsigned integer: {:?}",
                err
            ),
        }
    }

    if let Some(survival_rate) = matches.value_of("SURVIVAL_RATE") {
        match str::parse::<u32>(survival_rate) {
            Ok(survival_rate) => runtime_config.survival_rate = survival_rate,
            Err(err) => panic!(
                "SURVIVAL_RATE argument must be a 32-bit unsigned integer: {:?}",
                err
            ),
        }
    }

    if let Some(growth_factor) = matches.value_of("GROWTH_FACTOR") {
        match str::parse::<f64>(growth_factor) {
            Ok(growth_factor) => runtime_config.growth_factor = growth_factor,
            Err(err) => panic!("GROWTH_FACTOR argument must be a 64-bit float: {:?}", err),
        }
    }

    if let Some(small_heap_delta) = matches.value_of("SMALL_HEAP_DELTA") {
        match str::parse::<u64>(small_heap_delta) {
            Ok(small_heap_delta) => runtime_config.small_heap_delta = small_heap_delta,
            Err(err) => panic!(
                "SMALL_HEAP_DELTA argument must be a 64-bit unsigned integer: {:?}",
                err
            ),
        }
    }

    if let Some(max_hp_for_gc) = matches.value_of("MAX_HP_FOR_GC") {
        match str::parse::<u64>(max_hp_for_gc) {
            Ok(max_hp_for_gc) => runtime_config.max_hp_for_gc = max_hp_for_gc,
            Err(err) => panic!(
                "MAX_HP_FOR_GC argument must be a 64-bit unsigned integer: {:?}",
                err
            ),
        }
    }

    println!("{:#?}", runtime_config);

    let Points { hp, high_water } = generate_points(runtime_config);

    // Generate hp.csv
    {
        let mut file = File::create("hp.csv").unwrap();
        writeln!(&mut file, "hp").unwrap();

        for point in hp {
            writeln!(&mut file, "{}", point).unwrap();
        }
    }

    // Generate total_alloc.csv
    {
        let mut file = File::create("high_water.csv").unwrap();
        writeln!(&mut file, "high water").unwrap();

        for point in high_water {
            writeln!(&mut file, "{}", point).unwrap();
        }
    }
}

#[derive(Debug)]
struct Points {
    hp: Vec<u32>,
    // total_alloc: Vec<u32>,
    high_water: Vec<u32>,
}

fn generate_points(config: RuntimeConfig) -> Points {
    let RuntimeConfig {
        gc_strategy,
        scheduler,
        num_calls,
        allocation_rate,
        survival_rate,
        growth_factor,
        small_heap_delta,
        max_hp_for_gc,
    } = config;

    let mut hp: Vec<u32> = Vec::with_capacity(config.num_calls as usize);
    hp.push(0);

    let mut high_water: Vec<u32> = Vec::with_capacity(config.num_calls as usize);
    high_water.push(0);

    // Heap pointer after last gc
    let mut last_hp: u32 = 0;

    // Current heap pointer
    let mut hp_: u32 = 0;

    // High water mark for Wasm memory
    // NB. This is in bytes, not rounded up to Wasm page size
    let mut last_high_water: u32 = 0;

    // Number of gcs
    let mut num_gcs: u32 = 0;

    // Number of calls made so far
    let mut n_calls = 0;

    for _ in 0..num_calls {
        n_calls += 1;

        const COPYING_GC_MAX_LIVE: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB

        // Mark stack ignored. Max. bitmap can be 130,150,524 bytes.
        // (x + x / 32 = 4 GiB, x = 4,164,816,771, x/32 = 130,150,524)
        const MARK_COMPACT_GC_MAX_LIVE: u64 = 4_164_816_771;
        const MARK_COMPACT_GC_MAX_BITMAP_SIZE: u32 = 130_150_524;

        let heap_limit = match scheduler {
            Scheduler::Old => min(
                max(
                    (f64::from(last_hp) * growth_factor) as u64,
                    u64::from(last_hp) + small_heap_delta,
                ),
                max_hp_for_gc,
            ),
            Scheduler::New => {
                let max_live = match gc_strategy {
                    GcStrategy::MarkCompact => MARK_COMPACT_GC_MAX_LIVE,
                    GcStrategy::Copying => COPYING_GC_MAX_LIVE,
                };
                min(
                    (f64::from(last_hp) * growth_factor) as u64,
                    (u64::from(last_hp) + max_live) / 2,
                )
            }
        };

        hp_ += allocation_rate;

        if u64::from(hp_) >= heap_limit {
            num_gcs += 1;

            // New allocations since last GC
            let new_allocs = hp_ - last_hp;

            // Live data since last GC
            let new_live = (f64::from(new_allocs) * f64::from(survival_rate) / 100f64) as u32;

            // Do GC
            match gc_strategy {
                GcStrategy::MarkCompact => {
                    // Mark-compact GC only allocates a bitmap. Mark stack size is ignored.
                    match hp_.checked_add(MARK_COMPACT_GC_MAX_BITMAP_SIZE) {
                        Some(high_water) => last_high_water = max(last_high_water, high_water),
                        None => break,
                    }
                }
                GcStrategy::Copying => {
                    // Copying GC copies the entire live heap to another space and then back
                    let copied = last_hp + new_live;
                    match hp_.checked_add(copied) {
                        Some(high_water) => last_high_water = max(last_high_water, high_water),
                        None => break,
                    }
                }
            }

            hp_ = last_hp + new_live;
            high_water.push(last_high_water);
            hp.push(hp_);
            last_hp = hp_;

            println!("GC=YES, hp={}, high water={}", hp_, last_high_water);
        } else {
            // No GC
            last_high_water = max(last_high_water, hp_);
            high_water.push(last_high_water);
            hp.push(hp_);

            println!("GC=NO, hp={}, high water={}", hp_, last_high_water);
        }

        n_calls += 1;
    }

    println!("GCs={}, total_calls={}", num_gcs, n_calls);

    Points { hp, high_water }
}
