/* -*- c++ -*- */
/*
 * Copyright 2021 gr-sparsdr author.
 *
 * This is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 3, or (at your option)
 * any later version.
 *
 * This software is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this software; see the file COPYING.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street,
 * Boston, MA 02110-1301, USA.
 */

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <gnuradio/io_signature.h>
#include "time_encoded_to_tagged_stream_impl.h"

namespace gr {
  namespace sparsdr {

    time_encoded_to_tagged_stream::sptr
    time_encoded_to_tagged_stream::make()
    {
      return gnuradio::get_initial_sptr
        (new time_encoded_to_tagged_stream_impl());
    }


    /*
     * The private constructor
     */
    time_encoded_to_tagged_stream_impl::time_encoded_to_tagged_stream_impl()
      : gr::block("time_encoded_to_tagged_stream",
              gr::io_signature::make(1, 1, sizeof(uint64_t)),
              gr::io_signature::make(1, 1, sizeof(uint64_t))),
        d_key(pmt::string_to_symbol("timestamp"))
    {}

    /*
     * Our virtual destructor.
     */
    time_encoded_to_tagged_stream_impl::~time_encoded_to_tagged_stream_impl()
    {
    }

    void
    time_encoded_to_tagged_stream_impl::forecast (int noutput_items, gr_vector_int &ninput_items_required)
    {
      /* <+forecast+> e.g. ninput_items_required[0] = noutput_items */
      ninput_items_required[0] = noutput_items;
    }

    int
    time_encoded_to_tagged_stream_impl::general_work (int noutput_items,
                       gr_vector_int &ninput_items,
                       gr_vector_const_void_star &input_items,
                       gr_vector_void_star &output_items)
    {
      const uint64_t *in = (const uint64_t *) input_items[0];
      uint64_t *out = (uint64_t *) output_items[0];

      // remember number of samples to remove 
      int deleted_samples = 0;
      for(int i=0; i<noutput_items; i++) {
          if ((in[i] & 0x00000000ffffffff) == 0x00000000ffffffff) {
              // the item to be tagged is the next one in the datastream,
              // hence the absolute offset is all items that have been written
              // to output plus the current value of i minus items that have
              // been skipped/deleted plus one. We can omit plus one b/c
              // deleted_samples hasn't been incremented yet
              uint64_t absolute_offset = nitems_written(0) + i - deleted_samples;
              // for sparsdr one time unit equals 10.24µs, we will transmit tags
              // in µs so we need to transform
              float time = (float(in[i] >> 32)*10.24;
              pmt::pmt_t val = pmt::from_float(time);
              // write tag
              add_item_tag(0, absolute_offset, d_key, val);
              // ### CODE FOR DEBUGGING ###
              //if (absolute_offset>d_last_offset && time<d_last_time) {
              //  std::cout << "NEGATIVE TIME" << std::endl;
              //  printf("last: %8ld ---> %8f\n", d_last_offset, d_last_time);
              //  printf("curr: %8ld ---> %8f\n", absolute_offset, time);
              //}
              //d_last_offset = absolute_offset;
              //d_last_time = time;
              //printf("wrote tag t=%lf at offset %ld\n", (time), absolute_offset);
              // ### END CODE FOR DEBUGGING ###
              deleted_samples+=1;
          }
          else {
              out[i-deleted_samples]=in[i];
          }
      }

      // Tell runtime system how many input items we consumed on
      // each input stream.
      consume_each(noutput_items);

      // Tell runtime system how many output items we produced.
      return (noutput_items-deleted_samples);
    }

  } /* namespace sparsdr */
} /* namespace gr */

