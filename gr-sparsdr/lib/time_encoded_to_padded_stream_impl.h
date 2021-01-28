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

#ifndef INCLUDED_SPARSDR_TIME_ENCODED_TO_PADDED_STREAM_IMPL_H
#define INCLUDED_SPARSDR_TIME_ENCODED_TO_PADDED_STREAM_IMPL_H

#include <sparsdr/time_encoded_to_padded_stream.h>

namespace gr {
  namespace sparsdr {

    class time_encoded_to_padded_stream_impl : public time_encoded_to_padded_stream
    {
     private:
      // remember last timestamp
      uint32_t d_last_timestamp;
      // count number of items consumed since the last tag
      uint32_t d_items_read_since_last_tag;
      // keep track of number of zeroes we still need to pad
      int d_remaining_padding_length;
      // sample rate
      double d_sample_rate;

     public:
      time_encoded_to_padded_stream_impl(double sample_rate);
      ~time_encoded_to_padded_stream_impl();

      // Where all the action really happens
      void forecast (int noutput_items, gr_vector_int &ninput_items_required);

      int general_work(int noutput_items,
           gr_vector_int &ninput_items,
           gr_vector_const_void_star &input_items,
           gr_vector_void_star &output_items);

    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_TIME_ENCODED_TO_PADDED_STREAM_IMPL_H */

