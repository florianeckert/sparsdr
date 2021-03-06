use std::io::prelude::*;
use std::fs;

fn from_bytes<'a, T>(buf: &'a [u8]) -> &'a [T] {
    unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr() as *const T,
            buf.len() / std::mem::size_of::<T>()
        )
    }
}

fn bitorder(n: u16) -> u16 {

    let mut ret = 0;

    for i in 0..16 {
        ret |= ((n >> i) & 1) << (15-i);
    }

    ret
}

fn main() {

    let filename = "/home/basti/Downloads/compressed.iqz";
    let mut buffer = Vec::new();

    let mut file = fs::File::open(filename).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    let bins : &[u64] = from_bytes(&buffer);

    println!("read {} bytes", buffer.len());
    println!("number of bins: {}", bins.len());

    let mut n_avg = 0;
    let mut n_data = 0;

    for (n, bin) in bins.iter().enumerate() {
        let meta = *bin & 0xffff_ffff;
        let data = *bin >> 32;

        let is_average = ((meta >> 15) & 1) > 0;

        let fft_index = (meta << 1) & 0xffe0;
        let fft_index = bitorder(fft_index as u16);

        if is_average {
            let mag = ((data & 0xffff) << 16) | (data >> 16);
            // if mag > 0 {
            println!("{:064b} a {}: {}", bin, fft_index, mag);
            // }
            n_avg += 1;
        } else {
            // println!("{:064b} d {:4}: {}", bin, fft_index, n);
            n_data += 1;
        }
    }

    println!("bin 0: {}", bins[0]);
    println!("n_data: {}   n_avg: {}    {}%", n_data, n_avg, n_data as f64 / n_avg as f64);
}
