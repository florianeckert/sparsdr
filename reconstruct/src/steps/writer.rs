/*
 * Copyright 2019 The Regents of the University of California
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

//! Writing of samples to a destination

use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Stdout, Write};

use byteorder::{ByteOrder, NativeEndian};
use libc::{clock_gettime, timespec};
use num_complex::Complex32;

use crate::blocking::BlockLogger;
use crate::window::{Tag, TimeWindow};

/// A trait for outputs that can handle samples
pub trait SampleSink {
    /// Writes bytes to this sink
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error + Send>>;
}

impl<T: ?Sized> SampleSink for &'_ mut T
where
    T: SampleSink,
{
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error + Send>> {
        (*self).write_bytes(bytes)
    }
}

impl SampleSink for File {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error + Send>> {
        self.write_all(bytes).map_err(box_error)
    }
}

impl SampleSink for Stdout {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error + Send>> {
        self.write_all(bytes).map_err(box_error)
    }
}

impl SampleSink for Vec<u8> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error + Send>> {
        self.write_all(bytes).map_err(box_error)
    }
}

impl<W> SampleSink for BufWriter<W>
where
    W: Write,
{
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Box<dyn Error + Send>> {
        self.write_all(bytes).map_err(box_error)?;
        Ok(())
    }
}

fn box_error<E>(e: E) -> Box<dyn Error + Send>
where
    E: Error + Send + 'static,
{
    Box::new(e)
}

/// Writes samples to a destination
pub struct Writer {
    /// The index of the next sample to be written
    sample_index: u32,
}

impl Writer {
    /// Creates a new writer
    pub fn new() -> Self {
        Writer { sample_index: 0 }
    }

    /// Writes windows to a destination and returns the total number of samples written
    ///
    /// Each sample is written as two little-endian 32-bit floating-point values, first the real
    /// part and then the imaginary part.
    ///
    /// time_log: An optional destination where tagged windows will be logged
    pub fn write_windows<W, I>(
        &mut self,
        mut destination: W,
        windows: I,
        logger: &BlockLogger,
        mut time_log: Option<&mut (dyn Write + Send)>,
    ) -> Result<u64, Box<dyn Error + Send>>
    where
        W: SampleSink,
        I: IntoIterator<Item = TimeWindow>,
    {
        let mut samples_written = 0;

        for window in windows {
            logger.log_blocking(|| self.write_samples(&mut destination, window.samples()))?;
            samples_written += window.len() as u64;
            trace!("Writing {} samples", window.len());

            if let Some(ref mut log) = time_log {
                if let Some(tag) = window.tag() {
                    // Sample index is the index of the last sample in this window
                    log_window(log, tag, self.sample_index).map_err(box_error)?;
                }
            }
        }
        Ok(samples_written)
    }

    /// Writes samples to a destination
    ///
    /// Each sample is written as two little-endian 32-bit floating-point values, first the real
    /// part and then the imaginary part.
    fn write_samples<W>(
        &mut self,
        destination: &mut W,
        samples: &[Complex32],
    ) -> Result<(), Box<dyn Error + Send>>
    where
        W: SampleSink,
    {
        for sample in samples {
            let mut buffer = [0u8; 8];
            NativeEndian::write_f32(&mut buffer[0..4], sample.re);
            NativeEndian::write_f32(&mut buffer[4..8], sample.im);
            destination.write_bytes(&buffer)?;

            self.sample_index = self.sample_index.wrapping_add(1);
        }
        Ok(())
    }
}

/// Logs a window output
fn log_window(
    destination: &mut (dyn Write + Send),
    tag: &Tag,
    sample_index: u32,
) -> io::Result<()> {
    let mut now = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // Use clock_gettime, which should be the same in C++ and Rust
    let status = unsafe { clock_gettime(libc::CLOCK_MONOTONIC, &mut now) };
    if status != 0 {
        return Err(io::Error::last_os_error());
    }
    // CSV format: tag, sample index, seconds, nanoseconds
    writeln!(
        destination,
        "{},{},{},{}",
        tag, sample_index, now.tv_sec, now.tv_nsec
    )?;
    Ok(())
}
