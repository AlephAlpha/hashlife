用 Rust 写个 HashLife 玩玩。

玩过才知道，HashLife 算法中最费时间的部分在 HashMap。所以不要用 Rust 自带的 Hash 算法，可以用 [rustc-hash](https://crates.io/crates/rustc-hash) 这个 crate 里的 `FxHashMap`。