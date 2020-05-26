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

use std::io::Write;
use std::time::Duration;

use super::bins::choice::choose_bins;
use super::bins::BinRange;
use crate::steps::writer::SampleSink;

/// Default timeout before flushing samples to output
pub const TIMEOUT: Duration = Duration::from_millis(100);

const DEFAULT_COMPRESSED_BANDWIDTH: f32 = 100_000_000.0;
const DEFAULT_FFT_SIZE: u16 = 2048;

/// Setup for decompression of one band
pub struct BandSetup<'w> {
    /// The bins to decompress
    pub bins: BinRange,
    /// The actual FFT size to use
    pub fft_size: u16,
    /// Floor of the center frequency offset, in bins
    pub fc_bins: f32,
    /// Fractional part of center frequency offset, in bins
    pub bin_offset: f32,
    /// Time to wait for a compressed sample before flushing output
    pub timeout: Duration,
    /// The destination to write decompressed samples to
    pub destination: Box<dyn SampleSink + Send + 'w>,
    /// Tagged window time log
    pub time_log: Option<Box<dyn Write + Send>>,
}

impl<'w> BandSetup<'w> {
    /// Returns the bins to be decompressed for this band
    pub fn bins(&self) -> BinRange {
        self.bins.clone()
    }
    /// Returns the floor of the center frequency offset, in bins (this is a whole number)
    pub fn fc_bins(&self) -> f32 {
        self.fc_bins
    }
}

/// Setup builder for decompression of one band
pub struct BandSetupBuilder<'w> {
    /// Bandwidth of the compressed data
    compressed_bandwidth: f32,
    /// Center frequency to decompress, relative to the center of the compressed data
    center_frequency: f32,
    /// The number of bins to decompress
    bins: u16,
    /// The number of bins used to compress the signals
    compression_bins: u16,
    /// Time to wait for a compressed sample before flushing output
    timeout: Duration,
    /// The destination to write decompressed samples to
    destination: Box<dyn SampleSink + Send + 'w>,
    /// Tagged window time log
    time_log: Option<Box<dyn Write + Send>>,
}

impl<'w> BandSetupBuilder<'w> {
    /// Creates a default band setup that will decompress a full 100 MHz spectrum and write
    /// decompressed samples to the provided source
    pub fn new(destination: Box<dyn SampleSink + Send + 'w>, compression_bins: u16) -> Self {
        BandSetupBuilder {
            compressed_bandwidth: DEFAULT_COMPRESSED_BANDWIDTH,
            center_frequency: 0.0,
            bins: DEFAULT_FFT_SIZE,
            compression_bins,
            timeout: TIMEOUT,
            destination,
            time_log: None,
        }
    }
    /// Sets the center frequency to decompress
    pub fn center_frequency(self, center_frequency: f32) -> Self {
        BandSetupBuilder {
            center_frequency,
            ..self
        }
    }
    /// Sets the number of bins to decompress
    pub fn bins(self, bins: u16) -> Self {
        BandSetupBuilder { bins, ..self }
    }
    /// Sets the time to wait for a compressed sample before flushing output
    pub fn timeout(self, timeout: Duration) -> Self {
        BandSetupBuilder { timeout, ..self }
    }
    /// Sets the bandwidth of the compressed data
    pub fn compressed_bandwidth(self, compressed_bandwidth: f32) -> Self {
        BandSetupBuilder {
            compressed_bandwidth,
            ..self
        }
    }
    /// Sets the window output time log file
    pub fn time_log(self, time_log: Box<dyn Write + Send>) -> Self {
        BandSetupBuilder {
            time_log: Some(time_log),
            ..self
        }
    }

    /// Builds a setup from this builder
    pub fn build(self) -> BandSetup<'w> {
        let fft_size = self
            .bins
            .checked_next_power_of_two()
            .expect("FFT size too large to round up");

        let exact_bin_offset =
            f32::from(self.compression_bins) * self.center_frequency / self.compressed_bandwidth;
        let fc_bins = exact_bin_offset.floor();
        let bin_offset = exact_bin_offset.fract();

        let bin_range = choose_bins(self.bins, fc_bins as i16, self.compression_bins);

        BandSetup {
            bins: bin_range,
            fft_size,
            bin_offset,
            fc_bins,
            timeout: self.timeout,
            destination: self.destination,
            time_log: self.time_log,
        }
    }
}
