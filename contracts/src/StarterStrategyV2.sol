// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {AMMStrategyBaseV2} from "./AMMStrategyBaseV2.sol";
import {TradeInfoV2} from "./IAMMStrategyV2.sol";
import {TradeInfo} from "./IAMMStrategy.sol";

/// @title Starter Strategy V2 - 50 Basis Points
/// @notice Starter template for multi-asset / multi-pool simulations
contract Strategy is AMMStrategyBaseV2 {
    uint256 public constant FEE = 50 * BPS;

    function afterInitializeV2(
        uint256,
        uint256,
        uint256,
        uint256,
        uint256
    ) external pure override returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function afterSwapV2(TradeInfoV2 calldata) external pure override returns (uint256, uint256) {
        return (FEE, FEE);
    }

    // Compatibility entrypoints for current runtime.
    function afterInitialize(uint256, uint256) external pure returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function afterSwap(TradeInfo calldata) external pure returns (uint256, uint256) {
        return (FEE, FEE);
    }

    function getName() external pure override returns (string memory) {
        return "StarterStrategyV2";
    }
}
