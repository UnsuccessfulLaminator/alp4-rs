use crate::alp_binding::*;
use crate::error::*;
use libloading::Library;
use std::ffi::{OsStr, c_long};



macro_rules! alp_call {
    ($lib:expr, $name:expr, $sig:ty; $($args:expr),*) => {
        unsafe {
            if let Ok(f) = $lib.get::<$sig>($name.as_bytes()) {
                let ret = f($($args),*) as i64;

                if ret == ALP_OK as i64 { Ok(()) }
                else {
                    Err(AlpError::from_repr(ret).unwrap_or(AlpError::Unknown))
                }
            }
            else { panic!("couldn't get symbol {}", $name) }
        }
    }
}

type AlpDevAllocFn = unsafe extern fn(c_long, c_long, *mut ALP_ID) -> c_long;
type AlpDevFreeFn = unsafe extern fn(ALP_ID) -> c_long;
type AlpDevInquireFn = unsafe extern fn(ALP_ID, c_long, *mut c_long) -> c_long;
type AlpSeqAllocFn = unsafe extern fn(ALP_ID, c_long, c_long, *mut ALP_ID) -> c_long;
type AlpSeqFreeFn = unsafe extern fn(ALP_ID, ALP_ID) -> c_long;
type AlpSeqPutFn = unsafe extern fn(ALP_ID, ALP_ID, c_long, c_long, *const u8) -> c_long;
type AlpProjStartContFn = unsafe extern fn(ALP_ID, ALP_ID) -> c_long;
type AlpProjHaltFn = unsafe extern fn(ALP_ID) -> c_long;
type AlpProjInquireExFn = unsafe extern fn(ALP_ID, c_long, *mut tAlpProjProgress) -> c_long;
type AlpProjInquireFn = unsafe extern fn(ALP_ID, c_long, *mut c_long) -> c_long;
type AlpProjWaitFn = unsafe extern fn(ALP_ID) -> c_long;
type AlpSeqTimingFn = unsafe extern fn(ALP_ID, ALP_ID, c_long, c_long, c_long, c_long, c_long) -> c_long;
type AlpSeqControlFn = unsafe extern fn(ALP_ID, ALP_ID, c_long, c_long) -> c_long;



pub struct Alp {
    lib: Library
}

impl Alp {
    pub fn from_path<P: AsRef<OsStr>>(path: P) -> Result<Self, String> {
        let lib = unsafe { Library::new(path) }.map_err(|e| e.to_string())?;

        Ok(Self { lib })
    }

    pub fn allocate_device(&self, id: Option<u64>) -> AlpResult<AlpDevice> {
        let id = id.unwrap_or(0) as c_long;
        let mut ret_id = 0;

        alp_call!(self.lib, "AlpDevAlloc", AlpDevAllocFn; id, 0, &mut ret_id)?;

        Ok(AlpDevice {
            lib: &self.lib,
            id: ret_id
        })
    }
}



pub struct AlpDevice<'a> {
    lib: &'a Library,
    id: ALP_ID
}

impl<'a> Drop for AlpDevice<'a> {
    fn drop(&mut self) {
        alp_call!(self.lib, "AlpDevFree", AlpDevFreeFn; self.id)
            .expect("couldn't free ALP device");
    }
}

impl<'a> AlpDevice<'a> {
    pub fn allocate_sequence(&self, bitplanes: usize, images: usize)
    -> AlpResult<AlpSequence> {
        let mut id = 0;
        let bitplanes = bitplanes as c_long;
        let images = images as c_long;
        
        alp_call!(
            self.lib, "AlpSeqAlloc", AlpSeqAllocFn;
            self.id, bitplanes, images, &mut id
        )?;

        Ok(AlpSequence {
            lib: &self.lib,
            dev: self,
            id
        })
    }

    pub fn display_size(&self) -> AlpResult<(usize, usize)> {
        let (mut width, mut height) = (0, 0);

        alp_call!(
            self.lib, "AlpDevInquire", AlpDevInquireFn;
            self.id, ALP_DEV_DISPLAY_WIDTH as c_long, &mut width
        )?;
        
        alp_call!(
            self.lib, "AlpDevInquire", AlpDevInquireFn;
            self.id, ALP_DEV_DISPLAY_HEIGHT as c_long, &mut height
        )?;

        Ok((width as usize, height as usize))
    }

    pub fn halt(&self) -> AlpResult<()> {
        alp_call!(self.lib, "AlpProjHalt", AlpProjHaltFn; self.id)
    }

    pub fn current_sequence_id(&self) -> AlpResult<Option<u64>> {
        let mut progress = tAlpProjProgress {
            CurrentQueueId: 0,
            SequenceId: 0,
            nWaitingSequences: 0,
            nSequenceCounter: 0,
            nSequenceCounterUnderflow: 0,
            nFrameCounter: 0,
            nPictureTime: 0,
            nFramesPerSubSequence: 0,
            nFlags: 0
        };

        alp_call!(
            self.lib, "AlpProjInquireEx", AlpProjInquireExFn;
            self.id, ALP_PROJ_PROGRESS as c_long, &mut progress
        )?;

        let projecting = (progress.nFlags & ALP_FLAG_QUEUE_IDLE) == 0;

        Ok(projecting.then_some(progress.SequenceId as u64))
    }

    pub fn is_projecting(&self) -> AlpResult<bool> {
        let mut val = 0;

        alp_call!(
            self.lib, "AlpProjInquire", AlpProjInquireFn;
            self.id, ALP_PROJ_STATE as c_long, &mut val
        )?;

        Ok(val == ALP_PROJ_ACTIVE as c_long)
    }

    pub fn wait(&self) -> AlpResult<()> {
        alp_call!(self.lib, "AlpProjWait", AlpProjWaitFn; self.id)
    }
}



pub struct AlpSequence<'a> {
    lib: &'a Library,
    dev: &'a AlpDevice<'a>,
    id: ALP_ID
}

impl<'a> Drop for AlpSequence<'a> {
    fn drop(&mut self) {
        let current_seq = self.dev.current_sequence_id()
            .expect("couldn't get current ALP sequence ID");

        if current_seq == Some(self.id as u64) {
            self.dev.halt().expect("couldn't halt ALP device projection");
        }

        alp_call!(self.lib, "AlpSeqFree", AlpSeqFreeFn; self.dev.id, self.id)
            .expect("couldn't free ALP sequence");
    }
}

impl<'a> AlpSequence<'a> {
    pub fn id(&self) -> u64 {
        self.id as u64
    }

    pub fn put(&self, offset: usize, n: usize, data: &[u8]) -> AlpResult<()> {
        let offset = offset as c_long;
        let n = n as c_long;

        alp_call!(
            self.lib, "AlpSeqPut", AlpSeqPutFn;
            self.dev.id, self.id, offset, n, data.as_ptr()
        )
    }

    pub fn start_cont(&self) -> AlpResult<()> {
        alp_call!(
            self.lib, "AlpProjStartCont", AlpProjStartContFn;
            self.dev.id, self.id
        )
    }

    pub fn set_picture_time(&self, time_us: usize) -> AlpResult<()> {
        alp_call!(
            self.lib, "AlpSeqTiming", AlpSeqTimingFn;
            self.dev.id, self.id, ALP_DEFAULT as c_long, time_us as c_long,
            ALP_DEFAULT as c_long, ALP_DEFAULT as c_long, ALP_DEFAULT as c_long
        )
    }

    fn set_control(&self, control: Control, value: c_long) -> AlpResult<()> {
        alp_call!(
            self.lib, "AlpSeqControl", AlpSeqControlFn;
            self.dev.id, self.id, control as c_long, value
        )
    }

    pub fn set_data_format(&self, format: DataFormat) -> AlpResult<()> {
        self.set_control(Control::DataFormat, format as c_long)
    }
}



#[repr(i64)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DataFormat {
    MsbAlign = ALP_DATA_MSB_ALIGN as i64,
    LsbAlign = ALP_DATA_LSB_ALIGN as i64,
    BinaryTopDown = ALP_DATA_BINARY_TOPDOWN as i64,
    BinaryBottomUp = ALP_DATA_BINARY_BOTTOMUP as i64
}



#[allow(dead_code)]
#[repr(i64)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Control {
    SeqRepeat = ALP_SEQ_REPEAT as i64,
    FirstFrame = ALP_FIRSTFRAME as i64,
    LastFrame = ALP_LASTFRAME as i64,
    BitNum = ALP_BITNUM as i64,
    BinMode = ALP_BIN_MODE as i64,
    DataFormat = ALP_DATA_FORMAT as i64,
    SeqPutLock = ALP_SEQ_PUT_LOCK as i64,
    ScrollFromRow = ALP_SCROLL_FROM_ROW as i64,
    ScrollToRow = ALP_SCROLL_TO_ROW as i64,
    FirstLine = ALP_FIRSTLINE as i64,
    LastLine = ALP_LASTLINE as i64,
    LineInc = ALP_LINE_INC as i64,
    FlutMode = ALP_FLUT_MODE as i64,
    FlutEntries9 = ALP_FLUT_ENTRIES9 as i64,
    FlutOffset9 = ALP_FLUT_OFFSET9 as i64,
    SeqLines = ALP_SEQ_DMD_LINES as i64,
    PwmMode = ALP_PWM_MODE as i64,
    MaskSelect = ALP_DMD_MASK_SELECT as i64
}
