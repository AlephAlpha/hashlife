用 Rust 写个 HashLife 玩玩。

玩过才知道，HashLife 算法中最费时间的部分在 HashMap。所以不要用 Rust 自带的 Hash 算法，可以用 [rustc-hash](https://crates.io/crates/rustc-hash) 这个 crate 里的 `FxHashMap`。

已实现垃圾回收，但每次到了需要垃圾回收的时候就会卡上一段时间。因此垃圾回收的阈值调得比较高，模拟较大的图样时通常会占用 2G 左右的内存。

`src` 中仅为算法的实现。生命游戏模拟器见 `hashlife-sdl` 文件夹。编译前请确保电脑上装上了 `sdl`（参见 [Rust SDL2](https://github.com/Rust-SDL2/rust-sdl2) 的说明）。

参考了以下项目：

- [**golly**](https://github.com/AlephAlpha/golly) 最好的生命游戏模拟器，其中 HashLife 的实现在[`hlifealgo.cpp`](https://github.com/AlephAlpha/golly/blob/master/gollybase/hlifealgo.cpp)
- [**life**](https://github.com/copy/life) 最好的网页版生命游戏模拟器，用 JavaScript 实现了 HashLife 算法
- [**smeagol**](https://github.com/billyrieger/smeagol) 另一个用 Rust 实现的 HashLife
- [**lifeash**](https://github.com/LU15W1R7H/lifeash) 又一个用 Rust 实现的 HashLife