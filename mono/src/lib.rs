use std::marker::PhantomData;
use xpans_render::prelude::*;

/// The source interpreter for the mono rendering mode.
#[derive(Debug, Default)]
pub struct Interpreter;

impl InterpretationLength for Interpreter {
    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<Source> InterpretSource<Source> for Interpreter {
    type Interpretation = ();

    fn interpret_source(&mut self, _source: &Source, _result: &mut [Self::Interpretation]) {}
}

/// The sample processor for the mono rendering mode.
#[derive(Debug, Default, Clone, Copy)]
pub struct Processor<S>
where
    S: Copy,
{
    channels: usize,
    phantom_sample: PhantomData<S>,
}

impl<S> Processor<S>
where
    S: Copy,
{
    pub fn new(channels: usize) -> Self {
        Self {
            channels,
            phantom_sample: PhantomData,
        }
    }
}

impl<S> DelaySamples for Processor<S>
where
    S: Copy,
{
    fn delay_samples(&self, _sample_rate: u32) -> usize {
        0
    }
}
impl<S> OutputChannels for Processor<S>
where
    S: Copy,
{
    fn output_channels(&self) -> usize {
        self.channels
    }
}

impl<S, In, Out> ProcessSamples<In, Out> for Processor<S>
where
    In: Input<S>,
    Out: Output<S>,
    S: Copy,
{
    type Interpretation = ();

    fn process_samples(&mut self, _result: &[Self::Interpretation], input: &In, output: &mut Out) {
        let sample = input.current_sample();
        for channel in 0..self.output_channels() {
            output.set_channel(channel, sample);
        }
    }
}
