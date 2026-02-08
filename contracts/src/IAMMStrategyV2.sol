// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title Trade information passed to V2 AMM strategies
/// @notice Includes pool and token context for multi-asset simulations
struct TradeInfoV2 {
    bool isBuy;          // true if AMM bought tokenA (trader sold tokenA)
    uint256 amountA;     // Amount of tokenA traded (WAD precision)
    uint256 amountB;     // Amount of tokenB traded (WAD precision)
    uint256 timestamp;   // Simulation step number
    uint256 reserveA;    // Post-trade tokenA reserves (WAD precision)
    uint256 reserveB;    // Post-trade tokenB reserves (WAD precision)
    uint256 poolId;      // Pool identifier
    uint256 tokenA;      // tokenA index in global token list
    uint256 tokenB;      // tokenB index in global token list
}

/// @title AMM Strategy V2 Interface
/// @notice Interface for multi-asset / multi-pool strategies
/// @dev Fees are returned as WAD values (1e18 = 100%)
interface IAMMStrategyV2 {
    /// @notice Initialize with pool-local reserves and pool/token context
    /// @return bidFee Fee when AMM buys tokenA (WAD precision)
    /// @return askFee Fee when AMM sells tokenA (WAD precision)
    function afterInitializeV2(
        uint256 initialA,
        uint256 initialB,
        uint256 poolId,
        uint256 tokenA,
        uint256 tokenB
    ) external returns (uint256 bidFee, uint256 askFee);

    /// @notice Called after each trade for this pool
    /// @return bidFee Updated fee when AMM buys tokenA (WAD precision)
    /// @return askFee Updated fee when AMM sells tokenA (WAD precision)
    function afterSwapV2(TradeInfoV2 calldata trade) external returns (uint256 bidFee, uint256 askFee);

    /// @notice Get strategy display name
    function getName() external view returns (string memory);
}
