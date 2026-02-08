"""Tests for competition framework."""

import pytest
from decimal import Decimal

import amm_sim_rs

from amm_competition.competition.config import (
    BASELINE_VARIANCE,
    build_base_config,
    build_v2_configs_from_legacy,
)
from amm_competition.competition.match import (
    MatchRunner,
    MatchRunnerV2,
    HyperparameterVariance,
)


class TestMatchRunner:
    def test_run_match(self, vanilla_bytecode_and_abi):
        from amm_competition.evm.adapter import EVMStrategyAdapter

        config = amm_sim_rs.SimulationConfig(
            n_steps=50,
            initial_price=100.0,
            initial_x=100.0,
            initial_y=10000.0,
            gbm_mu=0.0,
            gbm_sigma=0.001,
            gbm_dt=1.0,
            retail_arrival_rate=5.0,
            retail_mean_size=2.0,
            retail_size_sigma=0.7,
            retail_buy_prob=0.5,
            seed=42,
        )
        variance = HyperparameterVariance(
            retail_mean_size_min=2.0,
            retail_mean_size_max=2.0,
            vary_retail_mean_size=False,
            retail_arrival_rate_min=5.0,
            retail_arrival_rate_max=5.0,
            vary_retail_arrival_rate=False,
            gbm_sigma_min=0.001,
            gbm_sigma_max=0.001,
            vary_gbm_sigma=False,
        )
        runner = MatchRunner(n_simulations=5, config=config, n_workers=1, variance=variance)

        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi, name="Vanilla_30bps")

        result = runner.run_match(strategy_a, strategy_b)

        assert result.total_games == 5
        assert result.wins_a + result.wins_b + result.draws == 5
        assert result.strategy_a == "Vanilla_30bps"
        assert result.strategy_b == "Vanilla_30bps"

    def test_match_winner(self, vanilla_bytecode_and_abi):
        from amm_competition.evm.adapter import EVMStrategyAdapter

        config = amm_sim_rs.SimulationConfig(
            n_steps=50,
            initial_price=100.0,
            initial_x=100.0,
            initial_y=10000.0,
            gbm_mu=0.0,
            gbm_sigma=0.001,
            gbm_dt=1.0,
            retail_arrival_rate=5.0,
            retail_mean_size=2.0,
            retail_size_sigma=0.7,
            retail_buy_prob=0.5,
            seed=42,
        )
        variance = HyperparameterVariance(
            retail_mean_size_min=2.0,
            retail_mean_size_max=2.0,
            vary_retail_mean_size=False,
            retail_arrival_rate_min=5.0,
            retail_arrival_rate_max=5.0,
            vary_retail_arrival_rate=False,
            gbm_sigma_min=0.001,
            gbm_sigma_max=0.001,
            vary_gbm_sigma=False,
        )
        runner = MatchRunner(n_simulations=11, config=config, n_workers=1, variance=variance)

        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi)

        result = runner.run_match(strategy_a, strategy_b)

        # Winner can be either, but total should be 11
        assert result.total_games == 11

    def test_pnl_accumulated(self, vanilla_bytecode_and_abi):
        from amm_competition.evm.adapter import EVMStrategyAdapter

        config = amm_sim_rs.SimulationConfig(
            n_steps=50,
            initial_price=100.0,
            initial_x=100.0,
            initial_y=10000.0,
            gbm_mu=0.0,
            gbm_sigma=0.001,
            gbm_dt=1.0,
            retail_arrival_rate=5.0,
            retail_mean_size=2.0,
            retail_size_sigma=0.7,
            retail_buy_prob=0.5,
            seed=42,
        )
        variance = HyperparameterVariance(
            retail_mean_size_min=2.0,
            retail_mean_size_max=2.0,
            vary_retail_mean_size=False,
            retail_arrival_rate_min=5.0,
            retail_arrival_rate_max=5.0,
            vary_retail_arrival_rate=False,
            gbm_sigma_min=0.001,
            gbm_sigma_max=0.001,
            vary_gbm_sigma=False,
        )
        runner = MatchRunner(n_simulations=5, config=config, n_workers=1, variance=variance)

        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi, name="Vanilla_50bps")

        result = runner.run_match(strategy_a, strategy_b)

        # PNL should be accumulated across simulations
        assert result.total_pnl_a != Decimal("0") or result.total_pnl_b != Decimal("0")

    def test_store_results(self, vanilla_bytecode_and_abi):
        from amm_competition.evm.adapter import EVMStrategyAdapter

        config = amm_sim_rs.SimulationConfig(
            n_steps=50,
            initial_price=100.0,
            initial_x=100.0,
            initial_y=10000.0,
            gbm_mu=0.0,
            gbm_sigma=0.001,
            gbm_dt=1.0,
            retail_arrival_rate=5.0,
            retail_mean_size=2.0,
            retail_size_sigma=0.7,
            retail_buy_prob=0.5,
            seed=42,
        )
        variance = HyperparameterVariance(
            retail_mean_size_min=2.0,
            retail_mean_size_max=2.0,
            vary_retail_mean_size=False,
            retail_arrival_rate_min=5.0,
            retail_arrival_rate_max=5.0,
            vary_retail_arrival_rate=False,
            gbm_sigma_min=0.001,
            gbm_sigma_max=0.001,
            vary_gbm_sigma=False,
        )
        runner = MatchRunner(n_simulations=3, config=config, n_workers=1, variance=variance)

        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi, name="Vanilla_30bps")

        result = runner.run_match(strategy_a, strategy_b, store_results=True)

        assert len(result.simulation_results) == 3

    def test_same_name_strategies_no_collision(self, vanilla_bytecode_and_abi):
        """Test that strategies with the same getName() don't cause HashMap collision."""
        from amm_competition.evm.adapter import EVMStrategyAdapter

        config = amm_sim_rs.SimulationConfig(
            n_steps=50,
            initial_price=100.0,
            initial_x=100.0,
            initial_y=10000.0,
            gbm_mu=0.0,
            gbm_sigma=0.001,
            gbm_dt=1.0,
            retail_arrival_rate=5.0,
            retail_mean_size=2.0,
            retail_size_sigma=0.7,
            retail_buy_prob=0.5,
            seed=42,
        )
        variance = HyperparameterVariance(
            retail_mean_size_min=2.0,
            retail_mean_size_max=2.0,
            vary_retail_mean_size=False,
            retail_arrival_rate_min=5.0,
            retail_arrival_rate_max=5.0,
            vary_retail_arrival_rate=False,
            gbm_sigma_min=0.001,
            gbm_sigma_max=0.001,
            vary_gbm_sigma=False,
        )
        runner = MatchRunner(n_simulations=5, config=config, n_workers=1, variance=variance)

        # Both strategies use same bytecode and will have same getName() return value
        # Without the fix, this would cause a HashMap key collision
        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi)

        # Both should return "Vanilla_30bps" from get_name()
        assert strategy_a.get_name() == strategy_b.get_name() == "Vanilla_30bps"

        result = runner.run_match(strategy_a, strategy_b, store_results=True)

        # Should complete without errors and have valid results
        assert result.total_games == 5
        # Since both strategies are identical, results should be a draw or close
        # The important thing is that we get results for both, not zeros
        assert len(result.simulation_results) == 5
        # Check that simulation results contain data for both strategies
        first_sim = result.simulation_results[0]
        assert len(first_sim.pnl) == 2  # Should have PnL for both strategies


class TestMatchRunnerV2:
    def _build_v2_configs(self, n: int) -> list[amm_sim_rs.SimulationConfigV2]:
        configs = []
        for i in range(n):
            configs.append(
                amm_sim_rs.SimulationConfigV2(
                    n_steps=30,
                    initial_prices=[1.0, 100.0, 150.0],
                    gbm_mu=0.0,
                    gbm_sigma=0.001,
                    gbm_dt=1.0,
                    retail_arrival_rate=0.8,
                    retail_mean_size=20.0,
                    retail_size_sigma=1.2,
                    retail_buy_prob=0.5,
                    numeraire_token=0,
                    pools=[
                        (0, 1, 10_000.0, 100.0),
                        (0, 2, 10_000.0, 66.6667),
                        (1, 2, 100.0, 66.6667),
                    ],
                    seed=i,
                )
            )
        return configs

    def test_run_match_v2(self, vanilla_bytecode_and_abi):
        from amm_competition.evm.adapter import EVMStrategyAdapter

        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi)

        runner = MatchRunnerV2(configs=self._build_v2_configs(4), n_workers=1)
        result = runner.run_match(strategy_a, strategy_b)

        assert result.total_games == 4
        assert result.wins_a + result.wins_b + result.draws == 4
        assert result.total_edge_a != Decimal("0") or result.total_edge_b != Decimal("0")

    def test_run_match_v2_store_results(self, vanilla_bytecode_and_abi):
        from amm_competition.evm.adapter import EVMStrategyAdapter

        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi)

        runner = MatchRunnerV2(configs=self._build_v2_configs(2), n_workers=1)
        result = runner.run_match(strategy_a, strategy_b, store_results=True)

        assert len(result.simulation_results) == 2
        first = result.simulation_results[0]
        assert len(first.final_prices) == 3
        # 3 configured pools x 2 strategies = 6 pool instances
        assert len(first.pools) == 6

    def test_from_legacy_builder(self, vanilla_bytecode_and_abi):
        from amm_competition.evm.adapter import EVMStrategyAdapter

        base = build_base_config(seed=None)
        variance = HyperparameterVariance(
            retail_mean_size_min=base.retail_mean_size,
            retail_mean_size_max=base.retail_mean_size,
            vary_retail_mean_size=False,
            retail_arrival_rate_min=base.retail_arrival_rate,
            retail_arrival_rate_max=base.retail_arrival_rate,
            vary_retail_arrival_rate=False,
            gbm_sigma_min=base.gbm_sigma,
            gbm_sigma_max=base.gbm_sigma,
            vary_gbm_sigma=False,
        )
        runner = MatchRunnerV2.from_legacy(
            n_simulations=3,
            config=base,
            n_workers=1,
            variance=variance,
        )
        assert len(runner.configs) == 3
        assert runner.configs[0].numeraire_token == 1
        assert runner.configs[0].pools == [(0, 1, base.initial_x, base.initial_y)]

        bytecode, abi = vanilla_bytecode_and_abi
        strategy_a = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        strategy_b = EVMStrategyAdapter(bytecode=bytecode, abi=abi)
        result = runner.run_match(strategy_a, strategy_b)
        assert result.total_games == 3


class TestConfigV2Compatibility:
    def test_build_v2_configs_from_legacy_shapes(self):
        base = build_base_config(seed=7)
        configs = build_v2_configs_from_legacy(
            base_config=base,
            n_simulations=4,
            variance=BASELINE_VARIANCE,
        )

        assert len(configs) == 4
        for i, cfg in enumerate(configs):
            assert cfg.seed == i
            assert cfg.numeraire_token == 1
            assert cfg.initial_prices == [base.initial_price, 1.0]
            assert cfg.pools == [(0, 1, base.initial_x, base.initial_y)]
