//! Parallel simulation runner using rayon.

use rayon::prelude::*;

use crate::evm::EVMStrategy;
use crate::simulation::engine::{SimulationEngine, SimulationError};
use crate::simulation::engine_v2::SimulationEngineV2;
use crate::types::config::{SimulationConfig, SimulationConfigV2};
use crate::types::result::{
    BatchSimulationResult, BatchSimulationResultV2, LightweightSimResult, LightweightSimResultV2,
};

/// Configuration for a batch of simulations.
pub struct SimulationBatchConfig {
    /// Bytecode for the submission strategy
    pub submission_bytecode: Vec<u8>,
    /// Bytecode for the baseline strategy
    pub baseline_bytecode: Vec<u8>,
    /// List of simulation configs (one per simulation)
    pub configs: Vec<SimulationConfig>,
    /// Number of parallel workers (None = auto-detect)
    pub n_workers: Option<usize>,
}

/// Configuration for a batch of multi-asset simulations.
pub struct SimulationBatchConfigV2 {
    /// Bytecode for the submission strategy
    pub submission_bytecode: Vec<u8>,
    /// Bytecode for the baseline strategy
    pub baseline_bytecode: Vec<u8>,
    /// List of simulation configs (one per simulation)
    pub configs: Vec<SimulationConfigV2>,
    /// Number of parallel workers (None = auto-detect)
    pub n_workers: Option<usize>,
}

/// Run multiple simulations in parallel.
pub fn run_simulations_parallel(
    batch_config: SimulationBatchConfig,
) -> Result<BatchSimulationResult, SimulationError> {
    // Configure thread pool
    let n_workers = batch_config
        .n_workers
        .unwrap_or_else(|| rayon::current_num_threads().min(8));

    // Build custom thread pool if needed
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(n_workers)
        .build()
        .map_err(|e| {
            SimulationError::InvalidConfig(format!("Failed to create thread pool: {}", e))
        })?;

    // Clone bytecodes for each worker (they need their own EVM instances)
    let submission_bytecode = batch_config.submission_bytecode;
    let baseline_bytecode = batch_config.baseline_bytecode;

    // Run simulations in parallel
    let results: Result<Vec<LightweightSimResult>, SimulationError> = pool.install(|| {
        batch_config
            .configs
            .into_par_iter()
            .map(|config| {
                // Create fresh EVM strategies for this worker
                let submission =
                    EVMStrategy::new(submission_bytecode.clone(), "Submission".to_string())
                        .map_err(|e| SimulationError::EVMError(e.to_string()))?;

                let baseline = EVMStrategy::new(baseline_bytecode.clone(), "Baseline".to_string())
                    .map_err(|e| SimulationError::EVMError(e.to_string()))?;

                let mut engine = SimulationEngine::new(config);
                engine.run(submission, baseline)
            })
            .collect()
    });

    let results = results?;

    // Extract strategy names from first result
    let strategies = if let Some(first) = results.first() {
        first.strategies.clone()
    } else {
        Vec::new()
    };

    Ok(BatchSimulationResult {
        results,
        strategies,
    })
}

/// Run multiple multi-asset simulations in parallel.
pub fn run_simulations_parallel_v2(
    batch_config: SimulationBatchConfigV2,
) -> Result<BatchSimulationResultV2, SimulationError> {
    let n_workers = batch_config
        .n_workers
        .unwrap_or_else(|| rayon::current_num_threads().min(8));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(n_workers)
        .build()
        .map_err(|e| {
            SimulationError::InvalidConfig(format!("Failed to create thread pool: {}", e))
        })?;

    let submission_bytecode = batch_config.submission_bytecode;
    let baseline_bytecode = batch_config.baseline_bytecode;

    let results: Result<Vec<LightweightSimResultV2>, SimulationError> = pool.install(|| {
        batch_config
            .configs
            .into_par_iter()
            .map(|config| {
                let mut engine = SimulationEngineV2::new(config);
                engine.run(&submission_bytecode, &baseline_bytecode)
            })
            .collect()
    });

    let results = results?;
    let strategies = if let Some(first) = results.first() {
        first.strategies.clone()
    } else {
        vec!["submission".to_string(), "normalizer".to_string()]
    };

    Ok(BatchSimulationResultV2 {
        results,
        strategies,
    })
}

/// Run a single simulation (non-parallel).
pub fn run_simulation(
    submission_bytecode: Vec<u8>,
    baseline_bytecode: Vec<u8>,
    config: SimulationConfig,
) -> Result<LightweightSimResult, SimulationError> {
    let submission = EVMStrategy::new(submission_bytecode, "Submission".to_string())
        .map_err(|e| SimulationError::EVMError(e.to_string()))?;

    let baseline = EVMStrategy::new(baseline_bytecode, "Baseline".to_string())
        .map_err(|e| SimulationError::EVMError(e.to_string()))?;

    let mut engine = SimulationEngine::new(config);
    engine.run(submission, baseline)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Full tests require EVM bytecode - see integration tests
}
