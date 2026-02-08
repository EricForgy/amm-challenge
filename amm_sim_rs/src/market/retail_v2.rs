//! Multi-asset retail flow generator.

use rand::Rng;
use rand::SeedableRng;
use rand_distr::{Distribution, LogNormal, Poisson};
use rand_pcg::Pcg64;

/// Multi-asset retail order in token-index space.
#[derive(Debug, Clone)]
pub struct RetailOrderV2 {
    pub token_in: usize,
    pub token_out: usize,
    /// Order notional measured in numeraire terms.
    pub size_numeraire: f64,
}

/// Generates retail flow across a token universe.
///
/// Buy probability controls direction for a sampled pair (a,b):
/// - buy: token_in=b, token_out=a
/// - sell: token_in=a, token_out=b
pub struct RetailTraderV2 {
    n_assets: usize,
    buy_prob: f64,
    rng: Pcg64,
    poisson: Poisson<f64>,
    lognormal: LogNormal<f64>,
}

impl RetailTraderV2 {
    pub fn new(
        n_assets: usize,
        arrival_rate: f64,
        mean_size_numeraire: f64,
        size_sigma: f64,
        buy_prob: f64,
        seed: Option<u64>,
    ) -> Self {
        let rng = match seed {
            Some(s) => Pcg64::seed_from_u64(s),
            None => Pcg64::from_entropy(),
        };
        let poisson = Poisson::new(arrival_rate.max(0.01)).unwrap_or_else(|_| Poisson::new(1.0).unwrap());
        let sigma = size_sigma.max(0.01);
        let mean = mean_size_numeraire.max(0.01);
        let mu = mean.ln() - 0.5 * sigma * sigma;
        let lognormal = LogNormal::new(mu, sigma).unwrap_or_else(|_| LogNormal::new(0.0, 1.0).unwrap());

        Self {
            n_assets,
            buy_prob,
            rng,
            poisson,
            lognormal,
        }
    }

    pub fn generate_orders(&mut self) -> Vec<RetailOrderV2> {
        let n_arrivals = self.poisson.sample(&mut self.rng) as usize;
        if n_arrivals == 0 {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(n_arrivals);
        for _ in 0..n_arrivals {
            let a = self.rng.gen_range(0..self.n_assets);
            let mut b = self.rng.gen_range(0..(self.n_assets - 1));
            if b >= a {
                b += 1;
            }
            let (token_in, token_out) = if self.rng.gen::<f64>() < self.buy_prob {
                (b, a)
            } else {
                (a, b)
            };
            out.push(RetailOrderV2 {
                token_in,
                token_out,
                size_numeraire: self.lognormal.sample(&mut self.rng),
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::RetailTraderV2;

    #[test]
    fn test_retail_v2_deterministic() {
        let mut t1 = RetailTraderV2::new(3, 1.2, 20.0, 1.2, 0.5, Some(7));
        let mut t2 = RetailTraderV2::new(3, 1.2, 20.0, 1.2, 0.5, Some(7));
        for _ in 0..10 {
            let o1 = t1.generate_orders();
            let o2 = t2.generate_orders();
            assert_eq!(o1.len(), o2.len());
            for (a, b) in o1.iter().zip(o2.iter()) {
                assert_eq!(a.token_in, b.token_in);
                assert_eq!(a.token_out, b.token_out);
                assert_eq!(a.size_numeraire, b.size_numeraire);
            }
        }
    }
}
