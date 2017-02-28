use nom::be_u32;

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub struct NTPTimestamp {
  seconds: u32,
  fraction: u32,
}

named!(pub parse_timestamp<NTPTimestamp>,
  do_parse!(
    seconds: be_u32 >>
    fraction: be_u32 >>
    (NTPTimestamp {
      seconds: seconds,
      fraction: fraction
    }
  ))
);

// TODO: add some conversions to+from chrono types
