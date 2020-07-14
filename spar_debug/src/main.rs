use std::io::prelude::*;
use std::convert::TryInto;
use std::fs;

fn from_bytes<'a, T>(buf: &'a [u8]) -> &'a [T] {
    unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr() as *const T,
            buf.len() / std::mem::size_of::<T>()
        )
    }
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
        let i = data >> 16;
        let q = data & 0xffff;

        let is_average = ((meta >> 15) & 1) > 0;
        let fft_index = meta & 0b111_1111_1111;
        let timestamp = ((meta >> 16) << 4) + ((meta  >> 11) & 0b1111);

        // if bin >> 32 > 0 {
        //     println!("{:064b}", bin);
        // }
        // // let fft_index = meta >> 16;
        // let timestamp = meta & 0b11_1111_1111;

        // assert!(fft_index < 2048);

        if is_average {
             println!("{:064b} a {}: {}/{}   {}", bin, fft_index, i , q, timestamp);
            assert_eq!(data, 0);
             n_avg += 1;
         } else {
        //      println!("d {} {}: {}/{}", n, fft_index, i , q);
             n_data += 1;
         }
    }

    println!("bin 0: {}", bins[0]);
    println!("n_data: {}   n_avg: {}    {}%", n_data, n_avg, n_data as f64 / n_avg as f64);
}
