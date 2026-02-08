//! Multi-asset simulation engine (N assets, many 2-token pools).

use std::collections::HashMap;

use crate::amm::CFMM;
use crate::evm::EVMStrategy;
use crate::market::{Arbitrageur, MultiAssetPriceProcess, RetailTraderV2};
use crate::simulation::engine::SimulationError;
use crate::types::config::SimulationConfigV2;
use crate::types::result::{LightweightSimResultV2, PoolStateV2};

/// Main simulation engine for multi-asset competition mode.
pub struct SimulationEngineV2 {
    config: SimulationConfigV2,
}

impl SimulationEngineV2 {
    pub fn new(config: SimulationConfigV2) -> Self {
        Self { config }
    }

    pub fn run(
        &mut self,
        submission_bytecode: &[u8],
        baseline_bytecode: &[u8],
    ) -> Result<LightweightSimResultV2, SimulationError> {
        let seed = self.config.seed.unwrap_or(0);
        let n_assets = self.config.initial_prices.len();

        if n_assets < 2 {
            return Err(SimulationError::InvalidConfig(
                "SimulationConfigV2 requires at least 2 assets".to_string(),
            ));
        }
        if self.config.numeraire_token >= n_assets {
            return Err(SimulationError::InvalidConfig(
                "numeraire_token out of bounds".to_string(),
            ));
        }
        if self.config.pools.is_empty() {
            return Err(SimulationError::InvalidConfig(
                "SimulationConfigV2 requires at least 1 pool".to_string(),
            ));
        }

        let mut price_process = MultiAssetPriceProcess::new(
            self.config.initial_prices.clone(),
            self.config.numeraire_token,
            self.config.gbm_mu,
            self.config.gbm_sigma,
            self.config.gbm_dt,
            Some(seed),
        );
        let mut retail_trader = RetailTraderV2::new(
            n_assets,
            self.config.retail_arrival_rate,
            self.config.retail_mean_size,
            self.config.retail_size_sigma,
            self.config.retail_buy_prob,
            Some(seed + 1),
        );

        let mut amms: Vec<CFMM> = Vec::with_capacity(self.config.pools.len() * 2);
        for (pool_idx, pool) in self.config.pools.iter().enumerate() {
            let (token_a, token_b, initial_a, initial_b) = *pool;
            if token_a == token_b || token_a >= n_assets || token_b >= n_assets {
                return Err(SimulationError::InvalidConfig(format!(
                    "Invalid pool at index {}: token indices must be distinct and in range",
                    pool_idx
                )));
            }
            if initial_a <= 0.0 || initial_b <= 0.0 {
                return Err(SimulationError::InvalidConfig(format!(
                    "Invalid pool at index {}: reserves must be > 0",
                    pool_idx
                )));
            }

            let submission =
                EVMStrategy::new(submission_bytecode.to_vec(), "submission".to_string())
                    .map_err(|e| SimulationError::EVMError(e.to_string()))?;
            let baseline = EVMStrategy::new(baseline_bytecode.to_vec(), "normalizer".to_string())
                .map_err(|e| SimulationError::EVMError(e.to_string()))?;

            let mut amm_submission = CFMM::new_with_pair(
                submission,
                initial_a,
                initial_b,
                token_a,
                token_b,
                pool_idx * 2,
            );
            amm_submission.name = "submission".to_string();
            amm_submission
                .initialize_v2_or_fallback()
                .map_err(|e| SimulationError::EVMError(e.to_string()))?;

            let mut amm_baseline = CFMM::new_with_pair(
                baseline,
                initial_a,
                initial_b,
                token_a,
                token_b,
                pool_idx * 2 + 1,
            );
            amm_baseline.name = "normalizer".to_string();
            amm_baseline
                .initialize_v2_or_fallback()
                .map_err(|e| SimulationError::EVMError(e.to_string()))?;

            amms.push(amm_submission);
            amms.push(amm_baseline);
        }

        let mut initial_value = HashMap::from([
            ("submission".to_string(), 0.0_f64),
            ("normalizer".to_string(), 0.0_f64),
        ]);
        for amm in &amms {
            let (rx, ry) = amm.reserves();
            let value = rx * price_process.current_prices()[amm.token_a]
                + ry * price_process.current_prices()[amm.token_b];
            *initial_value.get_mut(&amm.name).unwrap() += value;
        }

        let mut edges = HashMap::from([
            ("submission".to_string(), 0.0_f64),
            ("normalizer".to_string(), 0.0_f64),
        ]);

        let arbitrageur = Arbitrageur::new();

        for t in 0..self.config.n_steps {
            let prices = price_process.step();

            // 2) Arbitrage each pool to current fair cross-rate.
            for amm in amms.iter_mut() {
                let fair_price = prices[amm.token_b] / prices[amm.token_a];
                if let Some(result) = arbitrageur.execute_arb(amm, fair_price, t as u64) {
                    // Generic edge in numeraire terms:
                    // AMM edge = value_in - value_out.
                    let edge = if result.side == "buy" {
                        // AMM buys token_a, pays token_b.
                        result.amount_x * prices[amm.token_a]
                            - result.amount_y * prices[amm.token_b]
                    } else {
                        // AMM sells token_a, receives token_b.
                        result.amount_y * prices[amm.token_b]
                            - result.amount_x * prices[amm.token_a]
                    };
                    *edges.get_mut(&amm.name).unwrap() += edge;
                }
            }

            // 3) Generate retail orders and route to best direct pool.
            let orders = retail_trader.generate_orders();
            for order in orders {
                let token_in = order.token_in;
                let token_out = order.token_out;
                let amount_in = (order.size_numeraire / prices[token_in].max(1e-9)).max(1e-12);

                let mut best_idx: Option<usize> = None;
                let mut best_out = 0.0_f64;
                for (idx, amm) in amms.iter().enumerate() {
                    if let Some((out, _fee)) = amm.quote_exact_in(token_in, token_out, amount_in) {
                        if out > best_out {
                            best_out = out;
                            best_idx = Some(idx);
                        }
                    }
                }

                if let Some(idx) = best_idx {
                    if let Some((amount_out, _is_buy)) =
                        amms[idx].execute_exact_in(token_in, token_out, amount_in, t as u64)
                    {
                        let edge = amount_in * prices[token_in] - amount_out * prices[token_out];
                        let name = amms[idx].name.clone();
                        *edges.get_mut(&name).unwrap() += edge;
                    }
                }
            }
        }

        let mut pnl = HashMap::from([
            ("submission".to_string(), 0.0_f64),
            ("normalizer".to_string(), 0.0_f64),
        ]);
        for amm in &amms {
            let (rx, ry) = amm.reserves();
            let (fx, fy) = amm.accumulated_fees();
            let prices = price_process.current_prices();
            let final_value = (rx + fx) * prices[amm.token_a] + (ry + fy) * prices[amm.token_b];
            *pnl.get_mut(&amm.name).unwrap() += final_value;
        }
        for (name, init) in &initial_value {
            if let Some(total) = pnl.get_mut(name) {
                *total -= init;
            }
        }

        let pools = amms
            .iter()
            .map(|amm| {
                let (rx, ry) = amm.reserves();
                PoolStateV2 {
                    pool_id: amm.pool_id,
                    token_a: amm.token_a,
                    token_b: amm.token_b,
                    reserve_a: rx,
                    reserve_b: ry,
                }
            })
            .collect();

        Ok(LightweightSimResultV2 {
            seed,
            strategies: vec!["submission".to_string(), "normalizer".to_string()],
            pnl,
            edges,
            final_prices: price_process.current_prices().to_vec(),
            pools,
        })
    }
}
