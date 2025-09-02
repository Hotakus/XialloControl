use crate::{adaptive_sampler, mapping, preset, setting};

pub fn initialize() {
    setting::initialize();
    preset::initialize();

    mapping::initialize();

    adaptive_sampler::initialize();
}
