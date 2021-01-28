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
#include "time_encoded_to_padded_stream_impl.h"

namespace gr {
  namespace sparsdr {

    time_encoded_to_padded_stream::sptr
    time_encoded_to_padded_stream::make(double sample_rate)
    {
      return gnuradio::get_initial_sptr
        (new time_encoded_to_padded_stream_impl(sample_rate));
    }


    /*
     * The private constructor
     */
    time_encoded_to_padded_stream_impl::time_encoded_to_padded_stream_impl(double sample_rate)
      : gr::block("time_encoded_to_padded_stream",
              gr::io_signature::make(1, 1, sizeof(uint64_t)),
              gr::io_signature::make(1, 1, sizeof(uint64_t))),
            d_last_timestamp(0),
            d_items_read_since_last_tag(0),
            d_remaining_padding_length(0),
            d_sample_rate(sample_rate)
    {}

    /*
     * Our virtual destructor.
     */
    time_encoded_to_padded_stream_impl::~time_encoded_to_padded_stream_impl()
    {
    }

    void
    time_encoded_to_padded_stream_impl::forecast (int noutput_items, gr_vector_int &ninput_items_required)
    {
      /* <+forecast+> e.g. ninput_items_required[0] = noutput_items */
      ninput_items_required[0] = noutput_items;
    }

    int
    time_encoded_to_padded_stream_impl::general_work (int noutput_items,
                       gr_vector_int &ninput_items,
                       gr_vector_const_void_star &input_items,
                       gr_vector_void_star &output_items)
    {
      const uint64_t *in = (const uint64_t *) input_items[0];
      uint64_t *out = (uint64_t *) output_items[0];

      int written_items = 0;
      int consumed_items = 0;

      // write remaining padding zeroes from last call
      while(d_remaining_padding_length>0) {
        // stop if output buffer full
        if(written_items >= noutput_items) {
            consume_each(consumed_items);
            return written_items;
        }
        // pad with zero
        out[written_items] = 0;
        written_items++;
        d_remaining_padding_length--;
      }

      // only read and write within buffer lengths
      while(written_items<noutput_items && consumed_items<noutput_items)
      {
        const uint64_t curr_item = in[consumed_items];
        consumed_items++;

        // probe for timestamp advertisement
        if ((curr_item & 0x00000000ffffffff) == 0x00000000ffffffff) {
            uint32_t timestamp = curr_item >> 32;
            // compensate inital offset
            if(d_last_timestamp==0) {
                d_last_timestamp = timestamp;
            }
            // calculate difference between last timestamp and corresponding number of samples
            uint32_t timestamp_diff = timestamp - d_last_timestamp;
            float time_diff = float(timestamp_diff) * 10.24 * (d_sample_rate/1000000);
            uint32_t sample_diff = uint32_t(time_diff+0.5);
            // calculate the difference of expected samples vs actual samples since last timestamp
            // if this is above 0 we need to pad with zeroes, as there was a jump
            d_remaining_padding_length = sample_diff - d_items_read_since_last_tag;

            // reset these values for the next tag
            d_last_timestamp = timestamp;
            d_items_read_since_last_tag = 0;

            // start padding as far as possible
            while(d_remaining_padding_length>0) {
                // stop if output buffer full
                if(written_items >= noutput_items) {
                    consume_each(consumed_items);
                    return written_items;
                }
                // pad with zero
                out[written_items] = 0;
                written_items++;
                d_remaining_padding_length--;
            }
        }
        // no tag -> just copy the sample and increment counters
        else {
            out[written_items] = curr_item;;
            written_items++;
            d_items_read_since_last_tag++;
        }
      }

      // Tell runtime system how many input items we consumed on
      // each input stream.
      consume_each (consumed_items);

      // Tell runtime system how many output items we produced.
      return written_items;
    }

  } /* namespace sparsdr */
} /* namespace gr */

