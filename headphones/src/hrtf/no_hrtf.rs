use num::{Float, Zero, cast::AsPrimitive};
use xpans_render::input::{FractionalInput, SampleRate};

use crate::{Interpretation, Processor, get_delay_samples, hrtf::Hrtf};

/// No spectral filtering, only handles ITD.
#[derive(Debug, Default)]
pub struct NoHrtf;

impl NoHrtf {
    pub fn new() -> Self {
        Self
    }
}

impl<T, I, D, Law> Hrtf<T, I> for Processor<T, Law, D, NoHrtf>
where
    T: Float + Zero + 'static,
    I: FractionalInput<T, T> + SampleRate,
    u32: AsPrimitive<T>,
{
    fn get_samples(&mut self, interpretation: &Interpretation<T>, input: &I) -> [T; 2] {
        let mut samples = [T::zero(); 2];
        let delay = get_delay_samples(self.max_itd_nanos, interpretation, input);
        for (channel, sample) in samples.iter_mut().enumerate() {
            let is_delayed_channel = (channel == 0) == interpretation.lr().is_sign_positive();
            let sample_fetcher = [current, delayed][is_delayed_channel as usize];
            *sample = sample_fetcher(input, delay);
        }
        samples
    }
}

fn current<T>(input: &impl FractionalInput<T, T>, _delay: T) -> T {
    input.current_sample()
}

fn delayed<T>(input: &impl FractionalInput<T, T>, delay: T) -> T {
    input.fractional_sample(delay)
}
