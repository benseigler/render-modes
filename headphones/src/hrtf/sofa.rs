use std::ops::AddAssign;

use num::{Float, Zero, cast::AsPrimitive};
use sofar::reader::{Filter, OpenOptions, Sofar};
use xpans_render::input::{FractionalInput, SampleRate};

use crate::{Interpretation, Processor, get_delay_samples, hrtf::Hrtf};

const HRTF: &[u8] = include_bytes!("sofa/Subject1_HRIRs.sofa");

pub struct Sofa {
    sofa: Sofar,
    filter: Filter,
}

impl Sofa {
    pub fn new() -> Self {
        let sofa = OpenOptions::new()
            .sample_rate(48000.)
            .open_data(HRTF)
            .expect("SOFA err");
        let filter_len = sofa.filter_len();
        let filter = Filter::new(filter_len);
        Self {
            sofa,
            filter: filter,
        }
    }
}
impl<T, I, D, Law> Hrtf<T, I> for Processor<T, Law, D, Sofa>
where
    T: Float + Zero + AsPrimitive<f32> + AddAssign,
    I: FractionalInput<T, T> + SampleRate,
    f32: AsPrimitive<T>,
    usize: AsPrimitive<T>,
    u32: AsPrimitive<T>,
{
    fn get_samples(&mut self, interpretation: &Interpretation<T>, input: &I) -> [T; 2] {
        let mut samples = [T::zero(); 2];
        self.hrtf.sofa.filter(
            interpretation.bf.as_() * interpretation.distance.as_(),
            interpretation.lr.as_() * interpretation.distance.as_(),
            interpretation.du.as_() * interpretation.distance.as_(),
            &mut self.hrtf.filter,
        );
        let filter = &self.hrtf.filter;
        let filter = [
            filter.left.iter().as_slice(),
            filter.right.iter().as_slice(),
        ];

        let delay = get_delay_samples(self.max_itd_nanos, interpretation, input);
        for channel in 0..2 {
            let is_delayed_channel = (channel == 0) == interpretation.lr.is_sign_positive();
            let sample_fetcher = [current, delayed][is_delayed_channel as usize];

            samples[channel] = sample_fetcher(filter[channel], input, delay);
        }
        samples
    }
}

fn current<T>(filter: &[f32], input: &impl FractionalInput<T, T>, _delay: T) -> T
where
    T: Float + Zero + AddAssign + 'static,
    f32: AsPrimitive<T>,
{
    let mut sum = T::zero();
    for (i, ir) in filter.iter().rev().enumerate() {
        sum += ir.as_() * input.integer_sample(i)
    }
    sum
}

fn delayed<T>(filter: &[f32], input: &impl FractionalInput<T, T>, delay: T) -> T
where
    T: Float + Zero + AddAssign + 'static,
    f32: AsPrimitive<T>,
    usize: AsPrimitive<T>,
{
    let mut sum = T::zero();
    for (i, ir) in filter.iter().rev().enumerate() {
        sum += ir.as_() * input.fractional_sample(delay + i.as_())
    }
    sum
}
