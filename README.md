# Stain - Trait-based Rust Plugins

Stain was first implemented as a "simple" declarative macro that
used trait objects (e.g. `Box<dyn Traits>`) and stored them
using the `inventory` crate.

The goal was: Define some trait, implement that trait, and
then call all (or some) implementations of that trait in an
ordered fashion. This makes calling implementors as simple as
iterating over wherever the trait objects are stored and then
calling their object-safe methods. And implementors don't need
to worry about going to wherever this call is done, they just
need to register their implementation.

To be clear, `inventory` alone is already great for this,
but there's a lot of boilerplate when using `inventory` for
trait objects. A lot of types also can't be `const`-evaluated,
so you end up needing something like `LazyLock` or OnceCell.
So, I wanted registration to be dead simple, because I
didn't want to put that boilerplate burden on implementors
of the trait.

I ran into limitations with declarative macros, so I decided to
implement an actual proc-macro crate. With proc macros, it's also
easier to support traits with generics and associated types.

## Example

For the simple case (a trait with no generics or associated types):

```rust
#[stain::create_stain]
trait Discover {
    fn config(&mut self);
    fn discover(&self);
}

#[derive(Default)]
struct DiscoverHTTP(u8);
impl Discover for DiscoverHTTP {
    fn config(&mut self) {
        self.0 += 1;
    }

    fn discover(&self) {
        println!("Hello, world: {}", self.0);
    }
}

discover_stain!(DiscoverHTTP);

fn main() {
    let store = Store::<DiscoverEntry>::collect();

    for discover_method in store.iter_mut() {
        discover_method.config();
    }

    for discover_method in store.iter() {
        discover_method.discover(); // Hello, world: 1
    }
}
```

Here, we create the trait, `Discover`, and then implement that trait
for the `DiscoverHTTP` struct. We use the `#[stain::create_stain]` macro
on the `Discover` trait to generate `DiscoverEntry` and `discover_stain!(...)`.
We derive `Default` for `DiscoverHTTP`, as the `Default` implementation
is used for initializing the trait object.

The `Store` acts as a friendly interface to `inventory::collect`
and allows you to iterate (mutably and immutably) over the registered
`Discover` implementations.

`stain` also supports traits with generics and associated types:

```rust
#[stain::create_stain]
#[alias("json", Body = Json, Response)] // (name, <associated constants>, <generics>)
trait Handler <R> {
    type Body; // Associated type...

    fn handle(&self, body: Self::Body) -> R;
}

#[derive(Default)]
struct DummyHandler;
impl Handler<Response> for DummyHandler {
    type Body = Json;

    fn handle(&self, body: Json) -> Response {
        // ...
    }
}

handler_json_stain!(DummyHandler);

fn main () {
    let store = Store::<HandlerJsonEntry>::collect();

    ...
}
```

Here, we use the `alias` attribute supported by the `create_stain`
macro to specify the associated constants and generic parameters.
You can use `alias` multiple times to specify multiple variants of
your trait.

You can also specify `concrete` (`#[stain::create_stain(concrete)]`)
to enable the ability to get _specific_ implementations from the store:

```rust
let discover_http = store.get::<DiscoverHTTP>();
let discover_http = store.get_mut::<DiscoverHTTP>();
```

Behind the scenes, it adds the `stain::AsAny` super-trait to your
trait and implements `AsAny` for registered implementations.
