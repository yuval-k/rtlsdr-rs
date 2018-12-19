mod rtlsdrsys;

#[derive(Clone, Copy, Debug)]
pub enum DirectSampling {
    Disabled = 0,
    I = 1,
    Q = 2,
}

#[derive(Clone, Copy, Debug)]
pub enum Tuner {
    UNKNOWN = rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_UNKNOWN as isize,
    E4000 = rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_E4000 as isize,
    FC0012 = rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_FC0012 as isize,
    FC0013 = rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_FC0013 as isize,
    FC2580 = rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_FC2580 as isize,
    R820T = rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_R820T as isize,
    R828D = rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_R828D as isize,
}

pub struct Device {
    dev: *mut rtlsdrsys::rtlsdr_dev_t,
}

#[derive(Clone, Copy, Debug)]
pub struct Error {
    error: std::os::raw::c_int,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error is {}", self.error)
    }
}

impl std::error::Error for Error {}

impl Error {
    fn new(error: std::os::raw::c_int) -> Self {
        Error { error: error }
    }
}

impl Drop for Device {
    #[inline(never)]
    fn drop(&mut self) {
        if !self.dev.is_null() {
            unsafe {
                rtlsdrsys::rtlsdr_close(self.dev);
            }
            self.dev = std::ptr::null_mut();
        }
    }
}

struct CbWrapper<'a> {
    cb: &'a mut dyn FnMut(&[u8]),
}

pub struct USBStrings {
    pub manufacture: String,
    pub product: String,
    pub serial: String,
}

impl Device {
    fn tosuccess(ret: std::os::raw::c_int) -> Result<(), Error> {
        match ret {
            0 => Ok(()),
            err => Err(Error::new(err)),
        }
    }

    pub fn set_xtal_freq(&mut self, rtl_freq: u32, tuner_freq: u32) -> Result<(), Error> {
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_xtal_freq(self.dev, rtl_freq, tuner_freq) })
    }

    pub fn get_xtal_freq(&mut self) -> Result<(u32, u32), Error> {
        let mut rtl_freq: u32 = 0;
        let mut tuner_freq: u32 = 0;
        match unsafe { rtlsdrsys::rtlsdr_get_xtal_freq(self.dev, &mut rtl_freq, &mut tuner_freq) } {
            0 => Ok((rtl_freq, tuner_freq)),
            err => Err(Error::new(err)),
        }
    }

    pub fn get_usb_strings(&mut self) -> Result<USBStrings, Error> {
        let mut manufacture: [std::os::raw::c_char; 256] = [0; 256];
        let mut prod: [std::os::raw::c_char; 256] = [0; 256];
        let mut serial: [std::os::raw::c_char; 256] = [0; 256];

        fn convert(buf: &[std::os::raw::c_char]) -> Result<String, Error> {
            let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
            // TODO: use failure trait to wrap the conversion error
            let str_slice: &str = c_str.to_str().map_err(|_| Error::new(-1))?;
            Ok(str_slice.to_owned())
        }

        match unsafe {
            rtlsdrsys::rtlsdr_get_usb_strings(
                self.dev,
                manufacture.as_mut_ptr(),
                prod.as_mut_ptr(),
                serial.as_mut_ptr(),
            )
        } {
            0 => Ok(USBStrings {
                manufacture: convert(&manufacture)?,
                product: convert(&prod)?,
                serial: convert(&serial)?,
            }),
            err => Err(Error::new(err)),
        }
    }

    pub fn set_center_freq(&mut self, freq: u32) -> Result<(), Error> {
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_center_freq(self.dev, freq) })
    }

    pub fn get_center_freq(&mut self) -> Result<u32, Error> {
        match unsafe { rtlsdrsys::rtlsdr_get_center_freq(self.dev) } {
            0 => Err(Error::new(0)),
            freq => Ok(freq),
        }
    }

    pub fn set_freq_correction(&mut self, ppm: isize) -> Result<(), Error> {
        Self::tosuccess(unsafe {
            rtlsdrsys::rtlsdr_set_freq_correction(self.dev, ppm as std::os::raw::c_int)
        })
    }

    pub fn get_freq_correction(&mut self) -> isize {
        unsafe { rtlsdrsys::rtlsdr_get_freq_correction(self.dev) as isize }
    }

    pub fn rtlsdr_get_tuner_type(&mut self) -> Tuner {
        let ret = unsafe { rtlsdrsys::rtlsdr_get_tuner_type(self.dev) };
        match ret {
            rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_UNKNOWN => Tuner::UNKNOWN,
            rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_E4000 => Tuner::E4000,
            rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_FC0012 => Tuner::FC0012,
            rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_FC0013 => Tuner::FC0013,
            rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_FC2580 => Tuner::FC2580,
            rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_R820T => Tuner::R820T,
            rtlsdrsys::rtlsdr_tuner_RTLSDR_TUNER_R828D => Tuner::R828D,
            // TODO: need to update binding.. so print warning to log
            _ => Tuner::UNKNOWN,
        }
    }

    pub fn get_tuner_gains(&mut self) -> Result<std::vec::Vec<i32>, Error> {
        let num_gains =
            unsafe { rtlsdrsys::rtlsdr_get_tuner_gains(self.dev, std::ptr::null_mut()) };
        if num_gains <= 0 {
            return Err(Error::new(num_gains));
        }

        let mut ret: Vec<std::os::raw::c_int> = Vec::new();
        ret.resize(num_gains as usize, 0);
        match unsafe { rtlsdrsys::rtlsdr_get_tuner_gains(self.dev, ret.as_mut_ptr()) } {
            l if l == num_gains => Ok(ret.into_iter().map(|x| x as i32).collect()),
            err => Err(Error::new(err)),
        }
    }

    pub fn set_tuner_gain(&mut self, gain: i32) -> Result<(), Error> {
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_tuner_gain(self.dev, gain) })
    }

    pub fn get_tuner_gain(&mut self) -> Result<i32, Error> {
        match unsafe { rtlsdrsys::rtlsdr_get_tuner_gain(self.dev) } {
            0 => Err(Error::new(0)),
            gain => Ok(gain as i32),
        }
    }

    pub fn set_tuner_bandwidth(&mut self, bw: u32) -> Result<(), Error> {
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_tuner_bandwidth(self.dev, bw) })
    }

    pub fn set_tuner_if_gain(&mut self, state: usize, gain: i32) -> Result<(), Error> {
        Self::tosuccess(unsafe {
            rtlsdrsys::rtlsdr_set_tuner_if_gain(
                self.dev,
                state as std::os::raw::c_int,
                gain as std::os::raw::c_int,
            )
        })
    }

    pub fn set_tuner_gain_mode(&mut self, manual: bool) -> Result<(), Error> {
        let on = match manual {
            true => 1,
            false => 0,
        };
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_tuner_gain_mode(self.dev, on) })
    }

    pub fn set_sample_rate(&mut self, rate: u32) -> Result<(), Error> {
        // TODO: test for -EINVAL and have the error reflect that
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_sample_rate(self.dev, rate) })
    }

    pub fn get_sample_rate(&mut self) -> Result<u32, Error> {
        // TODO: test for -EINVAL and have the error reflect that
        match unsafe { rtlsdrsys::rtlsdr_get_sample_rate(self.dev) } {
            0 => Err(Error::new(-1)),
            hz => Ok(hz),
        }
    }

    pub fn set_testmode(&mut self, manual: bool) -> Result<(), Error> {
        let on = match manual {
            true => 1,
            false => 0,
        };
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_testmode(self.dev, on) })
    }

    pub fn set_agc_mode(&mut self, manual: bool) -> Result<(), Error> {
        let on = match manual {
            true => 1,
            false => 0,
        };
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_agc_mode(self.dev, on) })
    }

    pub fn set_direct_sampling(&mut self, ds: DirectSampling) -> Result<(), Error> {
        let on = ds as std::os::raw::c_int;
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_direct_sampling(self.dev, on) })
    }

    pub fn get_direct_sampling(&mut self) -> Result<DirectSampling, Error> {
        let on = unsafe { rtlsdrsys::rtlsdr_get_direct_sampling(self.dev) };
        match on {
            0 => Ok(DirectSampling::Disabled),
            1 => Ok(DirectSampling::I),
            2 => Ok(DirectSampling::Q),
            _ => Err(Error::new(1)),
        }
    }

    pub fn set_offset_tuning(&mut self, manual: bool) -> Result<(), Error> {
        let on = match manual {
            true => 1,
            false => 0,
        };
        Self::tosuccess(unsafe { rtlsdrsys::rtlsdr_set_offset_tuning(self.dev, on) })
    }

    pub fn rtlsdr_get_offset_tuning(&mut self) -> Result<bool, Error> {
        let on = unsafe { rtlsdrsys::rtlsdr_get_direct_sampling(self.dev) };
        match on {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::new(1)),
        }
    }
}

impl Device {

    pub fn get_device_count() -> u32 {
        unsafe {
            rtlsdrsys::rtlsdr_get_device_count()
        }
    }
    pub fn get_device_name(idx : u32) -> Result<&'static str, Error> {
        let cstr = unsafe {rtlsdrsys::rtlsdr_get_device_name(idx)};

            let c_str = unsafe { std::ffi::CStr::from_ptr(cstr) };
            // TODO: use failure trait to wrap the conversion error
            let str_slice: &str = c_str.to_str().map_err(|_| Error::new(-1))?;
            Ok(str_slice)
    }

    pub fn get_index_by_serial(serial : &str) -> Result<usize, Error> {
        // TODO: proper error propagation
        let cstr = std::ffi::CString::new(serial).map_err(|_|Error::new(0))?;
        let index = unsafe { rtlsdrsys::rtlsdr_get_index_by_serial(cstr.as_ptr()) };

        match index {
            idx if idx < 0  => Err(Error::new(index)),
            idx => Ok(idx as usize),
        }
    }


    pub fn new(index: u32) -> Result<Device, Error> {
        let mut dev: *mut rtlsdrsys::rtlsdr_dev_t = std::ptr::null_mut();
        match unsafe { rtlsdrsys::rtlsdr_open(&mut dev, index) } {
            0 => Ok(Device { dev }),
            err => Err(Error::new(err)),
        }
    }
}

// Reading
impl Device {

    #[allow(dead_code)]
    unsafe extern "C" fn cbwrapper(
        buf: *mut ::std::os::raw::c_uchar,
        len: u32,
        ctx: *mut ::std::os::raw::c_void,
    ) {
        let cb = ctx as *mut CbWrapper;
        let slice = std::slice::from_raw_parts(buf as *const u8, len as usize);
        ((*cb).cb)(slice)
    }

    pub fn reset_buffer(&mut self) -> Result<(), Error> {
        match unsafe { rtlsdrsys::rtlsdr_reset_buffer(self.dev) } {
            0 => Ok(()),
            err => Err(Error::new(err)),
        }
    }

    pub fn read_async<CB>(&mut self, mut cb: CB) -> Result<(), Error>
    where
        CB: FnMut(&[u8]),
    {
        let cbref: &mut dyn FnMut(&[u8]) = &mut cb;
        let mut wrapper = CbWrapper { cb: cbref };

        let ctx: *mut std::os::raw::c_void =
            &mut wrapper as *mut CbWrapper as *mut std::os::raw::c_void;
        match unsafe { rtlsdrsys::rtlsdr_wait_async(self.dev, Some(Self::cbwrapper), ctx) } {
            0 => Ok(()),
            err => Err(Error::new(err)),
        }
    }

    pub fn cancel_async(&mut self) -> Result<(), Error> {
        match unsafe { rtlsdrsys::rtlsdr_cancel_async(self.dev) } {
            0 => Ok(()),
            err => Err(Error::new(err)),
        }
    }

    pub fn set_bias_tee(&mut self, enable: bool) -> Result<(), Error> {
        let on: std::os::raw::c_int = match enable {
            true => 1,
            false => 0,
        };
        match unsafe { rtlsdrsys::rtlsdr_set_bias_tee(self.dev, on) } {
            0 => Ok(()),
            err => Err(Error::new(err)),
        }
    }
}

impl std::io::Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        let len: std::os::raw::c_int = buf.len() as std::os::raw::c_int;
        let mut n: std::os::raw::c_int = 0;
        let bufptr: *mut std::os::raw::c_void =
            buf.as_mut_ptr() as *mut u8 as (*mut std::os::raw::c_void);
        match unsafe { rtlsdrsys::rtlsdr_read_sync(self.dev, bufptr, len, &mut n) } {
            0 => Ok(n as usize),
            err => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                Error::new(err),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
