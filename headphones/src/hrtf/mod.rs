use xpans_render::input::{FractionalInput, SampleRate};

use crate::Interpretation;

pub mod no_hrtf;

/// Possibly applies delay, filtering, etc. on input samples
pub trait Hrtf<T, I>
where
    I: FractionalInput<T, T> + SampleRate,
{
    fn get_samples(&mut self, interpretation: &Interpretation<T>, input: &I) -> [T; 2];
}
