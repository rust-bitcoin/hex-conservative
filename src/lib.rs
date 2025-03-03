// SPDX-License-Identifier: CC0-1.0

//! Hex encoding and decoding.
//!
//! General purpose hex encoding/decoding library with a conservative MSRV and dependency policy.
//!
//! ## Stabalization strategy
//!
//! In an effort to release stable 1.0 crates that are forward compatible we are striving
//! relentlessly to release the bare minimum amount of code.

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
// Experimental features we need.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// Coding conventions
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[doc(hidden)]
pub mod _export {
    /// A re-export of core::*
    pub mod _core {
        pub use core::*;
    }
}
