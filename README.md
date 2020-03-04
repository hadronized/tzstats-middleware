# How to build

You need [rustup](https://rustup.rs). Then:

```
git clone https://github.com/phaazon/tzstats-middleware
cd tzstats-middleware
rustup override set nightly
cargo build --release
```

Run with:

```
cargo run --release
```

The binary is `target/release/tzstats-middleware`.

# How to customize

You can’t right now, as I made everything hardcoded, like a good o’ hack. :D Have a look
at the `main.rs` if you want to change port / destination. It should be possible to add
CLI with a few lines of code; contact me if you want it.
