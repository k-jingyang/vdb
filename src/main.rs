#![warn(unused_extern_crates)]

// TODO: add so many mods here?
// good rust dir structure?
mod constant;
mod graph;
mod plotter;
mod vanama;
fn main() {
    vanama::init();
}
