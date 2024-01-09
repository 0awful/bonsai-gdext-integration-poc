use godot::prelude::*;

pub mod example;
pub mod example_mob;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
