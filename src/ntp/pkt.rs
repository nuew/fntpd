use nom::{IResult, Needed, be_u8, be_i8, be_u16, be_u32, be_i32};
use std::{io, fmt};
use std::net::UdpSocket;
use super::timestamp::{NTPTimestamp, parse_timestamp};

/// The maximum packet length that will be supported.
const MAX_PACKET_LENGTH: usize = 128;

// temporary
type FixedInt32 = i32;
type FixedUInt32 = u32;

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
/// Network Time Protocol Packet
struct NTPPacket<'a> {
  /// 2-bit field warning of leap seconds
  leap: u8,
  /// 3-bit integer describing current protocol version
  version: u8,
  /// 3-bit integer representing the mode
  mode: NTPMode,
  /// Indicates server stratum, or 0 for unspecified.
  stratum: u8,
  /// Max interval between successive messages, as exponent of 2, in seconds
  poll: u8,
  /// System clock precision, as exponent of 2, in seconds
  precision: i8,
  /// Total round-trip delay to primary reference source, in seconds.
  /// The fraction point is between bits 15 and 16.
  rootdelay: FixedInt32,
  /// Maximum error due to clock freq tolerance, in seconds.
  /// The fraction point is between bits 15 and 16.
  rootdisp: FixedUInt32,
  /// Reference ID identifying reference source.
  /// See specification for contents.
  refid: u32,
  /// Last time system clock set or corrected
  reference_timestamp: NTPTimestamp,
  /// Time when request departed client for server
  org: NTPTimestamp,
  /// Time when request arrived at server or reply arrived at client
  rec: NTPTimestamp,
  /// Time when request departed client or reply departed server
  xmt: NTPTimestamp,
  // `dst` is intentionally omitted, as it is only used on the client,
  // and never transits the network.
  /// First NTP Extension Field
  ext1: Option<NTPExt<'a>>,
  /// Second NTP Extension Field
  ext2: Option<NTPExt<'a>>,
  /// Used for NTP authentication
  mac: Option<NTPMAC<'a>>,
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq)]
/// NTP packet modes
enum NTPMode {
  Reserved = 0,
  SymmetricActive = 1,
  SymmetricPassive = 2,
  Client = 3,
  Server = 4,
  Broadcast = 5,
  NTPControl = 6,
  ReservedPrivate = 7
}

impl NTPMode {
  /// Creates a NTPMode from the NTPPacket value.
  /// 
  /// Panics if mode >= 8.
  fn new(mode: u8) -> NTPMode {
    match mode {
      0 => NTPMode::Reserved,
      1 => NTPMode::SymmetricActive,
      2 => NTPMode::SymmetricPassive,
      3 => NTPMode::Client,
      4 => NTPMode::Server,
      5 => NTPMode::Broadcast,
      6 => NTPMode::NTPControl,
      7 => NTPMode::ReservedPrivate,
      _ => panic!("Impossible NTP Mode!")
    }
  }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
/// The extension field of a NTPPacket
// perhaps move this to its own file?
struct NTPExt<'a> {
  field_type: u16,
  length: u16,
  value: &'a [u8],
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
/// The optional Message Authentication Code for a NTPPacket
struct NTPMAC<'a> {
  /// Secret MD5-key identifier
  keyid: u32,
  /// MD5 Digest
  digest: &'a [u8],
}

named!(parse<NTPPacket>,
  do_parse!(
    livemo: bits!(tuple!(
      take_bits!(u8,2), // leap
      take_bits!(u8,3), // version
      take_bits!(u8,3) // mode
    )) >>
    stratum: be_u8    >>
    poll: be_u8       >>
    precision: be_i8  >>
    rootdelay: be_i32 >>
    rootdisp: be_u32  >>
    refid: be_u32     >>
    reference_timestamp: parse_timestamp >>
    org: parse_timestamp >>
    rec: parse_timestamp >>
    xmt: parse_timestamp >>
    ext1: opt!(complete!(parse_ntpext)) >>
    ext2: opt!(complete!(parse_ntpext)) >>
    mac: opt!(complete!(parse_ntpmac)) >>
    (NTPPacket {
      leap: livemo.0,
      version: livemo.1,
      mode: NTPMode::new(livemo.2),
      stratum: stratum,
      poll: poll,
      precision: precision,
      rootdelay: rootdelay,
      rootdisp: rootdisp,
      refid: refid,
      reference_timestamp: reference_timestamp,
      org: org,
      rec: rec,
      xmt: xmt,
      ext1: ext1,
      ext2: ext2,
      mac: mac
    }
  ))
);

named!(parse_ntpext<NTPExt>,
  do_parse!(
    field_type: be_u16    >>
    length: be_u16        >>
    value: take!(length)  >>
    (NTPExt {
      field_type: field_type,
      length: length,
      value: value
    }
  ))
);

named!(parse_ntpmac<NTPMAC>,
  do_parse!(
    keyid: be_u32 >>
    digest: take!(16) >>
    (NTPMAC {
      keyid: keyid,
      digest: digest
    }
  ))
);

impl<'a> NTPPacket<'a> {
  /// Check sanity of a NTPPacket.
  fn validate<T: fmt::Display>(&self, from: T) -> bool {
    // check version
    if self.version != super::VERSION {
      warn!("Packet from {} has version {}, but our version is {}.", from, self.version, super::VERSION);
    }

    match self.mode {
      NTPMode::Client => {},
      _ => {
        error!("Packet from {} has unsupported type {:?}", from, self.mode);
        return false;
      }
    }

    // check stratum
    if self.stratum > super::MAXSTRAT {
      warn!("Packet from {} at stratum {}, which is greater than the maximum stratum of {}.", from, self.stratum, super::MAXSTRAT);
    }

    return true;
  }
}

pub fn inner_loop(socket: &UdpSocket) -> io::Result<()> {
  let mut buf = [0u8; MAX_PACKET_LENGTH];
  let (bytes, from) = socket.recv_from(&mut buf)?;
  let pkt = match parse(&buf.split_at(bytes + 1).0) {
    IResult::Done(ext, pkt) => {
      if ext.len() > 0 {
        warn!("{} bytes of extraneous data from {}", ext.len(), from);
      }

      pkt
    }
    IResult::Error(err) => panic!(err), // TODO return an error
    IResult::Incomplete(needed) => {
      match needed {
        Needed::Unknown => error!("Received incomplete packet from {}", from),
        Needed::Size(bytes) => {
          error!("Received incomplete packet (missing {} bytes) from {}", bytes, from)
        }
      }

      // TODO return an error
      panic!("should return an error here")
    }
  };
  println!("{}: {:?}", pkt.validate(from), pkt);
  Ok(())
}
