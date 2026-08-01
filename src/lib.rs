//! # Ferronote Crate
//!
//! A blazing-fast terminal note-taking library and application inspired by Notational Velocity.
//! Built on **The Elm Architecture (TEA)** with `ratatui` and `crossterm`.

pub mod action;
pub mod app;
pub mod components;
pub mod config;
pub mod environment;
pub mod event;
pub mod focus;
pub mod note_store;
pub mod queue;
pub mod search;
pub mod theme;
pub mod tui;
