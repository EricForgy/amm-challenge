"""Lightweight smoke checks for multi-asset V2 strategy runtime.

This script is intended for CI where full pytest/bench runs may be unavailable.
It validates key V2 paths end-to-end:
1) V2-only callback strategy compiles/validates/runs
2) V2 callback revert falls back to V1 callbacks at runtime
"""

from __future__ import annotations

import math
import sys

import amm_sim_rs

from amm_competition.evm.baseline import get_vanilla_bytecode_and_abi
from amm_competition.evm.compiler import SolidityCompiler
from amm_competition.evm.validator import SolidityValidator


V2_ONLY_SOURCE = """// SPDX-License-Identifier: MIT
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


V2_FALLBACK_SOURCE = """// SPDX-License-Identifier: MIT
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


def _build_cfg(seed: int) -> amm_sim_rs.SimulationConfigV2:
    return amm_sim_rs.SimulationConfigV2(
        n_steps=15,
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
        seed=seed,
    )


def _assert_finite(x: float, label: str) -> None:
    if not math.isfinite(x):
        raise AssertionError(f"{label} is not finite: {x}")


def _compile_and_validate(source: str) -> bytes:
    validator = SolidityValidator()
    validation = validator.validate(source)
    if not validation.valid:
        raise AssertionError(f"Validation failed: {validation.errors}")

    compiler = SolidityCompiler()
    compiled = compiler.compile(source)
    if not compiled.success:
        raise AssertionError(f"Compilation failed: {compiled.errors}")
    return compiled.bytecode


def main() -> int:
    baseline_bytecode, _ = get_vanilla_bytecode_and_abi()

    # Case 1: V2-only callbacks
    v2_only_bytecode = _compile_and_validate(V2_ONLY_SOURCE)
    result_v2_only = amm_sim_rs.run_batch_v2(
        list(v2_only_bytecode),
        list(baseline_bytecode),
        [_build_cfg(seed=11)],
        1,
    )
    if len(result_v2_only.results) != 1:
        raise AssertionError("Expected exactly 1 result for V2-only case")
    edge_v2_only = result_v2_only.results[0].edges.get("submission")
    if edge_v2_only is None:
        raise AssertionError("Missing submission edge for V2-only case")
    _assert_finite(edge_v2_only, "V2-only edge")

    # Case 2: V2 callback failure with V1 fallback
    fallback_bytecode = _compile_and_validate(V2_FALLBACK_SOURCE)
    result_fallback = amm_sim_rs.run_batch_v2(
        list(fallback_bytecode),
        list(baseline_bytecode),
        [_build_cfg(seed=12)],
        1,
    )
    if len(result_fallback.results) != 1:
        raise AssertionError("Expected exactly 1 result for fallback case")
    edge_fallback = result_fallback.results[0].edges.get("submission")
    if edge_fallback is None:
        raise AssertionError("Missing submission edge for fallback case")
    _assert_finite(edge_fallback, "Fallback edge")

    print("V2 smoke checks passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"V2 smoke checks failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
