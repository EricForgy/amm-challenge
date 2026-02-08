// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IAMMStrategyV2, TradeInfoV2} from "./IAMMStrategyV2.sol";

/// @title AMM Strategy Base Contract V2
/// @notice Base contract for multi-asset / multi-pool strategy implementations
abstract contract AMMStrategyBaseV2 is IAMMStrategyV2 {
    /// @notice 1e18 - represents 100% in WAD precision
    uint256 public constant WAD = 1e18;

    /// @notice Maximum allowed fee: 10% (1e17)
    uint256 public constant MAX_FEE = WAD / 10;

    /// @notice Minimum allowed fee: 0
    uint256 public constant MIN_FEE = 0;

    /// @notice 1 basis point in WAD (0.01% = 0.0001 = 1e14)
    uint256 public constant BPS = 1e14;

    /// @notice Fixed storage array - strategies can only use these 32 slots
    uint256[32] public slots;

    function wmul(uint256 x, uint256 y) internal pure returns (uint256) {
        return (x * y) / WAD;
    }

    function wdiv(uint256 x, uint256 y) internal pure returns (uint256) {
        return (x * WAD) / y;
    }

    function clamp(uint256 value, uint256 minVal, uint256 maxVal) internal pure returns (uint256) {
        if (value < minVal) return minVal;
        if (value > maxVal) return maxVal;
        return value;
    }

    function bpsToWad(uint256 bps) internal pure returns (uint256) {
        return bps * BPS;
    }

    function wadToBps(uint256 wadValue) internal pure returns (uint256) {
        return wadValue / BPS;
    }

    function clampFee(uint256 fee) internal pure returns (uint256) {
        return clamp(fee, MIN_FEE, MAX_FEE);
    }

    function absDiff(uint256 a, uint256 b) internal pure returns (uint256) {
        return a > b ? a - b : b - a;
    }

    function sqrt(uint256 x) internal pure returns (uint256 y) {
        if (x == 0) return 0;
        uint256 z = (x + 1) / 2;
        y = x;
        while (z < y) {
            y = z;
            z = (x / z + z) / 2;
        }
    }

    function readSlot(uint256 index) internal view returns (uint256) {
        require(index < 32, "Slot index out of bounds");
        return slots[index];
    }

    function writeSlot(uint256 index, uint256 value) internal {
        require(index < 32, "Slot index out of bounds");
        slots[index] = value;
    }
}
