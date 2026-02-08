"""Shared lightweight competition types."""

from dataclasses import dataclass


@dataclass
class HyperparameterVariance:
    """Configuration for hyperparameter variance across simulations."""

    retail_mean_size_min: float
    retail_mean_size_max: float
    vary_retail_mean_size: bool

    retail_arrival_rate_min: float
    retail_arrival_rate_max: float
    vary_retail_arrival_rate: bool

    gbm_sigma_min: float
    gbm_sigma_max: float
    vary_gbm_sigma: bool
