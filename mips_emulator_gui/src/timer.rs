use std::time::Instant;

#[allow(dead_code)]
/// A timer that steps at a set interval.
/// 
/// There are two implementations because I don't really want to use the big
/// `chrono` dependency for WASM; instead I just have a basic frame timer.
pub enum CpuTimer {
	/// Timer that steps at a set interval in microseconds, using `std::time`.
	/// Because of this, it will not work in WASM builds.
	Micro { interval: u64, last: Option<Instant>, },
	
	/// Simpler timer that steps after a set amount of ticks. Not accurate at
	/// all, but works well enough for WASM builds.
	Frames { interval: f32, left: usize, },
}
impl CpuTimer {
	/// Makes a `Micro`second timer
	#[allow(dead_code)]
	pub fn micro(interval: u64) -> Self {
		CpuTimer::Micro { interval, last: None, }
	}
	
	/// Makes a `Frame` timer
	#[allow(dead_code)]
	pub fn frames(interval: f32) -> Self {
		CpuTimer::Frames { interval, left: 0, }
	}
	
	/// Returns how many times the CPU should step.
	pub fn tick(&mut self) -> usize {
		use CpuTimer::*;
		match self {
			Micro { interval, last } => {
				let now = Instant::now();
				match last {
					None => { *last = Some(now); 0 },
					Some(last_tick) => {
						let since = now.duration_since(*last_tick);
						let times = (since.as_micros() as u64 / *interval) as usize;
						if times > 0 { *last = Some(now); }
						times
					}
				}
			},
			Frames { interval, left } => {
				if *left > 0 {
					*left -= 1; 0
				} else if *interval > 1.0 {
					*left = interval.round() as usize; 1
				} else {
					interval.recip().round() as usize
				}
			},
		}
	}
	
	/// Resets the timer. Can be called while still reset, that's fine too.
	pub fn reset(&mut self) {
		use CpuTimer::*;
		match self {
			Micro { last, .. } => *last = None,
			Frames { left, ..} => *left = 0,
		}
	}
}
