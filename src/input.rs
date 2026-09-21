use std::collections::HashSet;
use winit::keyboard::PhysicalKey;

pub struct Input {
    keys: HashSet<PhysicalKey>,
    mouse: Option<()>, // don't know what type this is yet
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: HashSet::new(),
            mouse: None,
        }
    }
}
