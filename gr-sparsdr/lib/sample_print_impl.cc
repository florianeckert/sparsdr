#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <gnuradio/io_signature.h>
#include "sample_print_impl.h"

namespace gr {
  namespace sparsdr {

    sample_print::sptr
    sample_print::make()
    {
      return gnuradio::get_initial_sptr
        (new sample_print_impl());
    }

    /*
     * The private constructor
     */
    sample_print_impl::sample_print_impl()
      : gr::sync_block("sample_print",
              // Each compressed sample is really 8 bytes, but this also works.
              // The work function can reassemble each sample from two 4-byte
              // integers.
              gr::io_signature::make(1, 1, sizeof(uint32_t)),
              gr::io_signature::make(0, 0, 0))
    {}

    /*
     * Our virtual destructor.
     */
    sample_print_impl::~sample_print_impl()
    {
    }

    int
    sample_print_impl::work(int noutput_items,
        gr_vector_const_void_star &input_items,
        gr_vector_void_star &output_items)
    {
      const uint32_t* in = reinterpret_cast<const uint32_t*>(input_items[0]);
      const int sample_count = noutput_items / 2;
      for (int i = 0; i < sample_count; i++) {
          // Get the first half of the sample and check bit 15, which
          // indicates an average
          const uint32_t sample0 = in[i * 2];
          const uint32_t sample1 = in[(i * 2) + 1];

          const uint32_t time = ((sample0 & 0xf) << 16) | (sample0 >> 16);
          const uint16_t fft_index = (sample0 >> 4) & 0x7ff;     
          const bool is_average = (sample0 >> 15) & 1 == 1;

          const int16_t real = sample1 & 0xffff;
          const int16_t imag = sample1 >> 16;
          const uint32_t mag = (sample1 >> 16) | ((sample1 & 0xffff) << 16);

          if (is_average) {
              printf("Average,%12d,%12d,%12d\n", fft_index, time, mag);
          }
          else {
              printf("FFT Sample,%12d,%12d,%12d,%12d\n", fft_index, time, real, imag);
          }
      }

      // Tell runtime system how many output items we produced.
      return sample_count*2;
    }

  } /* namespace sparsdr */
} /* namespace gr */
