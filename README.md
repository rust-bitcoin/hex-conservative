# A Rust hexadecimal decoding library

General purpose hex encoding/decoding library with a conservative MSRV and dependency policy.

## Stabilization strategy

Because downstream crates may need to return hex errors in their APIs and they need to be
stabilized soon, this crate only exposes the errors and two basic decoding functions. This
should already help with the vast majority of the cases and we're sufficiently confident that
these errors won't have a breaking change any time soon (possibly never).

If you're writing a binary you don't need to worry about any of this and just use the unstable
version for now. If you're writing a library you should use these stable errors in the API but
you may internally depend on the unstable crate version to get the advanced features that won't
affect your API. This way your API can stabilize before all features in this crate are fully
stable and you still can use all of them.

## Crate feature flags

* `std` - enables the standard library, on by default.
* `alloc` - enables features that require allocation such as decoding into `Vec<u8>`, implied
by `std`.
* `newer-rust-version` - enables Rust version detection and thus newer features, may add
                         dependency on a feature detection crate to reduce compile times. This
                         feature is expected to do nothing once the native detection is in Rust
                         and our MSRV is at least that version. We may also remove the feature
                         gate in 2.0 with semver trick once that happens.

## Minimum Supported Rust Version (MSRV)

This library should compile with almost any combination of features on **Rust 1.74.0**, however we
reserve the right to use features to guard compiler specific code so `--all-features` may not work
using the MSRV toolchain.

### Policy

Policy is to never use an MSRV that is less than two years old and also that ships in Debian stable.
We may bump our MSRV in a minor version, but we have no plans to. 

Note though that the dependencies may have looser policy. This is not considered
breaking/wrong - you would just need to pin them in `Cargo.lock` (not `.toml`).


## Githooks

To assist devs in catching errors _before_ running CI we provide some githooks. If you do not
already have locally configured githooks you can use the ones in this repository by running, in the
root directory of the repository:
```
git config --local core.hooksPath githooks/
```

Alternatively add symlinks in your `.git/hooks` directory to any of the githooks we provide.
