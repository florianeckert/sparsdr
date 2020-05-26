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

//! The input stage of the decompression, which reads samples from a source, groups them
//! into windows, and shifts them into logical order

use std::collections::BTreeMap;
use std::io::{Error, ErrorKind, Result, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use libc::{clock_gettime, timespec};
use sparsdr_bin_mask::BinMask;

use crate::bins::BinRange;
use crate::blocking::BlockLogs;
use crate::channel_ext::LoggingSender;
use crate::input::Sample;
use crate::iter_ext::IterExt;
use crate::window::{Logical, Tag, Window};

/// The setup for the input stage
pub struct InputSetup<I> {
    /// An iterator yielding samples from the source
    pub samples: I,
    /// Send half of channels used to send windows to all FFT stages
    pub destinations: Vec<ToFft>,
    /// A file or file-like thing where the time when each channel becomes active will be written
    pub input_time_log: Option<Box<dyn Write>>,
    /// The number of FFT bins used to compress the samples
    pub fft_size: u16,
}

/// Information about an FFT stage, and a channel that can be used to send windows there
pub struct ToFft {
    /// The range of bins the FFT stage is interested in
    pub bins: BinRange,
    /// The mask of bins the FFT stage is interested in (same as bins)
    pub bin_mask: BinMask,
    /// A sender on the channel to the FFT stage
    pub tx: LoggingSender<Window<Logical>>,
}

impl ToFft {
    /// Sends a window to this FFT/output stage, if this stage is interested in the window
    ///
    /// This function returns Err if the FFT/output thread thread has exited. On success,
    /// it returns true if the window was sent.
    pub fn send_if_interested(&self, window: &Window<Logical>) -> Result<bool> {
        // Check if the stage is interested
        if window.active_bins().overlaps(&self.bin_mask) {
            match self.tx.send(window.clone()) {
                Ok(()) => Ok(true),
                Err(_) => {
                    // This can happen when using the stop flag if the other thread stops before
                    // this one. It's not really an error here. If the other thread panicked,
                    // the higher-level code will detect it.
                    Err(Error::new(
                        ErrorKind::Other,
                        "A decompression thread has exited unexpectedly",
                    ))
                }
            }
        } else {
            // Not interested, don't send
            Ok(false)
        }
    }
}

pub fn run_input_stage<I>(mut setup: InputSetup<I>, stop: Arc<AtomicBool>) -> Result<InputReport>
where
    I: Iterator<Item = Result<Sample>>,
{
    // Write time log CSV headers
    if let Some(log) = setup.input_time_log.as_mut() {
        writeln!(log, "Channel,Tag,Seconds,Nanoseconds")?;
    }

    // Tag windows that have newly active channels
    let mut next_tag = Tag::default();

    // Set up iterator chain
    // Shift and send to the decompression thread
    let shift = setup
        .samples
        .take_while(|_| !stop.load(Ordering::Relaxed))
        .group(usize::from(setup.fft_size))
        .shift_result(setup.fft_size);

    // Process windows
    // Latency measurement hack: detect when the channel changes from active to inactive
    let mut prev_active = false;
    for window in shift {
        let mut window = window?;
        // Give this window a tag
        let window_tag = next_tag;
        window.set_tag(window_tag);
        next_tag = next_tag.next();

        // Send to each interested FFT stage
        for fft_stage in setup.destinations.iter() {
            match fft_stage.send_if_interested(&window) {
                Ok(sent) => {
                    // Latency measurement hack that works correctly only when there is only one
                    // band to decompress: Detect when the bins have become inactive and log that
                    if !sent && prev_active {
                        if let Some(ref mut log) = setup.input_time_log {
                            log_inactive_channel(&mut *log, "BLE 37", window_tag)?;
                        }
                    }

                    prev_active = sent;
                }
                Err(_) => {
                    // Other thread could have exited normally due to the stop flag, so just
                    // exit this thread normally
                    break;
                }
            }
        }
    }

    // Collect block logs
    let channel_send_blocks = setup
        .destinations
        .into_iter()
        .map(|to_fft| (to_fft.bins, to_fft.tx.logs()))
        .collect();

    Ok(InputReport {
        channel_send_blocks,
    })
}

/// Writes a log entry indicating that a channel with the provided name is active
fn log_inactive_channel(destination: &mut dyn Write, channel_name: &str, tag: Tag) -> Result<()> {
    let mut now = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // Use clock_gettime, which should be the same in C++ and Rust
    let status = unsafe { clock_gettime(libc::CLOCK_MONOTONIC, &mut now) };
    if status != 0 {
        return Err(Error::last_os_error());
    }
    // CSV format: channel, tag, seconds, nanoseconds
    writeln!(
        destination,
        "{},{},{},{}",
        channel_name, tag, now.tv_sec, now.tv_nsec
    )?;
    Ok(())
}

/// A report on the results of the input stage
#[derive(Debug)]
pub struct InputReport {
    /// Logs of blocks on channels for sending to the FFT/output stages
    pub channel_send_blocks: BTreeMap<BinRange, BlockLogs>,
}
