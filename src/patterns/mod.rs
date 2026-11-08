//! Design patterns implementation for Pinpoint.
//!
//! This module provides reusable design patterns used throughout the
//! application:
//!
//! - **Repository** — Abstract data access behind trait-based interfaces
//! - **Strategy** — Pluggable algorithms (precision handling, routing)
//! - **Command** — Encapsulate user actions as objects
//! - **State** — Screen/flow state management
//! - **Observer** — Publish/subscribe for state changes
//! - **Factory** — Create API clients and OSM services
//! - **Decorator** — Add logging/caching/retries to services
//! - **Proxy** — Offline/online proxy for network services

pub mod command;
pub mod decorator;
pub mod factory;
pub mod observer;
pub mod proxy;
pub mod repository;
pub mod state;
pub mod strategy;
