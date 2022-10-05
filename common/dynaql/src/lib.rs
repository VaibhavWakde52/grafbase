//! # A GraphQL server library implemented in Rust
//!
//! <div align="center">
//! <!-- CI -->
//! <img src="https://github.com/async-graphql/async-graphql/workflows/CI/badge.svg" />
//! <!-- codecov -->
//! <img src="https://codecov.io/gh/async-graphql/async-graphql/branch/master/graph/badge.svg" />
//! <!-- Crates version -->
//! <a href="https://crates.io/crates/async-graphql">
//! <img src="https://img.shields.io/crates/v/async-graphql.svg?style=flat-square"
//! alt="Crates.io version" />
//! </a>
//! <!-- Downloads -->
//! <a href="https://crates.io/crates/async-graphql">
//! <img src="https://img.shields.io/crates/d/async-graphql.svg?style=flat-square"
//! alt="Download" />
//! </a>
//! <!-- docs.rs docs -->
//! <a href="https://docs.rs/async-graphql">
//! <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
//! alt="docs.rs docs" />
//! </a>
//! <a href="https://github.com/rust-secure-code/safety-dance/">
//! <img src="https://img.shields.io/badge/unsafe-forbidden-success.svg?style=flat-square"
//! alt="Unsafe Rust forbidden" />
//! </a>
//! </div>
//!
//! ## Documentation
//!
//! * [Feature Comparison](https://github.com/async-graphql/async-graphql/blob/master/feature-comparison.md)
//! * [Book](https://async-graphql.github.io/async-graphql/en/index.html)
//! * [中文文档](https://async-graphql.github.io/async-graphql/zh-CN/index.html)
//! * [Docs](https://docs.rs/async-graphql)
//! * [GitHub repository](https://github.com/async-graphql/async-graphql)
//! * [Cargo package](https://crates.io/crates/async-graphql)
//! * Minimum supported Rust version: 1.56.1 or later
//!
//! ## Features
//!
//! * Fully supports async/await
//! * Type safety
//! * Rustfmt friendly (Procedural Macro)
//! * Custom scalars
//! * Minimal overhead
//! * Easy integration ([poem](https://crates.io/crates/poem), actix_web, tide, warp, rocket ...)
//! * File upload (Multipart request)
//! * Subscriptions (WebSocket transport)
//! * Custom extensions
//! * Apollo Tracing extension
//! * Limit query complexity/depth
//! * Error Extensions
//! * Apollo Federation
//! * Batch Queries
//! * Apollo Persisted Queries
//!
//! ## Crate features
//!
//! This crate offers the following features, all of which are not activated by default:
//!
//! - `apollo_tracing`: Enable the [Apollo tracing extension](extensions/struct.ApolloTracing.html).
//! - `apollo_persisted_queries`: Enable the [Apollo persisted queries extension](extensions/apollo_persisted_queries/struct.ApolloPersistedQueries.html).
//! - `log`: Enable the [logger extension](extensions/struct.Logger.html).
//! - `tracing`: Enable the [tracing extension](extensions/struct.Tracing.html).
//! - `opentelemetry`: Enable the [OpenTelemetry extension](extensions/struct.OpenTelemetry.html).
//! - `unblock`: Support [asynchronous reader for Upload](types/struct.Upload.html)
//! - `bson`: Integrate with the [`bson` crate](https://crates.io/crates/bson).
//! - `chrono`: Integrate with the [`chrono` crate](https://crates.io/crates/chrono).
//! - `chrono-tz`: Integrate with the [`chrono-tz` crate](https://crates.io/crates/chrono-tz).
//! - `url`: Integrate with the [`url` crate](https://crates.io/crates/url).
//! - `uuid`: Integrate with the [`uuid` crate](https://crates.io/crates/uuid).
//! - `string_number`: Enable the [StringNumber](types/struct.StringNumber.html).
//! - `dataloader`: Support [DataLoader](dataloader/struct.DataLoader.html).
//! - `decimal`: Integrate with the [`rust_decimal` crate](https://crates.io/crates/rust_decimal).
//! - `cbor`: Support for [serde_cbor](https://crates.io/crates/serde_cbor).
//! - `smol_str`: Integrate with the [`smol_str` crate](https://crates.io/crates/smol_str).
//! - `hashbrown`: Integrate with the [`hashbrown` crate](https://github.com/rust-lang/hashbrown).
//! - `time`: Integrate with the [`time` crate](https://github.com/time-rs/time).
//! - `unstable_oneof`: Enable the `OneofObject` macro to define the oneof input object.
//!
//! ## Integrations
//!
//! * Poem [async-graphql-poem](https://crates.io/crates/async-graphql-poem)
//! * Actix-web [async-graphql-actix-web](https://crates.io/crates/async-graphql-actix-web)
//! * Warp [async-graphql-warp](https://crates.io/crates/async-graphql-warp)
//! * Tide [async-graphql-tide](https://crates.io/crates/async-graphql-tide)
//! * Rocket [async-graphql-rocket](https://github.com/async-graphql/async-graphql/tree/master/integrations/rocket)
//! * Axum [async-graphql-axum](https://github.com/async-graphql/async-graphql/tree/master/integrations/axum)
//!
//! ## License
//!
//! Licensed under either of
//!
//! * Apache License, Version 2.0,
//! (./LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
//! * MIT license (./LICENSE-MIT or <http://opensource.org/licenses/MIT>)
//! at your option.
//!
//! ## References
//!
//! * [GraphQL](https://graphql.org)
//! * [GraphQL Multipart Request](https://github.com/jaydenseric/graphql-multipart-request-spec)
//! * [GraphQL Cursor Connections Specification](https://facebook.github.io/relay/graphql/connections.htm)
//! * [GraphQL over WebSocket Protocol](https://github.com/apollographql/subscriptions-transport-ws/blob/master/PROTOCOL.md)
//! * [Apollo Tracing](https://github.com/apollographql/apollo-tracing)
//! * [Apollo Federation](https://www.apollographql.com/docs/apollo-server/federation/introduction)
//!
//! ## Examples
//!
//! All examples are in the [sub-repository](https://github.com/async-graphql/examples), located in the examples directory.
//!
//! **Run an example:**
//!
//! ```shell
//! git submodule update # update the examples repo
//! cd examples && cargo run --bin [name]
//! ```
//!
//! ## Benchmarks
//!
//! Ensure that there is no CPU-heavy process in background!
//!
//! ```shell script
//! cd benchmark
//! cargo bench
//! ```
//!
//! Now a HTML report is available at `benchmark/target/criterion/report`.
//!

#![deny(clippy::all)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::match_wildcard_for_single_variants)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::future_not_send)]
#![allow(clippy::if_not_else)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::map_flatten)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::panic)]
#![allow(clippy::redundant_else)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unused_self)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::use_self)]
#![allow(clippy::useless_let_if_seq)]
#![allow(elided_lifetimes_in_paths)]
#![recursion_limit = "256"]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod base;
mod custom_directive;
mod error;
mod guard;
mod look_ahead;
#[doc(hidden)]
pub mod model;
mod request;
mod response;
mod schema;
mod subscription;
pub mod validation;

pub mod auth;
pub mod context;
#[cfg(feature = "dataloader")]
#[cfg_attr(docsrs, doc(cfg(feature = "dataloader")))]
pub mod dataloader;
pub mod extensions;
pub mod http;
pub mod resolver_utils;
pub mod types;
#[doc(hidden)]
pub mod validators;

#[doc(hidden)]
pub mod registry;

#[doc(hidden)]
pub use async_stream;
#[doc(hidden)]
pub use async_trait;
#[doc(hidden)]
pub use context::ContextSelectionSet;
#[doc(hidden)]
pub use futures_util;
#[doc(hidden)]
pub use indexmap;
#[doc(hidden)]
pub use static_assertions;
#[doc(hidden)]
pub use subscription::SubscriptionType;

pub use auth::*;
pub use base::{
    ComplexObject, Description, InputObjectType, InputType, InterfaceType, ObjectType, OutputType,
    UnionType,
};
pub use custom_directive::{CustomDirective, CustomDirectiveFactory};
pub use dynaql_parser as parser;
pub use dynaql_value::{
    from_value, to_value, value, ConstValue as Value, DeserializerError, Name, Number,
    SerializerError, Variables,
};
pub use error::{
    Error, ErrorExtensionValues, ErrorExtensions, InputValueError, InputValueResult,
    ParseRequestError, PathSegment, Result, ResultExt, ServerError, ServerResult,
};
pub use extensions::ResolveFut;
pub use guard::{Guard, GuardExt};
pub use look_ahead::Lookahead;
pub use registry::CacheControl;
pub use request::{BatchRequest, Request};
#[doc(no_inline)]
pub use resolver_utils::{ContainerType, EnumType, ScalarType};
pub use response::{BatchResponse, Response};
pub use schema::{Schema, SchemaBuilder, SchemaEnv};
pub use validation::{ValidationMode, ValidationResult, VisitorContext};
pub use validators::CustomValidator;

pub use context::*;
#[doc(no_inline)]
pub use parser::{Pos, Positioned};
pub use types::*;

/// An alias of [dynaql::Error](struct.Error.html). Present for backward compatibility
/// reasons.
pub type FieldError = Error;

/// An alias of [dynaql::Result](type.Result.html). Present for backward compatibility
/// reasons.
pub type FieldResult<T> = Result<T>;

#[doc = include_str!("docs/complex_object.md")]
pub use dynaql_derive::ComplexObject;
#[doc = include_str!("docs/description.md")]
pub use dynaql_derive::Description;
#[doc = include_str!("docs/directive.md")]
pub use dynaql_derive::Directive;
#[doc = include_str!("docs/enum.md")]
pub use dynaql_derive::Enum;
#[doc = include_str!("docs/input_object.md")]
pub use dynaql_derive::InputObject;
#[doc = include_str!("docs/interface.md")]
pub use dynaql_derive::Interface;
#[doc = include_str!("docs/merged_object.md")]
pub use dynaql_derive::MergedObject;
#[doc = include_str!("docs/merged_subscription.md")]
pub use dynaql_derive::MergedSubscription;
#[doc = include_str!("docs/newtype.md")]
pub use dynaql_derive::NewType;
#[doc = include_str!("docs/object.md")]
pub use dynaql_derive::Object;
#[cfg(feature = "unstable_oneof")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable_oneof")))]
#[doc = include_str!("docs/oneof_object.md")]
pub use dynaql_derive::OneofObject;
#[doc = include_str!("docs/scalar.md")]
pub use dynaql_derive::Scalar;
#[doc = include_str!("docs/simple_object.md")]
pub use dynaql_derive::SimpleObject;
#[doc = include_str!("docs/subscription.md")]
pub use dynaql_derive::Subscription;
#[doc = include_str!("docs/union.md")]
pub use dynaql_derive::Union;
