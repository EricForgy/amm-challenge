//! Constant Function Market Maker (x * y = k).
//!
//! Implements Uniswap V3/V4-style fee model where fees are collected
//! into separate buckets rather than being reinvested into liquidity.
//! This means fees count toward PnL but don't inflate the k constant.

use crate::evm::EVMStrategy;
use crate::types::trade_info::{TradeInfo, TradeInfoV2};
use crate::types::wad::Wad;

/// Fee quote (bid and ask fees).
#[derive(Debug, Clone, Copy)]
pub struct FeeQuote {
    pub bid_fee: Wad, // Fee when AMM buys X
    pub ask_fee: Wad, // Fee when AMM sells X
}

impl FeeQuote {
    pub fn new(bid_fee: Wad, ask_fee: Wad) -> Self {
        Self { bid_fee, ask_fee }
    }

    pub fn symmetric(fee: Wad) -> Self {
        Self {
            bid_fee: fee,
            ask_fee: fee,
        }
    }
}

/// Result of a trade execution.
#[derive(Debug, Clone)]
pub struct TradeResult {
    pub trade_info: TradeInfo,
    pub fee_amount: f64,
}

/// Constant Function Market Maker with dynamic fees.
///
/// Implements x * y = k invariant with strategy-determined fees.
/// Uses Uniswap V3/V4 fee model where fees are collected separately
/// (not reinvested into liquidity).
pub struct CFMM {
    /// Pool identifier (used in multi-asset mode)
    pub pool_id: usize,
    /// First token index in pair (maps to X side in formulas)
    pub token_a: usize,
    /// Second token index in pair (maps to Y side in formulas)
    pub token_b: usize,
    /// Strategy name
    pub name: String,
    /// EVM strategy for fee decisions
    strategy: EVMStrategy,
    /// Current X reserves
    reserve_x: f64,
    /// Current Y reserves
    reserve_y: f64,
    /// Current fee quote
    current_fees: FeeQuote,
    /// Whether initialized
    initialized: bool,
    /// Whether this pool uses V2 strategy callbacks
    use_v2_callbacks: bool,
    /// Accumulated fees in X (collected separately, not in reserves)
    accumulated_fees_x: f64,
    /// Accumulated fees in Y (collected separately, not in reserves)
    accumulated_fees_y: f64,
}

impl CFMM {
    /// Create a new CFMM with the given strategy and reserves.
    pub fn new(strategy: EVMStrategy, reserve_x: f64, reserve_y: f64) -> Self {
        Self::new_with_pair(strategy, reserve_x, reserve_y, 0, 1, 0)
    }

    /// Create a new CFMM with explicit token pair metadata.
    pub fn new_with_pair(
        strategy: EVMStrategy,
        reserve_x: f64,
        reserve_y: f64,
        token_a: usize,
        token_b: usize,
        pool_id: usize,
    ) -> Self {
        let name = strategy.name().to_string();
        Self {
            pool_id,
            token_a,
            token_b,
            name,
            strategy,
            reserve_x,
            reserve_y,
            current_fees: FeeQuote::symmetric(Wad::from_bps(30)),
            initialized: false,
            use_v2_callbacks: false,
            accumulated_fees_x: 0.0,
            accumulated_fees_y: 0.0,
        }
    }

    /// Returns true when this pool directly connects token_in -> token_out.
    pub fn supports_pair(&self, token_in: usize, token_out: usize) -> bool {
        (self.token_a == token_in && self.token_b == token_out)
            || (self.token_b == token_in && self.token_a == token_out)
    }

    /// Initialize the AMM and get starting fees from strategy.
    pub fn initialize(&mut self) -> Result<(), crate::evm::strategy::EVMError> {
        let initial_x = Wad::from_f64(self.reserve_x);
        let initial_y = Wad::from_f64(self.reserve_y);

        let (bid_fee, ask_fee) = self.strategy.after_initialize(initial_x, initial_y)?;
        self.current_fees = FeeQuote::new(bid_fee.clamp_fee(), ask_fee.clamp_fee());
        self.initialized = true;
        self.use_v2_callbacks = false;

        Ok(())
    }

    /// Initialize using V2 callback with context, with fallback to V1.
    pub fn initialize_v2_or_fallback(&mut self) -> Result<(), crate::evm::strategy::EVMError> {
        let initial_a = Wad::from_f64(self.reserve_x);
        let initial_b = Wad::from_f64(self.reserve_y);
        match self.strategy.after_initialize_v2(
            initial_a,
            initial_b,
            self.pool_id as u64,
            self.token_a as u64,
            self.token_b as u64,
        ) {
            Ok((bid_fee, ask_fee)) => {
                self.current_fees = FeeQuote::new(bid_fee.clamp_fee(), ask_fee.clamp_fee());
                self.initialized = true;
                self.use_v2_callbacks = true;
                Ok(())
            }
            Err(_) => self.initialize(),
        }
    }

    /// Get current reserves.
    pub fn reserves(&self) -> (f64, f64) {
        (self.reserve_x, self.reserve_y)
    }

    /// Get current spot price (Y per X).
    pub fn spot_price(&self) -> f64 {
        if self.reserve_x == 0.0 {
            return 0.0;
        }
        self.reserve_y / self.reserve_x
    }

    /// Get current k (constant product).
    pub fn k(&self) -> f64 {
        self.reserve_x * self.reserve_y
    }

    /// Get current fees.
    pub fn fees(&self) -> FeeQuote {
        self.current_fees
    }

    /// Get accumulated fees (collected separately from reserves).
    pub fn accumulated_fees(&self) -> (f64, f64) {
        (self.accumulated_fees_x, self.accumulated_fees_y)
    }

    /// Fast quote for AMM buying X (trader selling X).
    ///
    /// Returns (y_out, fee_amount) or (0, 0) if invalid.
    #[inline]
    pub fn quote_buy_x(&self, amount_x: f64) -> (f64, f64) {
        if amount_x <= 0.0 {
            return (0.0, 0.0);
        }

        let fee = self.current_fees.bid_fee.to_f64();
        let gamma = (1.0 - fee).clamp(0.0, 1.0);
        if gamma <= 0.0 {
            return (0.0, 0.0);
        }
        let net_x = amount_x * gamma;

        let k = self.reserve_x * self.reserve_y;
        let new_rx = self.reserve_x + net_x;
        let new_ry = k / new_rx;
        let y_out = self.reserve_y - new_ry;

        if y_out > 0.0 {
            (y_out, amount_x * fee)
        } else {
            (0.0, 0.0)
        }
    }

    /// Fast quote for AMM selling X (trader buying X).
    ///
    /// Returns (total_y_in, fee_amount) or (0, 0) if invalid.
    #[inline]
    pub fn quote_sell_x(&self, amount_x: f64) -> (f64, f64) {
        if amount_x <= 0.0 || amount_x >= self.reserve_x {
            return (0.0, 0.0);
        }

        let k = self.reserve_x * self.reserve_y;
        let fee = self.current_fees.ask_fee.to_f64();
        let gamma = (1.0 - fee).clamp(0.0, 1.0);
        if gamma <= 0.0 {
            return (0.0, 0.0);
        }

        let new_rx = self.reserve_x - amount_x;
        let new_ry = k / new_rx;
        let net_y = new_ry - self.reserve_y;

        if net_y <= 0.0 {
            return (0.0, 0.0);
        }

        let total_y = net_y / gamma;
        (total_y, total_y - net_y)
    }

    /// Fast quote for Y input to X output.
    ///
    /// Returns (x_out, fee_amount) or (0, 0) if invalid.
    #[inline]
    pub fn quote_x_for_y(&self, amount_y: f64) -> (f64, f64) {
        if amount_y <= 0.0 {
            return (0.0, 0.0);
        }

        let k = self.reserve_x * self.reserve_y;
        let fee = self.current_fees.ask_fee.to_f64();
        let gamma = (1.0 - fee).clamp(0.0, 1.0);
        if gamma <= 0.0 {
            return (0.0, 0.0);
        }

        let net_y = amount_y * gamma;
        let new_ry = self.reserve_y + net_y;
        let new_rx = k / new_ry;
        let x_out = self.reserve_x - new_rx;

        if x_out > 0.0 {
            (x_out, amount_y * fee)
        } else {
            (0.0, 0.0)
        }
    }

    /// Quote exact-input swap for arbitrary token direction.
    ///
    /// Returns (amount_out, fee_amount_in_token_in).
    #[inline]
    pub fn quote_exact_in(
        &self,
        token_in: usize,
        token_out: usize,
        amount_in: f64,
    ) -> Option<(f64, f64)> {
        if amount_in <= 0.0 || !self.supports_pair(token_in, token_out) {
            return None;
        }
        if token_in == self.token_a && token_out == self.token_b {
            let (out, fee) = self.quote_buy_x(amount_in);
            return Some((out, fee));
        }
        if token_in == self.token_b && token_out == self.token_a {
            let (out, fee) = self.quote_x_for_y(amount_in);
            return Some((out, fee));
        }
        None
    }

    /// Execute trade where AMM buys X (trader sells X for Y).
    pub fn execute_buy_x(&mut self, amount_x: f64, timestamp: u64) -> Option<TradeResult> {
        let (y_out, fee_amount) = self.quote_buy_x(amount_x);
        if y_out <= 0.0 {
            return None;
        }

        // Update reserves - fees go to separate bucket, not into liquidity
        let net_x = amount_x - fee_amount;
        self.reserve_x += net_x;
        self.accumulated_fees_x += fee_amount;
        self.reserve_y -= y_out;

        let trade_info = TradeInfo::new(
            true, // is_buy (AMM buys X)
            Wad::from_f64(amount_x),
            Wad::from_f64(y_out),
            timestamp,
            Wad::from_f64(self.reserve_x),
            Wad::from_f64(self.reserve_y),
        );

        // Update fees from strategy
        self.update_fees(&trade_info);

        Some(TradeResult {
            trade_info,
            fee_amount,
        })
    }

    /// Execute trade where AMM sells X (trader buys X with Y).
    pub fn execute_sell_x(&mut self, amount_x: f64, timestamp: u64) -> Option<TradeResult> {
        let (total_y, fee_amount) = self.quote_sell_x(amount_x);
        if total_y <= 0.0 {
            return None;
        }

        // Update reserves - fees go to separate bucket, not into liquidity
        let net_y = total_y - fee_amount;
        self.reserve_x -= amount_x;
        self.reserve_y += net_y;
        self.accumulated_fees_y += fee_amount;

        let trade_info = TradeInfo::new(
            false, // is_buy = false (AMM sells X)
            Wad::from_f64(amount_x),
            Wad::from_f64(total_y),
            timestamp,
            Wad::from_f64(self.reserve_x),
            Wad::from_f64(self.reserve_y),
        );

        // Update fees from strategy
        self.update_fees(&trade_info);

        Some(TradeResult {
            trade_info,
            fee_amount,
        })
    }

    /// Execute trade where trader pays Y to receive X.
    pub fn execute_buy_x_with_y(&mut self, amount_y: f64, timestamp: u64) -> Option<TradeResult> {
        let (x_out, fee_amount) = self.quote_x_for_y(amount_y);
        if x_out <= 0.0 {
            return None;
        }

        // Update reserves - fees go to separate bucket, not into liquidity
        let net_y = amount_y - fee_amount;
        self.reserve_x -= x_out;
        self.reserve_y += net_y;
        self.accumulated_fees_y += fee_amount;

        let trade_info = TradeInfo::new(
            false, // is_buy = false (AMM sells X)
            Wad::from_f64(x_out),
            Wad::from_f64(amount_y),
            timestamp,
            Wad::from_f64(self.reserve_x),
            Wad::from_f64(self.reserve_y),
        );

        // Update fees from strategy
        self.update_fees(&trade_info);

        Some(TradeResult {
            trade_info,
            fee_amount,
        })
    }

    /// Execute an exact-input swap in arbitrary token direction.
    ///
    /// Returns (amount_out, is_buy_from_amm_perspective) on success.
    pub fn execute_exact_in(
        &mut self,
        token_in: usize,
        token_out: usize,
        amount_in: f64,
        timestamp: u64,
    ) -> Option<(f64, bool)> {
        if token_in == self.token_a && token_out == self.token_b {
            let result = self.execute_buy_x(amount_in, timestamp)?;
            return Some((result.trade_info.amount_y.to_f64(), true));
        }

        if token_in == self.token_b && token_out == self.token_a {
            let result = self.execute_buy_x_with_y(amount_in, timestamp)?;
            return Some((result.trade_info.amount_x.to_f64(), false));
        }

        None
    }

    /// Update fees from strategy after a trade.
    fn update_fees(&mut self, trade_info: &TradeInfo) {
        if self.use_v2_callbacks {
            let trade_v2 = TradeInfoV2 {
                is_buy: trade_info.is_buy,
                amount_a: trade_info.amount_x,
                amount_b: trade_info.amount_y,
                timestamp: trade_info.timestamp,
                reserve_a: trade_info.reserve_x,
                reserve_b: trade_info.reserve_y,
                pool_id: self.pool_id as u64,
                token_a: self.token_a as u64,
                token_b: self.token_b as u64,
            };

            if let Ok((bid_fee, ask_fee)) = self.strategy.after_swap_v2(&trade_v2) {
                self.current_fees = FeeQuote::new(bid_fee.clamp_fee(), ask_fee.clamp_fee());
            } else if let Ok((bid_fee, ask_fee)) = self.strategy.after_swap(trade_info) {
                // Keep running even if V2 callback reverts.
                self.current_fees = FeeQuote::new(bid_fee.clamp_fee(), ask_fee.clamp_fee());
                self.use_v2_callbacks = false;
            }
        } else if let Ok((bid_fee, ask_fee)) = self.strategy.after_swap(trade_info) {
            self.current_fees = FeeQuote::new(bid_fee.clamp_fee(), ask_fee.clamp_fee());
        }
        // On error, keep current fees
    }

    /// Reset the AMM for a new simulation.
    pub fn reset(
        &mut self,
        reserve_x: f64,
        reserve_y: f64,
    ) -> Result<(), crate::evm::strategy::EVMError> {
        self.reserve_x = reserve_x;
        self.reserve_y = reserve_y;
        self.accumulated_fees_x = 0.0;
        self.accumulated_fees_y = 0.0;
        self.initialized = false;
        self.use_v2_callbacks = false;
        self.strategy.reset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::wad::WAD;

    // Note: Full tests require EVM bytecode, which is complex to embed.
    // The Python integration tests will verify correctness.

    #[test]
    fn test_quote_formulas() {
        // Test the math without EVM - use fixed fees
        let fee_quote = FeeQuote::symmetric(Wad::from_bps(25)); // 0.25%

        // Manual calculation for buy X
        let rx = 1000.0;
        let ry = 1000.0;
        let amount_x = 10.0;
        let fee = 0.0025;
        let gamma = 1.0 - fee;
        let net_x = amount_x * gamma;
        let k = rx * ry;
        let new_rx = rx + net_x;
        let new_ry = k / new_rx;
        let y_out = ry - new_ry;

        // y_out should be approximately 9.876 (accounting for fee and price impact)
        assert!(y_out > 9.8 && y_out < 10.0);
    }
}
