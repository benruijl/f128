use f128_derive::*;
use ffi;
use ffi::*;
use libc::c_int;
use num_traits::*;
use std::convert::{From, Into};
use std::ffi::CString;
use std::ffi::NulError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::iter::*;
use std::mem;
use std::num::FpCategory;
use std::ops::*;
use std::slice;
use std::str;

macro_rules! f128_from_x {
    ($x: ty, $n: expr, $it: expr) => {{
        // 32 is ascii space, so this buff will be filled with spaces after the number
        let mut buf: [u8; $n] = [32u8; $n];
        write!(&mut buf[..], "{}", $it).expect("Failed to write integer to buffer");
        f128::parse(str::from_utf8(&buf[..]).unwrap()).ok()
    }};
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct f128(pub(crate) [u8; 16]);

impl f128 {
    pub const RADIX: u32 = 128;
    pub const MANTISSA_DIGITS: u32 = 112;

    pub const MAX_10_EXP: u32 = 4932;
    pub const MAX_EXP: u32 = 16383;
    pub const MIN_10_EXP: i32 = -4931;
    pub const MIN_EXP: i32 = -16382;
    pub const ZERO: f128 = f128([0; 16]);

    #[cfg(target_endian = "big")]
    pub const SIGN_BIT: f128 = f128([
        0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);
    #[cfg(target_endian = "little")]
    pub const SIGN_BIT: f128 = f128([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80,
    ]);

    #[cfg(target_endian = "big")]
    pub const EXPONENT_BITS: f128 = f128([
        0x7f, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);
    #[cfg(target_endian = "little")]
    pub const EXPONENT_BITS: f128 = f128([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff,
        0x7f,
    ]);

    #[cfg(target_endian = "big")]
    pub const FRACTION_BITS: f128 = f128([
        0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF,
    ]);
    #[cfg(target_endian = "little")]
    pub const FRACTION_BITS: f128 = f128([
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
        0x00,
    ]);

    #[cfg(target_endian = "big")]
    pub const MIN: f128 = f128([
        0xff, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff,
    ]);
    #[cfg(target_endian = "little")]
    pub const MIN: f128 = f128([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
        0xff,
    ]);

    #[cfg(target_endian = "big")]
    pub const MIN_POSITIVE: f128 = f128([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]);
    #[cfg(target_endian = "little")]
    pub const MIN_POSITIVE: f128 = f128([
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);

    #[cfg(target_endian = "big")]
    pub const ONE: f128 = f128([
        0x3f, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);
    #[cfg(target_endian = "little")]
    pub const ONE: f128 = f128([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff,
        0x3f,
    ]);

    #[cfg(target_endian = "big")]
    pub const TWO: f128 = f128([
        0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ]);
    #[cfg(target_endian = "little")]
    pub const TWO: f128 = f128([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x40,
    ]);

    #[cfg(target_endian = "big")]
    pub const E: f128 = f128([
        0x40, 0x00, 0x5b, 0xf0, 0xa8, 0xb1, 0x45, 0x76, 0x95, 0x35, 0x5f, 0xb8, 0xac, 0x40, 0x4e,
        0x7a,
    ]);
    #[cfg(target_endian = "little")]
    pub const E: f128 = f128([
        0x7a, 0x4e, 0x40, 0xac, 0xb8, 0x5f, 0x35, 0x95, 0x76, 0x45, 0xb1, 0xa8, 0xf0, 0x5b, 0x00,
        0x40,
    ]);

    #[cfg(target_endian = "big")]
    pub const PI: f128 = f128([
        0x40, 0x00, 0x92, 0x1f, 0xb5, 0x44, 0x42, 0xd1, 0x84, 0x69, 0x89, 0x8c, 0xc5, 0x17, 0x01,
        0xb8,
    ]);
    #[cfg(target_endian = "little")]
    pub const PI: f128 = f128([
        0xb8, 0x01, 0x17, 0xc5, 0x8c, 0x89, 0x69, 0x84, 0xd1, 0x42, 0x44, 0xb5, 0x1f, 0x92, 0x00,
        0x40,
    ]);

    #[cfg(target_endian = "little")]
    pub const INFINITY: f128 = f128([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0x7f]);
    #[cfg(target_endian = "big")]
    pub const INFINITY: f128 = f128([0x7F, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    #[cfg(target_endian = "big")]
    pub const NAN: f128 = f128([
        0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFFu8,
        0xFFu8, 0xFF,
    ]);
    #[cfg(target_endian = "little")]
    pub const NAN: f128 = f128([
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFFu8,
        0xFFu8, 0x7F,
    ]);

    #[cfg(target_endian = "little")]
    pub const NEG_INFINITY: f128 = f128([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff]);
    #[cfg(target_endian = "big")]
    pub const NEG_INFINITY: f128 = f128([0xff, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    #[cfg(target_endian = "little")]
    pub const EPSILON: f128 = f128([
        0x65, 0x64, 0xD2, 0x5B, 0x93, 0xF2, 0x61, 0x43, 0, 0x51, 0x2F, 0x7F, 0x8A, 0, 0x8F, 0x3F,
    ]);
    #[cfg(target_endian = "big")]
    pub const EPSILON: f128 = f128([
        0x3F, 0x8F, 0x0, 0x8A, 0x7F, 0x2F, 0x51, 0x0, 0x43, 0x61, 0xF2, 0x93, 0x5B, 0xD2, 0x64,
        0x65,
    ]);

    #[cfg(target_endian = "little")]
    pub const NEG_ZERO: f128 = f128([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80]);
    #[cfg(target_endian = "big")]
    pub const NEG_ZERO: f128 = f128([0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    #[cfg(target_endian = "little")]
    pub const MAX: f128 = f128([
        0x7f, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff,
    ]);
    #[cfg(target_endian = "big")]
    pub const MAX: f128 = f128([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
        0x7f,
    ]);

    pub(crate) fn from_arr(d: [u8; 16]) -> Self {
        f128(d)
    }

    #[inline(always)]
    pub(crate) fn from_raw_u128(d: u128) -> Self {
        f128::from_arr(unsafe { mem::transmute::<u128, [u8; 16]>(d) })
    }
    #[inline(always)]
    pub(crate) fn from_raw_i128(d: i128) -> Self {
        f128::from_arr(unsafe { mem::transmute::<i128, [u8; 16]>(d) })
    }

    #[inline(always)]
    pub(crate) fn inner_as_i128(self) -> i128 {
        unsafe { mem::transmute::<[u8; 16], i128>(self.0) }
    }

    #[inline(always)]
    pub(crate) fn inner_as_u128(&self) -> u128 {
        unsafe { mem::transmute::<[u8; 16], u128>(self.0) }
    }

    #[inline(always)]
    pub fn new<T: Into<f128>>(a: T) -> Self {
        a.into()
    }

    #[inline]
    pub fn to_string(&self) -> String {
        self.to_string_fmt("%.36Qg").unwrap()
    }

    pub fn to_string_fmt<T: AsRef<str>>(&self, fmt: T) -> Option<String> {
        let mut buf = [0u8; 128];
        let cstr;
        match CString::new(fmt.as_ref()) {
            Ok(e) => cstr = e,
            Err(_) => return None,
        };
        let n = unsafe { qtostr((&mut buf).as_mut_ptr(), 128, cstr.as_ptr(), self.clone()) };
        let mut v = Vec::with_capacity(n as usize);
        for i in 0..n {
            v.push(buf[i as usize]);
        }
        Some(String::from_utf8(v).unwrap())
    }

    #[inline(always)]
    pub fn inner(&self) -> [u8; 16] {
        self.0.clone()
    }

    #[inline(always)]
    pub fn into_inner(self) -> [u8; 16] {
        self.0
    }

    pub fn parse<T: AsRef<str>>(s: T) -> Result<Self, NulError> {
        let cstr = CString::new(s.as_ref())?;
        let result = unsafe { strtoflt128_f(cstr.as_ptr()) };

        Ok(unsafe { strtoflt128_f(cstr.as_ptr()) })
    }

    #[inline]
    pub fn exp_bits(&self) -> u32 {
        let exp_bits = f128::EXPONENT_BITS.inner_as_u128();
        ((self.inner_as_u128() & exp_bits) >> 112) as u32
    }

    #[inline]
    pub fn fract_bits(&self) -> u128 {
        self.inner_as_u128() & f128::FRACTION_BITS.inner_as_u128()
    }
}

impl Default for f128 {
    #[inline]
    fn default() -> f128 {
        f128::ZERO
    }
}

impl fmt::Display for f128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME: use actual format string and do not
        // allocate a string
        write!(f, "{}", self.to_string())
    }
}

impl fmt::LowerExp for f128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME: use actual format string and do not
        // allocate a string
        write!(f, "{}", self.to_string_fmt("%.36Qe").unwrap())
    }
}

impl Zero for f128 {
    #[inline]
    fn is_zero(&self) -> bool {
        self.0 == f128::ZERO.0
    }
    #[inline]
    fn zero() -> Self {
        f128::ZERO
    }
}

impl One for f128 {
    #[inline]
    fn one() -> Self {
        f128::ONE
    }
}

impl ToPrimitive for f128 {
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        Some(unsafe { f128_to_i64(*self) })
    }
    #[inline]
    fn to_u64(&self) -> Option<u64> {
        Some(unsafe { f128_to_u64(*self) })
    }
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        Some(unsafe { f128_to_i64(*self) as isize })
    }
    #[inline]
    fn to_i8(&self) -> Option<i8> {
        Some(unsafe { f128_to_i8(*self) })
    }
    #[inline]
    fn to_i16(&self) -> Option<i16> {
        Some(unsafe { f128_to_i16(*self) })
    }
    #[inline]
    fn to_i32(&self) -> Option<i32> {
        Some(unsafe { f128_to_i32(*self) })
    }
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        Some(unsafe { f128_to_u64(*self) as usize })
    }
    #[inline]
    fn to_u8(&self) -> Option<u8> {
        Some(unsafe { f128_to_u8(*self) })
    }
    #[inline]
    fn to_u16(&self) -> Option<u16> {
        Some(unsafe { f128_to_u16(*self) })
    }
    #[inline]
    fn to_u32(&self) -> Option<u32> {
        Some(unsafe { f128_to_u32(*self) })
    }
    #[inline]
    fn to_f32(&self) -> Option<f32> {
        Some(unsafe { f128_to_f32(*self) })
    }
    #[inline]
    fn to_f64(&self) -> Option<f64> {
        Some(unsafe { f128_to_f64(*self) })
    }
    #[inline]
    fn to_i128(&self) -> Option<i128> {
        Some(unsafe { f128_to_i128(*self) })
    }
    #[inline]
    fn to_u128(&self) -> Option<u128> {
        Some(unsafe { f128_to_u128(*self) })
    }
}

impl FromPrimitive for f128 {
    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        Some(unsafe { i64_to_f128(n) })
    }
    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        Some(unsafe { u64_to_f128(n) })
    }
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        Some(unsafe { isize_to_f128(n) })
    }
    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        Some(unsafe { i8_to_f128(n) })
    }
    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        Some(unsafe { i16_to_f128(n) })
    }
    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        Some(unsafe { i32_to_f128(n) })
    }
    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        Some(unsafe { usize_to_f128(n) })
    }
    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        Some(unsafe { u8_to_f128(n) })
    }
    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        Some(unsafe { u16_to_f128(n) })
    }
    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        Some(unsafe { u32_to_f128(n) })
    }
    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        Some(unsafe { f32_to_f128(n) })
    }
    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        Some(unsafe { f64_to_f128(n) })
    }
    #[inline]
    fn from_u128(n: u128) -> Option<Self> {
        Some(unsafe { u128_to_f128(mem::transmute(n)) })
    }
    #[inline]
    fn from_i128(n: i128) -> Option<Self> {
        Some(unsafe { i128_to_f128(mem::transmute(n)) })
    }
}

impl Num for f128 {
    type FromStrRadixErr = ();
    #[inline]
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, ()> {
        unimplemented!()
    }
}

impl NumCast for f128 {
    #[inline]
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        let int_comp = f128::from_i128(n.to_i128().expect("This should never happen."))
            .expect("This shouldnt happen either.");
        let frac_comp = f128::from_f64(n.to_f64().expect("This also shouldnt happen either."))
            .expect("This shouldnt happen.");
        Some((frac_comp - int_comp) + int_comp)
    }
}

impl Inv for f128 {
    type Output = Self;

    #[inline]
    fn inv(self) -> Self {
        self.recip()
    }
}

impl Signed for f128 {
    #[inline]
    fn abs(&self) -> Self {
        <Self as Float>::abs(*self)
    }

    #[inline]
    fn abs_sub(&self, rhs: &Self) -> Self {
        <Self as Float>::abs_sub(*self, *rhs)
    }

    #[inline]
    fn signum(&self) -> Self {
        <Self as Float>::signum(*self)
    }

    #[inline]
    fn is_positive(&self) -> bool {
        self.is_sign_positive()
    }

    #[inline]
    fn is_negative(&self) -> bool {
        self.is_sign_negative()
    }
}

impl FloatConst for f128 {
    fn E() -> Self {
        f128::E
    }
    fn FRAC_1_PI() -> Self {
        f128::PI.inv()
    }
    fn FRAC_1_SQRT_2() -> Self {
        unimplemented!();
    }
    fn FRAC_2_PI() -> Self {
        f128::new(2.0) / f128::PI
    }
    fn FRAC_2_SQRT_PI() -> Self {
        unimplemented!();
    }
    fn FRAC_PI_2() -> Self {
        f128::PI / f128::new(2.0)
    }
    fn FRAC_PI_3() -> Self {
        f128::PI / f128::new(3.0)
    }
    fn FRAC_PI_4() -> Self {
        f128::PI / f128::new(4.0)
    }
    fn FRAC_PI_6() -> Self {
        f128::PI / f128::new(6.0)
    }
    fn FRAC_PI_8() -> Self {
        f128::PI / f128::new(8.0)
    }
    fn LN_10() -> Self {
        unimplemented!();
    }
    fn LN_2() -> Self {
        unimplemented!();
    }
    fn LOG10_E() -> Self {
        unimplemented!();
    }
    fn LOG2_E() -> Self {
        unimplemented!();
    }
    fn PI() -> Self {
        f128::PI
    }
    fn SQRT_2() -> Self {
        unimplemented!();
    }
}

impl num_traits::float::FloatCore for f128 {
    #[inline]
    fn epsilon() -> Self {
        f128::EPSILON
    }

    #[inline]
    fn to_degrees(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn to_radians(self) -> Self {
        unimplemented!()
    }

    #[inline]
    fn nan() -> Self {
        f128::NAN
    }

    #[inline]
    fn infinity() -> Self {
        f128::INFINITY
    }

    #[inline]
    fn neg_infinity() -> Self {
        f128::NEG_INFINITY
    }

    #[inline]
    fn neg_zero() -> Self {
        f128::NEG_ZERO
    }

    #[inline]
    fn min_value() -> f128 {
        f128::MIN
    }

    #[inline]
    fn max_value() -> f128 {
        f128::MAX
    }

    #[inline]
    fn min_positive_value() -> f128 {
        f128::MIN_POSITIVE
    }

    #[inline]
    fn is_finite(self) -> bool {
        Float::is_finite(self)
    }

    #[inline]
    fn is_infinite(self) -> bool {
        Float::is_infinite(self)
    }

    #[inline]
    fn is_nan(self) -> bool {
        Float::is_nan(self)
    }

    #[inline]
    fn is_normal(self) -> bool {
        Float::is_normal(self)
    }

    #[inline]
    fn classify(self) -> FpCategory {
        Float::classify(self)
    }

    #[inline]
    fn floor(self) -> Self {
        Float::floor(self)
    }

    #[inline]
    fn ceil(self) -> Self {
        Float::ceil(self)
    }

    #[inline]
    fn round(self) -> Self {
        Float::round(self)
    }

    #[inline]
    fn trunc(self) -> Self {
        Float::trunc(self)
    }

    #[inline]
    fn fract(self) -> Self {
        Float::fract(self)
    }

    #[cfg(target_endian = "big")]
    #[inline]
    fn abs(mut self) -> Self {
        Float::abs(self)
    }
    #[cfg(target_endian = "little")]
    #[inline]
    fn abs(mut self) -> Self {
        Float::abs(self)
    }

    #[inline]
    fn signum(self) -> Self {
        Float::signum(self)
    }

    #[inline]
    fn is_sign_negative(self) -> bool {
        Float::is_sign_negative(self)
    }

    #[inline]
    fn is_sign_positive(self) -> bool {
        Float::is_sign_positive(self)
    }

    #[inline]
    fn recip(self) -> f128 {
        f128::ONE / self
    }

    #[inline]
    fn powi(self, n: i32) -> f128 {
        Float::powi(self, n)
    }

    #[inline]
    fn max(self, other: f128) -> f128 {
        Float::max(self, other)
    }

    #[inline]
    fn min(self, other: f128) -> f128 {
        Float::min(self, other)
    }

    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        unimplemented!("This function cannot be accurately implemented with num v0.2.6 - the mantissa type needs to be upped to u128.")
    }
}

impl Float for f128 {
    #[inline]
    fn nan() -> Self {
        f128::NAN
    }

    #[inline]
    fn infinity() -> Self {
        f128::INFINITY
    }

    #[inline]
    fn neg_infinity() -> Self {
        f128::NEG_INFINITY
    }

    #[inline]
    fn neg_zero() -> Self {
        f128::NEG_ZERO
    }

    #[inline]
    fn min_value() -> f128 {
        f128::MIN
    }

    #[inline]
    fn max_value() -> f128 {
        f128::MAX
    }

    #[inline]
    fn min_positive_value() -> f128 {
        f128::MIN_POSITIVE
    }

    #[inline]
    fn is_finite(self) -> bool {
        !self.is_infinite() && !self.is_nan()
    }

    #[inline]
    fn is_infinite(self) -> bool {
        // It's fine to compare the bits here since there is only 1 bit pattern that is inf, and one
        // that is -inf.
        let res = (self.inner_as_u128() & 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128);
        res == f128::EXPONENT_BITS.inner_as_u128()
    }

    #[inline]
    fn is_nan(self) -> bool {
        (self.inner_as_u128() & 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128)
            > f128::EXPONENT_BITS.inner_as_u128()
    }

    #[inline]
    fn is_normal(self) -> bool {
        let exp = self.exp_bits();
        exp >= 0x0001u32 && exp <= 0x7FFEu32
    }

    #[inline]
    fn classify(self) -> FpCategory {
        let x = (self.is_normal(), self.is_finite(), self.is_nan());
        match x {
            (true, true, false) => FpCategory::Normal,
            (false, true, false) => FpCategory::Subnormal,
            (_, _, true) => FpCategory::Nan,
            (_, false, _) => FpCategory::Infinite,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn floor(self) -> Self {
        unsafe { floorq_f(self) }
    }

    #[inline]
    fn ceil(self) -> Self {
        unsafe { ceilq_f(self) }
    }

    #[inline]
    fn round(self) -> Self {
        unsafe { roundq_f(self) }
    }

    #[inline]
    fn trunc(self) -> Self {
        unsafe { truncq_f(self) }
    }

    #[inline]
    fn fract(self) -> Self {
        let mut x: c_int = 0;
        unsafe { frexpq_f(self, &mut x) }
    }

    #[cfg(target_endian = "big")]
    #[inline]
    fn abs(mut self) -> Self {
        self.0[0] &= 0x7F;
        self
    }
    #[cfg(target_endian = "little")]
    #[inline]
    fn abs(mut self) -> Self {
        self.0[15] &= 0x7F;
        self
    }

    #[inline]
    fn signum(self) -> Self {
        if self == Self::NAN {
            return self;
        } else {
            if self.is_sign_positive() {
                Self::ONE
            } else {
                -Self::ONE
            }
        }
    }

    #[cfg(target_endian = "big")]
    #[inline]
    fn is_sign_negative(self) -> bool {
        match self.0[0] & 0x80 {
            0 => true,
            0x80 => false,
            _ => unreachable!(),
        }
    }

    #[cfg(target_endian = "little")]
    #[inline]
    fn is_sign_negative(self) -> bool {
        match self.0[15] & 0x80 {
            0 => false,
            0x80 => true,
            _ => unreachable!(),
        }
    }

    #[cfg(target_endian = "big")]
    #[inline]
    fn is_sign_positive(self) -> bool {
        match self.0[0] & 0x80 {
            0 => true,
            0x80 => false,
            _ => unreachable!(),
        }
    }

    #[cfg(target_endian = "little")]
    #[inline]
    fn is_sign_positive(self) -> bool {
        match self.0[15] & 0x80 {
            0 => true,
            0x80 => false,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn mul_add(self, a: f128, b: f128) -> f128 {
        unsafe { fmaq_f(self, a, b) }
    }

    #[inline]
    fn recip(self) -> f128 {
        f128::ONE / self
    }

    #[inline]
    fn powi(self, n: i32) -> f128 {
        let mut i = self.clone();
        if n == 0 {
            return f128::ONE;
        };
        if (n < 0) {
            for _ in n as i64 - 1..0 {
                i /= self;
            }
        } else {
            for _ in 1..n {
                i *= self;
            }
        }
        i
    }

    #[inline]
    fn powf(self, n: f128) -> f128 {
        unsafe { powq_f(self, n) }
    }

    #[inline]
    fn sqrt(self) -> f128 {
        unsafe { sqrtq_f(self) }
    }

    #[inline]
    fn exp(self) -> f128 {
        unsafe { expq_f(self) }
    }

    #[inline]
    fn exp2(self) -> f128 {
        // TODO: Change this to a constant two
        (f128::ONE * f128::from_u8(2).unwrap()).powf(self)
    }

    #[inline]
    fn ln(self) -> f128 {
        unsafe { logq_f(self) }
    }

    #[inline]
    fn log(self, base: f128) -> f128 {
        // Change of base formula
        let numr = self.ln();
        let denm = base.ln();
        numr / denm
    }

    #[inline]
    fn log2(self) -> f128 {
        unsafe { log2q_f(self) }
    }

    #[inline]
    fn log10(self) -> f128 {
        unsafe { log10q_f(self) }
    }

    #[inline]
    fn max(self, other: f128) -> f128 {
        unsafe {
            let a = mem::transmute::<f128, i128>(self);
            let b = mem::transmute::<f128, i128>(other);
            mem::transmute::<i128, f128>(if a > b { a } else { b })
        }
    }

    #[inline]
    fn min(self, other: f128) -> f128 {
        unsafe {
            let a = mem::transmute::<f128, i128>(self);
            let b = mem::transmute::<f128, i128>(other);
            mem::transmute::<i128, f128>(if a > b { b } else { a })
        }
    }

    #[inline]
    fn abs_sub(self, other: f128) -> f128 {
        (self - other).abs()
    }

    #[inline]
    fn cbrt(self) -> f128 {
        unsafe { cbrtq_f(self) }
    }

    #[inline]
    fn hypot(self, other: f128) -> f128 {
        unsafe { hypotq_f(self, other) }
    }

    #[inline]
    fn sin(self) -> f128 {
        unsafe { sinq_f(self) }
    }

    #[inline]
    fn cos(self) -> f128 {
        unsafe { cosq_f(self) }
    }

    #[inline]
    fn tan(self) -> f128 {
        unsafe { tanq_f(self) }
    }

    #[inline]
    fn asin(self) -> f128 {
        unsafe { asinq_f(self) }
    }

    #[inline]
    fn acos(self) -> f128 {
        unsafe { acosq_f(self) }
    }

    #[inline]
    fn atan(self) -> f128 {
        unsafe { atanq_f(self) }
    }

    #[inline]
    fn atan2(self, other: f128) -> f128 {
        unsafe { atan2q_f(self, other) }
    }

    #[inline]
    fn sin_cos(self) -> (f128, f128) {
        (self.sin(), self.cos())
    }

    #[inline]
    fn exp_m1(self) -> f128 {
        unsafe { expm1q_f(self) }
    }

    #[inline]
    fn ln_1p(self) -> f128 {
        unsafe { log1pq_f(self) }
    }

    #[inline]
    fn sinh(self) -> f128 {
        unsafe { sinhq_f(self) }
    }

    #[inline]
    fn cosh(self) -> f128 {
        unsafe { coshq_f(self) }
    }

    #[inline]
    fn tanh(self) -> f128 {
        unsafe { tanhq_f(self) }
    }

    #[inline]
    fn asinh(self) -> f128 {
        unsafe { asinhq_f(self) }
    }

    #[inline]
    fn acosh(self) -> f128 {
        unsafe { acoshq_f(self) }
    }

    #[inline]
    fn atanh(self) -> f128 {
        unsafe { atanhq_f(self) }
    }

    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        unimplemented!("This function cannot be accurately implemented with num v0.2.6 - the mantissa type needs to be upped to u128.")
    }
}
