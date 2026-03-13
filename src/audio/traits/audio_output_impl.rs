use crate::audio::traits::AudioOutputTrait;

impl AudioOutputTrait for f32 {}
impl AudioOutputTrait for f64 {}

impl AudioOutputTrait for i8 {}
impl AudioOutputTrait for i16 {}
impl AudioOutputTrait for i32 {}
impl AudioOutputTrait for i64 {}
