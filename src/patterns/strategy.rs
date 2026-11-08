//! Strategy Pattern — Pluggable algorithms for precision handling.
//!
//! Precision strategies determine how location coordinates are rounded
//! before being displayed or shared. Each strategy implements the same
//! interface so they can be swapped at runtime based on the active
//! share's precision level.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Precision level as defined by the API contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionLevel {
    Exact,
    Approximate,
    CityOnly,
}

impl PrecisionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Approximate => "approximate",
            Self::CityOnly => "city_only",
        }
    }
}

impl FromStr for PrecisionLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exact" => Ok(Self::Exact),
            "approximate" => Ok(Self::Approximate),
            "city_only" => Ok(Self::CityOnly),
            _ => Err(format!("Invalid precision level: {s}")),
        }
    }
}

/// A geographic coordinate with precision metadata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
}

/// Trait for precision strategies.
pub trait PrecisionStrategy: Send + Sync {
    /// Returns the precision level this strategy applies.
    fn level(&self) -> PrecisionLevel;

    /// Applies precision rounding to a coordinate.
    fn apply(&self, coord: Coordinate) -> Coordinate;

    /// Returns a human-readable description of the precision.
    fn description(&self) -> &'static str;
}

/// Exact precision — no rounding applied.
pub struct ExactStrategy;

impl PrecisionStrategy for ExactStrategy {
    fn level(&self) -> PrecisionLevel {
        PrecisionLevel::Exact
    }

    fn apply(&self, coord: Coordinate) -> Coordinate {
        coord
    }

    fn description(&self) -> &'static str {
        "Exact (GPS coordinates)"
    }
}

/// Approximate precision — rounds to ~1.1 km (2 decimal places).
pub struct ApproximateStrategy;

impl PrecisionStrategy for ApproximateStrategy {
    fn level(&self) -> PrecisionLevel {
        PrecisionLevel::Approximate
    }

    fn apply(&self, coord: Coordinate) -> Coordinate {
        Coordinate {
            latitude: (coord.latitude * 100.0).round() / 100.0,
            longitude: (coord.longitude * 100.0).round() / 100.0,
            accuracy_meters: Some(250.0),
        }
    }

    fn description(&self) -> &'static str {
        "Approximate (neighborhood, ~250m)"
    }
}

/// City-only precision — rounds to ~11 km (1 decimal place).
pub struct CityOnlyStrategy;

impl PrecisionStrategy for CityOnlyStrategy {
    fn level(&self) -> PrecisionLevel {
        PrecisionLevel::CityOnly
    }

    fn apply(&self, coord: Coordinate) -> Coordinate {
        Coordinate {
            latitude: (coord.latitude * 10.0).round() / 10.0,
            longitude: (coord.longitude * 10.0).round() / 10.0,
            accuracy_meters: Some(11000.0),
        }
    }

    fn description(&self) -> &'static str {
        "City only (city level, ~11km)"
    }
}

/// Context that holds the current precision strategy and delegates to it.
pub struct PrecisionContext {
    strategy: Box<dyn PrecisionStrategy>,
}

impl PrecisionContext {
    pub fn new(level: PrecisionLevel) -> Self {
        Self {
            strategy: Self::strategy_for(level),
        }
    }

    fn strategy_for(level: PrecisionLevel) -> Box<dyn PrecisionStrategy> {
        match level {
            PrecisionLevel::Exact => Box::new(ExactStrategy),
            PrecisionLevel::Approximate => Box::new(ApproximateStrategy),
            PrecisionLevel::CityOnly => Box::new(CityOnlyStrategy),
        }
    }

    pub fn set_strategy(&mut self, level: PrecisionLevel) {
        self.strategy = Self::strategy_for(level);
    }

    pub fn level(&self) -> PrecisionLevel {
        self.strategy.level()
    }

    pub fn apply(&self, coord: Coordinate) -> Coordinate {
        self.strategy.apply(coord)
    }

    pub fn description(&self) -> &'static str {
        self.strategy.description()
    }
}

impl Default for PrecisionContext {
    fn default() -> Self {
        Self::new(PrecisionLevel::Exact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_preserves_coordinates() {
        let ctx = PrecisionContext::new(PrecisionLevel::Exact);
        let coord = Coordinate {
            latitude: 41.890234,
            longitude: -87.632345,
            accuracy_meters: Some(8.0),
        };
        let result = ctx.apply(coord);
        assert_eq!(result.latitude, 41.890234);
        assert_eq!(result.longitude, -87.632345);
    }

    #[test]
    fn approximate_rounds_to_2_decimals() {
        let ctx = PrecisionContext::new(PrecisionLevel::Approximate);
        let coord = Coordinate {
            latitude: 41.890234,
            longitude: -87.632345,
            accuracy_meters: Some(8.0),
        };
        let result = ctx.apply(coord);
        assert_eq!(result.latitude, 41.89);
        assert_eq!(result.longitude, -87.63);
        assert_eq!(result.accuracy_meters, Some(250.0));
    }

    #[test]
    fn city_only_rounds_to_1_decimal() {
        let ctx = PrecisionContext::new(PrecisionLevel::CityOnly);
        let coord = Coordinate {
            latitude: 41.890234,
            longitude: -87.632345,
            accuracy_meters: Some(8.0),
        };
        let result = ctx.apply(coord);
        assert_eq!(result.latitude, 41.9);
        assert_eq!(result.longitude, -87.6);
        assert_eq!(result.accuracy_meters, Some(11000.0));
    }

    #[test]
    fn strategy_switching() {
        let mut ctx = PrecisionContext::new(PrecisionLevel::Exact);
        assert_eq!(ctx.level(), PrecisionLevel::Exact);

        ctx.set_strategy(PrecisionLevel::CityOnly);
        assert_eq!(ctx.level(), PrecisionLevel::CityOnly);
        assert_eq!(ctx.description(), "City only (city level, ~11km)");
    }
}
