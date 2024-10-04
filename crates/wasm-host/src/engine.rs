use anyhow::Result;
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};
use tracing::{debug, info};
use wasmtime::{Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig};

// global engine hashmap with string key with sync mutex
lazy_static! {
    pub static ref ENGINE_MAP: Mutex<HashMap<String, Engine>> = Mutex::new(HashMap::new());
}

/// MODULE_VERSION is the module version
/// w25 means wasmtime 25
pub const MODULE_VERSION: &str = "w25";

// 10 ms to trigger epoch increment
pub const EPOCH_INC_INTERVAL: u64 = 10;

/// init_engines initialize default engine
pub fn init_engines() -> Result<()> {
    // try use std to run this loop. not tokio
    std::thread::spawn(|| {
        epoch_deadline_callback();
    });
    let _ = get("default")?;
    Ok(())
}

/// epoch_deadline_callback
fn epoch_deadline_callback() {
    loop {
        // if ENGINE_MAP is empty, sleep 3 seconds to wait for new engine
        if ENGINE_MAP.lock().unwrap().is_empty() {
            debug!("ENGINE_MAP is empty, sleep 3 seconds to wait for new engine");
            std::thread::sleep(std::time::Duration::from_secs(3));
            continue;
        }

        // iterate ENGINE_MAP to increment epoch for every EPOCH_INC_INTERVAL ms
        for (_, engine) in ENGINE_MAP.lock().unwrap().iter() {
            engine.increment_epoch();
        }
        // use std thread to sleep 3 seconds
        std::thread::sleep(std::time::Duration::from_millis(EPOCH_INC_INTERVAL));
    }
}

/// get engine by key
pub fn get(key: &str) -> Result<Engine> {
    let mut map = ENGINE_MAP.lock().unwrap();
    if map.contains_key(key) {
        return Ok(map.get(key).unwrap().clone());
    }
    info!("Create new engine for key: {}", key);
    let config = create_config()?;
    let engine = Engine::new(&config).unwrap();
    map.insert(key.to_string(), engine.clone());
    Ok(engine)
}

fn create_config() -> Result<Config> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    config.epoch_interruption(true);
    config.module_version(wasmtime::ModuleVersionStrategy::Custom(
        MODULE_VERSION.to_string(),
    ))?;

    const MB: usize = 1 << 20;
    let mut pooling_allocation_config = PoolingAllocationConfig::default();

    // This number matches Compute production
    pooling_allocation_config.max_core_instance_size(MB);

    // Core wasm programs have 1 memory
    pooling_allocation_config.total_memories(100);
    pooling_allocation_config.max_memories_per_module(1);

    // allow for up to 128MiB of linear memory. Wasm pages are 64k
    pooling_allocation_config.max_memory_size(128 * MB);

    // Core wasm programs have 1 table
    pooling_allocation_config.max_tables_per_module(1);

    // Some applications create a large number of functions, in particular
    // when compiled in debug mode or applications written in swift. Every
    // function can end up in the table
    pooling_allocation_config.table_elements(98765);

    // Maximum number of slots in the pooling allocator to keep "warm", or those
    // to keep around to possibly satisfy an affine allocation request or an
    // instantiation of a module previously instantiated within the pool.
    pooling_allocation_config.max_unused_warm_slots(10);

    // Use a large pool, but one smaller than the default of 1000 to avoid runnign out of virtual
    // memory space if multiple engines are spun up in a single process. We'll likely want to move
    // to the on-demand allocator eventually for most purposes; see
    // https://github.com/fastly/Viceroy/issues/255
    pooling_allocation_config.total_core_instances(100);

    config.allocation_strategy(InstanceAllocationStrategy::Pooling(
        pooling_allocation_config,
    ));

    debug!("Create new config: {:?}", config);

    Ok(config)
}
