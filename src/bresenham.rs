pub type Coord = i32;
pub type Vec2 = (Coord, Coord);

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Octant {
	Octant0, Octant1, Octant2, Octant3,
	Octant4, Octant5, Octant6, Octant7,
}
impl From<u8> for Octant {
	fn from(n: u8) -> Self {
		use Octant::*;
		match n % 8 {
			0 => Octant0, 1 => Octant1,
			2 => Octant2, 3 => Octant3,
			4 => Octant4, 5 => Octant5,
			6 => Octant6, 7 => Octant7,
			_ => Octant0
		}
	}
}
impl Octant {
	pub fn from_point(delta: Vec2) -> Self {
		let mut delta = delta;
		let mut octant = 0;
		
		if delta.1 < 0 {
			delta = (-delta.0, -delta.1);
			octant += 4;
		}
		if delta.0 < 0 {
			delta = ( delta.1, -delta.0);
			octant += 2;
		}
		if delta.0 < delta.1 {
			octant += 1;
		}
		
		octant.into()
	}
	pub fn pt_into_octant0(self, p: Vec2) -> Vec2 {
		use Octant::*;
		match self {
			Octant0 => ( p.0,  p.1), Octant1 => ( p.1,  p.0),
			Octant2 => ( p.1, -p.0), Octant3 => (-p.0,  p.1),
			Octant4 => (-p.0, -p.1), Octant5 => (-p.1, -p.0),
			Octant6 => (-p.1,  p.0), Octant7 => ( p.0, -p.1),
		}
	}
	pub fn pt_from_octant0(self, p: Vec2) -> Vec2 {
		use Octant::*;
		match self {
			Octant0 => ( p.0,  p.1), Octant1 => ( p.1,  p.0),
			Octant2 => (-p.1,  p.0), Octant3 => (-p.0,  p.1),
			Octant4 => (-p.0, -p.1), Octant5 => (-p.1, -p.0),
			Octant6 => ( p.1, -p.0), Octant7 => ( p.0, -p.1),
		}
	}
}

pub struct Line {
	position: Vec2,
	delta: Vec2,
	target_x: Coord,
	diff: Coord,
	octant: Octant,
}
impl Line {
	pub fn new(a: Vec2, b: Vec2) -> Self {
		let octant = Octant::from_point((b.0 - a.0, b.1 - a.1));
		
		let a = octant.pt_into_octant0(a);
		let b = octant.pt_into_octant0(b);
		
		let delta = (b.0 - a.0, b.1 - a.1);
		let diff = delta.1 - delta.0;
		
		Line {
			position: a,
			delta,
			target_x: b.0,
			diff,
			octant
		}
	}
}

impl Iterator for Line {
	type Item = Vec2;
	
	fn next(&mut self) -> Option<Self::Item> {
		if self.position.0 > self.target_x {
			return None;
		}
		
		let p = self.position;
		
		if self.diff >= 0 {
			self.position.1 += 1;
			self.diff -= self.delta.0;
		}
		
		self.diff += self.delta.1;
		
		self.position.0 += 1;
		
		Some(self.octant.pt_from_octant0(p))
	}
}
