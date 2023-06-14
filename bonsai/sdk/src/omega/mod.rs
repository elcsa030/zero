// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![doc = include_str!("./README.md")]

pub mod client;
pub mod types;

pub use client::Client;

/// The routes for the API.
pub mod routes {
    /// Route for `MemoryImage` related APIs.
    pub const IMAGE_ROUTE: &str = "/v1/images";
    /// Route for `Session` related APIs.
    pub const SESSION_ROUTE: &str = "/v1/sessions";
    /// Route for `Receipt` related APIs.
    pub const RECEIPT_ROUTE: &str = "/v1/receipts";
}
