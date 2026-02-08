//! Simulation result types.

use pyo3::prelude::*;
use std::collections::HashMap;

/// Lightweight step result for charting (minimal memory footprint).
#[pyclass]
#[derive(Debug, Clone)]
pub struct LightweightStepResult {
    /// Simulation step number
    #[pyo3(get)]
    pub timestamp: u32,

    /// Fair price at this step
    #[pyo3(get)]
    pub fair_price: f64,

    /// Spot prices by strategy name
    #[pyo3(get)]
    pub spot_prices: HashMap<String, f64>,

    /// Running PnL by strategy name
    #[pyo3(get)]
    pub pnls: HashMap<String, f64>,

    /// Fees (bid, ask) by strategy name
    #[pyo3(get)]
    pub fees: HashMap<String, (f64, f64)>,
}

#[pymethods]
impl LightweightStepResult {
    fn __repr__(&self) -> String {
        format!(
            "LightweightStepResult(timestamp={}, fair_price={:.4})",
            self.timestamp, self.fair_price
        )
    }
}

/// Lightweight simulation result for charting.
#[pyclass]
#[derive(Debug, Clone)]
pub struct LightweightSimResult {
    /// Seed used for this simulation
    #[pyo3(get)]
    pub seed: u64,

    /// Strategy names
    #[pyo3(get)]
    pub strategies: Vec<String>,

    /// Final PnL by strategy name
    #[pyo3(get)]
    pub pnl: HashMap<String, f64>,

    /// Edge by strategy name (sum over trades)
    #[pyo3(get)]
    pub edges: HashMap<String, f64>,

    /// Initial fair price
    #[pyo3(get)]
    pub initial_fair_price: f64,

    /// Initial reserves by strategy name: (reserve_x, reserve_y)
    #[pyo3(get)]
    pub initial_reserves: HashMap<String, (f64, f64)>,

    /// Step results for charting
    #[pyo3(get)]
    pub steps: Vec<LightweightStepResult>,

    /// Total arb volume (in Y) by strategy name
    #[pyo3(get)]
    pub arb_volume_y: HashMap<String, f64>,

    /// Total retail volume (in Y) by strategy name
    #[pyo3(get)]
    pub retail_volume_y: HashMap<String, f64>,

    /// Average fees (bid, ask) by strategy name over the simulation
    #[pyo3(get)]
    pub average_fees: HashMap<String, (f64, f64)>,
}

#[pymethods]
impl LightweightSimResult {
    /// Get the winner of this simulation.
    fn winner(&self) -> Option<String> {
        let names: Vec<_> = self.strategies.iter().collect();
        if names.len() != 2 {
            return None;
        }

        let pnl_a = self.pnl.get(names[0]).copied().unwrap_or(0.0);
        let pnl_b = self.pnl.get(names[1]).copied().unwrap_or(0.0);
        let edge_a = self.edges.get(names[0]).copied().unwrap_or(pnl_a);
        let edge_b = self.edges.get(names[1]).copied().unwrap_or(pnl_b);

        if edge_a > edge_b {
            Some(names[0].clone())
        } else if edge_b > edge_a {
            Some(names[1].clone())
        } else {
            None // Draw
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "LightweightSimResult(seed={}, pnl={:?})",
            self.seed, self.pnl
        )
    }
}

/// Batch result containing all simulation results.
#[pyclass]
#[derive(Debug, Clone)]
pub struct BatchSimulationResult {
    /// Individual simulation results
    #[pyo3(get)]
    pub results: Vec<LightweightSimResult>,

    /// Strategy names
    #[pyo3(get)]
    pub strategies: Vec<String>,
}

#[pymethods]
impl BatchSimulationResult {
    /// Get win counts: (wins_a, wins_b, draws)
    fn win_counts(&self) -> (u32, u32, u32) {
        if self.strategies.len() != 2 {
            return (0, 0, 0);
        }

        let name_a = &self.strategies[0];
        let name_b = &self.strategies[1];

        let mut wins_a = 0u32;
        let mut wins_b = 0u32;
        let mut draws = 0u32;

        for result in &self.results {
            let pnl_a = result.pnl.get(name_a).copied().unwrap_or(0.0);
            let pnl_b = result.pnl.get(name_b).copied().unwrap_or(0.0);
            let edge_a = result.edges.get(name_a).copied().unwrap_or(pnl_a);
            let edge_b = result.edges.get(name_b).copied().unwrap_or(pnl_b);

            if edge_a > edge_b {
                wins_a += 1;
            } else if edge_b > edge_a {
                wins_b += 1;
            } else {
                draws += 1;
            }
        }

        (wins_a, wins_b, draws)
    }

    /// Get total PnL: (total_pnl_a, total_pnl_b)
    fn total_pnl(&self) -> (f64, f64) {
        if self.strategies.len() != 2 {
            return (0.0, 0.0);
        }

        let name_a = &self.strategies[0];
        let name_b = &self.strategies[1];

        let mut total_a = 0.0f64;
        let mut total_b = 0.0f64;

        for result in &self.results {
            total_a += result.pnl.get(name_a).copied().unwrap_or(0.0);
            total_b += result.pnl.get(name_b).copied().unwrap_or(0.0);
        }

        (total_a, total_b)
    }

    /// Get the overall winner based on win count.
    fn overall_winner(&self) -> Option<String> {
        let (wins_a, wins_b, _) = self.win_counts();
        if wins_a > wins_b {
            Some(self.strategies[0].clone())
        } else if wins_b > wins_a {
            Some(self.strategies[1].clone())
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        let (wins_a, wins_b, draws) = self.win_counts();
        format!(
            "BatchSimulationResult(n={}, wins=({}, {}, {}))",
            self.results.len(),
            wins_a,
            wins_b,
            draws
        )
    }

    fn __len__(&self) -> usize {
        self.results.len()
    }
}

/// Per-pool state snapshot at end of simulation (V2).
#[pyclass]
#[derive(Debug, Clone)]
pub struct PoolStateV2 {
    #[pyo3(get)]
    pub pool_id: usize,
    #[pyo3(get)]
    pub token_a: usize,
    #[pyo3(get)]
    pub token_b: usize,
    #[pyo3(get)]
    pub reserve_a: f64,
    #[pyo3(get)]
    pub reserve_b: f64,
}

#[pymethods]
impl PoolStateV2 {
    fn __repr__(&self) -> String {
        format!(
            "PoolStateV2(id={}, pair=({},{}) reserves=({},{}))",
            self.pool_id, self.token_a, self.token_b, self.reserve_a, self.reserve_b
        )
    }
}

/// Lightweight simulation result for multi-asset mode.
#[pyclass]
#[derive(Debug, Clone)]
pub struct LightweightSimResultV2 {
    /// Seed used for this simulation
    #[pyo3(get)]
    pub seed: u64,

    /// Strategy names
    #[pyo3(get)]
    pub strategies: Vec<String>,

    /// Final PnL in numeraire by strategy name
    #[pyo3(get)]
    pub pnl: HashMap<String, f64>,

    /// Edge in numeraire by strategy name
    #[pyo3(get)]
    pub edges: HashMap<String, f64>,

    /// Final token fair prices in numeraire terms
    #[pyo3(get)]
    pub final_prices: Vec<f64>,

    /// Final reserves by pool id
    #[pyo3(get)]
    pub pools: Vec<PoolStateV2>,
}

#[pymethods]
impl LightweightSimResultV2 {
    fn __repr__(&self) -> String {
        format!(
            "LightweightSimResultV2(seed={}, n_pools={})",
            self.seed,
            self.pools.len()
        )
    }
}

/// Batch result for multi-asset simulations.
#[pyclass]
#[derive(Debug, Clone)]
pub struct BatchSimulationResultV2 {
    /// Individual simulation results
    #[pyo3(get)]
    pub results: Vec<LightweightSimResultV2>,

    /// Strategy names
    #[pyo3(get)]
    pub strategies: Vec<String>,
}

#[pymethods]
impl BatchSimulationResultV2 {
    fn __len__(&self) -> usize {
        self.results.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "BatchSimulationResultV2(n={}, strategies={:?})",
            self.results.len(),
            self.strategies
        )
    }
}
