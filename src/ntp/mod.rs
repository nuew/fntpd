mod pkt;
mod timestamp;

pub use self::pkt::inner_loop;

/// NTP Port Number
pub const PORT: u16 = 123;

/// NTP Version Number
pub const VERSION: u8 = 4;

/// frequency tolerance ϕ (s/s)
pub const TOLERANCE: f32 = 15e-6;

/// minimum poll exponent (16 s)
pub const MINPOLL: u8 = 4;

/// maximum poll exponent (36 h)
pub const MAXPOLL: u8 = 17;

/// maximum dispersion (16 s)
pub const MAXDISP: u8 = 16;

/// minimum dispersion increment (s)
pub const MINDISP: f32 = 0.005;

/// distance threshold (1 s)
pub const MAXDIST: u8 = 1;

/// maximum stratum number
pub const MAXSTRAT: u8 = 16;
