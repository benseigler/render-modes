/*!
Implementation of xpans' Stereo rendering mode
*/
#![no_std]
use core::marker::PhantomData;
pub mod pan_law;
use nalgebra::{SimdRealField, Vector3};
use num::{Float, FromPrimitive};
use pan_law::PanLaw;
use xpans::{Extent, Position};
use xpans_common_lr::{FlipSign, gain};
use xpans_render::prelude::*;

/**
Interprets audio sources based on their position within the scene.

Positional stereo is more physically accurate in typical stereo speaker
setups as opposed to headphones.
*/
#[derive(Default, Clone, Copy)]
pub struct Positional<T> {
    phantom_data: PhantomData<T>,
}
impl<T> InterpretationLength for Positional<T> {
    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<Source, T> InterpretSource<Source> for Positional<T>
where
    T: SimdRealField + Copy,
    Source: Position<T> + Extent<T>,
{
    type Interpretation = T;

    fn interpret_source(&mut self, source: &Source, result: &mut [Self::Interpretation]) {
        let position = Vector3::new(source.pos_x(), source.pos_y(), source.pos_z());
        let extent = Vector3::new(source.ext_x(), source.ext_y(), source.ext_z());
        let bal_unclamped = (position.x.simd_abs() - extent.x)
            .simd_max(T::zero())
            .simd_copysign(position.x);
        let bal = bal_unclamped.simd_min(T::one());
        result[0] = bal;
    }
}

/**
Interprets audio sources based on their direction from the center of the scene.

Directional stereo is more physically accurate in headphones
as opposed to typical stereo speaker setups.
*/
#[derive(Default, Clone, Copy)]
pub struct Directional<T> {
    phantom_data: PhantomData<T>,
}
impl<T> InterpretationLength for Directional<T> {
    fn interpretation_length(&self) -> usize {
        1
    }
}

impl<Source, T> InterpretSource<Source> for Directional<T>
where
    T: SimdRealField + Copy,
    Source: Position<T> + Extent<T>,
{
    type Interpretation = T;

    /// NOTE: Extent has only been clumsily and incorrectly implemented.
    fn interpret_source(&mut self, source: &Source, result: &mut [Self::Interpretation]) {
        let position = Vector3::new(source.pos_x(), source.pos_y(), source.pos_z());
        let extent = Vector3::new(source.ext_x(), source.ext_y(), source.ext_z());
        let divergence = directional_cuboid_solve(position, extent);
        let mut pos_norm = position;
        pos_norm.normalize_mut();
        let converged_balance = pos_norm.x * divergence.x;
        result[0] = converged_balance;
    }
}
fn directional_cuboid_solve<T: SimdRealField + Copy>(
    position: Vector3<T>,
    extent: Vector3<T>,
) -> Vector3<T> {
    let position_abs = simd_abs(position);
    let mut plus = position_abs + extent;
    for i in plus.iter_mut() {
        i.simd_max(T::zero());
    }
    let plus = simd_abs(plus.normalize());
    let mut minus = position_abs - extent;
    for i in minus.iter_mut() {
        i.simd_max(T::zero());
    }
    let minus = simd_abs(minus.normalize());
    let mut divergence = Vector3::zeros();
    for ((plus, minus), d) in plus.iter().zip(minus.iter()).zip(divergence.iter_mut()) {
        *d = plus.simd_min(*minus);
    }
    divergence
}

fn simd_abs<T: SimdRealField>(value: Vector3<T>) -> Vector3<T> {
    let mut result = Vector3::zeros();
    for (a, b) in result.iter_mut().zip(value.iter()) {
        *a = b.simd_abs();
    }
    result
}

/// The sample processor for the stereo rendering mode.
#[derive(Default)]
pub struct Processor<T, Law> {
    pan_law: Law,
    scalar: PhantomData<T>,
}
impl<T, Law> Processor<T, Law>
where
    Law: PanLaw<T>,
{
    pub fn new(pan_law: Law) -> Self {
        Self {
            pan_law,
            scalar: PhantomData,
        }
    }

    pub fn set_pan_law(&mut self, pan_law: Law) {
        self.pan_law = pan_law;
    }
}

impl<T, Law> DelaySamples for Processor<T, Law> {
    fn delay_samples(&self, _sample_rate: u32) -> usize {
        0
    }
}
impl<T, Law> OutputChannels for Processor<T, Law> {
    fn output_channels(&self) -> usize {
        2
    }
}

impl<T, Law, In, Out> ProcessSamples<In, Out> for Processor<T, Law>
where
    Law: PanLaw<T>,
    In: Input<T>,
    Out: Output<T>,
    T: Float + FlipSign + FromPrimitive,
{
    type Interpretation = T;

    fn process_samples(&mut self, result: &[T], input: &In, output: &mut Out) {
        for channel in 0..2u8 {
            let gain = gain(channel, result[0]);
            let attenuated = self.pan_law.attenuate(gain);
            let current_sample = input.current_sample();
            output.set_channel(usize::from(channel), current_sample * attenuated);
        }
    }
}
