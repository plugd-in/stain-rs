# stain

[<img alt="GitHub" src="https://img.shields.io/badge/github-plugd--in%2Fstain--rs-blue?style=for-the-badge&logo=github">](https://github.com/plugd-in/stain-rs)
[<img alt="Build Status" src="https://img.shields.io/github/check-runs/plugd-in/stain-rs/master?style=for-the-badge">](https://github.com/plugd-in/stain-rs/actions?query=branch%3Amaster)

**A compile-time, distributed plugin system for Rust.**

`stain` allows you to define a trait in one module and "collect" implementations of that trait from anywhere in your dependency graph—without manually registering them in your `main` function.

It is built on top of [linkme](https://github.com/dtolnay/linkme) and provides a type-safe, ergonomic layer that handles complex Trait signatures, Generics, and Associated Types (GATs).

## Features

* **Distributed Registration:** Implement traits in decoupled modules or crates; collect them automatically at link time.

* **Zero-Cost Abstraction:** Registration happens at compile-time via linker sections. Runtime overhead is limited to a one-time collection and sorting pass.

* **Rich Type Support:** First-class support for **Generics** and **Generic Associated Types (GATs)**, which are often difficult to handle in plugin registries.

* **Deterministic Ordering:** Assign priorities (e.g., `u64` or custom Enums) to ensure plugins execute in a specific order.

* **Concrete Access:** Iterate over trait objects or downcast to specific concrete structs when you need full access to the underlying type.

## Usage

Add `stain` to your `Cargo.toml`.

```toml
[dependencies]
stain = "0.1"
```

### Example

Here is a complete example showing how to define a plugin system, register a plugin, and use it.

```rust
use stain::{create_stain, stain, Store};

// 1. Define your plugin interface
pub trait Discover<T> {
    type Error;
    fn discover(&self) -> Result<T, Self::Error>;
}

// 2. Create the storage infrastructure
create_stain! {
    trait Discover;
    ordering: u64;

    // Define the signature of the trait:
    type &'static str;              // The Generic T
    trait type Error = &'static str; // The Associated Type

    store: pub mod discover_store;
}

// 3. Implement the trait normally
#[derive(Default)]
struct DiscoverLinux;

impl Discover<&'static str> for DiscoverLinux {
    type Error = &'static str;

    fn discover(&self) -> Result<&'static str, Self::Error> {
        Ok("Hello, Linux!")
    }
}

// 4. Stain (register) the implementation
stain! {
    store: discover_store;
    item: DiscoverLinux;
    ordering: 0;
}

fn main() {
    // 5. Collect and use all registered plugins
    let store = discover_store::Store::collect();

    for plugin in store.iter() {
        println!("Plugin name: {}", plugin.name());
        println!("Result: {:?}", plugin.discover());
    }
}
```

## How It Works

1. **`create_stain!`** generates a module containing a `Store` struct and defines a custom linker section using `linkme`.

2. **`stain!`** creates a static entry in that linker section. It wraps your struct's constructor in a `LazyLock`, ensuring that your plugin is only initialized (allocated) when it is first accessed.

3. **`Store::collect()`** reads the linker section at runtime, sorts the pointers based on your defined `ordering`, and prepares them for iteration.

## TODOs

[ ] Add examples.
[ ] Current MSRV is 1.91 due to TypeId::of const support.
    Add support for earlier Rust versions by working around
    the const limitation in earlier versions.
