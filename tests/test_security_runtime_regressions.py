"""Runtime security regression tests for executor/adapter behavior."""

from decimal import Decimal

import pytest
import amm_sim_rs

from amm_competition.core.trade import TradeInfo
from amm_competition.evm.adapter import EVMStrategyAdapter
from amm_competition.evm.executor import EVMExecutionResult, EVMStrategyExecutor
from amm_competition.evm.compiler import SolidityCompiler


def _sample_trade() -> TradeInfo:
    return TradeInfo(
        side="buy",
        amount_x=Decimal("1"),
        amount_y=Decimal("100"),
        timestamp=1,
        reserve_x=Decimal("99"),
        reserve_y=Decimal("10100"),
    )


def test_after_swap_fast_rejects_short_return_data(vanilla_bytecode_and_abi) -> None:
    bytecode, abi = vanilla_bytecode_and_abi
    executor = EVMStrategyExecutor(bytecode=bytecode, abi=abi)

    class ShortReturnEVM:
        def message_call(self, **kwargs):
            return b"\x00" * 63

    executor.evm = ShortReturnEVM()

    with pytest.raises(RuntimeError, match="afterSwap failed: Invalid return data length"):
        executor.after_swap_fast(_sample_trade())


def test_after_swap_fast_surfaces_evm_errors(vanilla_bytecode_and_abi) -> None:
    bytecode, abi = vanilla_bytecode_and_abi
    executor = EVMStrategyExecutor(bytecode=bytecode, abi=abi)

    class ExplodingEVM:
        def message_call(self, **kwargs):
            raise RuntimeError("boom")

    executor.evm = ExplodingEVM()

    with pytest.raises(RuntimeError, match="afterSwap failed: boom"):
        executor.after_swap_fast(_sample_trade())


def test_adapter_clamps_out_of_range_initialize_fees(vanilla_bytecode_and_abi) -> None:
    bytecode, abi = vanilla_bytecode_and_abi
    adapter = EVMStrategyAdapter(bytecode=bytecode, abi=abi)

    class FakeExecutor:
        def after_initialize(self, initial_x, initial_y):
            return EVMExecutionResult(
                bid_fee=Decimal("-1"),
                ask_fee=Decimal("999"),
                gas_used=123,
                success=True,
            )

    adapter._executor = FakeExecutor()
    quote = adapter.after_initialize(Decimal("100"), Decimal("10000"))
    assert quote.bid_fee == Decimal("0")
    assert quote.ask_fee == Decimal("0.1")


def test_adapter_clamps_out_of_range_swap_fees(vanilla_bytecode_and_abi) -> None:
    bytecode, abi = vanilla_bytecode_and_abi
    adapter = EVMStrategyAdapter(bytecode=bytecode, abi=abi)

    class FakeExecutor:
        def after_swap_fast(self, trade):
            return (-1, 2 * 10**17)  # -1 WAD, 20% WAD

    adapter._executor = FakeExecutor()
    quote = adapter.after_swap(_sample_trade())
    assert quote.bid_fee == Decimal("0")
    assert quote.ask_fee == Decimal("0.1")


def test_run_batch_v2_accepts_v2_only_callbacks(vanilla_bytecode_and_abi) -> None:
    source = """// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {AMMStrategyBaseV2} from "./AMMStrategyBaseV2.sol";
import {TradeInfoV2} from "./IAMMStrategyV2.sol";

contract Strategy is AMMStrategyBaseV2 {
    uint256 public constant FEE = 45 * BPS;

    function afterInitializeV2(uint256, uint256, uint256, uint256, uint256) external pure override returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function afterSwapV2(TradeInfoV2 calldata) external pure override returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function getName() external pure override returns (string memory) {
        return "V2Only";
    }
}
"""
    compiled = SolidityCompiler().compile(source)
    assert compiled.success, compiled.errors

    baseline_bytecode, _ = vanilla_bytecode_and_abi
    cfg = amm_sim_rs.SimulationConfigV2(
        n_steps=10,
        initial_prices=[1.0, 100.0, 150.0],
        gbm_mu=0.0,
        gbm_sigma=0.001,
        gbm_dt=1.0,
        retail_arrival_rate=0.8,
        retail_mean_size=20.0,
        retail_size_sigma=1.2,
        retail_buy_prob=0.5,
        numeraire_token=0,
        pools=[(0, 1, 10000.0, 100.0), (0, 2, 10000.0, 66.6667), (1, 2, 100.0, 66.6667)],
        seed=1,
    )
    result = amm_sim_rs.run_batch_v2(
        list(compiled.bytecode),
        list(baseline_bytecode),
        [cfg],
        1,
    )
    assert len(result.results) == 1
    assert "submission" in result.results[0].edges


def test_run_batch_v2_falls_back_to_v1_when_v2_swap_reverts(vanilla_bytecode_and_abi) -> None:
    source = """// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {AMMStrategyBaseV2} from "./AMMStrategyBaseV2.sol";
import {TradeInfoV2} from "./IAMMStrategyV2.sol";
import {TradeInfo} from "./IAMMStrategy.sol";

contract Strategy is AMMStrategyBaseV2 {
    uint256 public constant FEE = 35 * BPS;

    function afterInitializeV2(uint256, uint256, uint256, uint256, uint256) external pure override returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function afterSwapV2(TradeInfoV2 calldata) external pure override returns (uint256, uint256) {
        revert("v2 disabled");
    }

    // Fallback callbacks for runtime compatibility.
    function afterInitialize(uint256, uint256) external pure returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function afterSwap(TradeInfo calldata) external pure returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function getName() external pure override returns (string memory) {
        return "V2Fallback";
    }
}
"""
    compiled = SolidityCompiler().compile(source)
    assert compiled.success, compiled.errors

    baseline_bytecode, _ = vanilla_bytecode_and_abi
    cfg = amm_sim_rs.SimulationConfigV2(
        n_steps=20,
        initial_prices=[1.0, 100.0, 150.0],
        gbm_mu=0.0,
        gbm_sigma=0.001,
        gbm_dt=1.0,
        retail_arrival_rate=0.9,
        retail_mean_size=20.0,
        retail_size_sigma=1.2,
        retail_buy_prob=0.5,
        numeraire_token=0,
        pools=[(0, 1, 10000.0, 100.0), (0, 2, 10000.0, 66.6667), (1, 2, 100.0, 66.6667)],
        seed=2,
    )

    # Should not raise even though afterSwapV2 reverts (engine should fallback to V1 callbacks).
    result = amm_sim_rs.run_batch_v2(
        list(compiled.bytecode),
        list(baseline_bytecode),
        [cfg],
        1,
    )
    assert len(result.results) == 1
