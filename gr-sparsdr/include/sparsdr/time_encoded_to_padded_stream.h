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

#ifndef INCLUDED_SPARSDR_TIME_ENCODED_TO_PADDED_STREAM_H
#define INCLUDED_SPARSDR_TIME_ENCODED_TO_PADDED_STREAM_H

#include <sparsdr/api.h>
#include <gnuradio/block.h>

namespace gr {
  namespace sparsdr {

    /*!
     * \brief <+description of block+>
     * \ingroup sparsdr
     *
     */
    class SPARSDR_API time_encoded_to_padded_stream : virtual public gr::block
    {
     public:
      typedef boost::shared_ptr<time_encoded_to_padded_stream> sptr;

      /*!
       * \brief Return a shared_ptr to a new instance of sparsdr::time_encoded_to_padded_stream.
       *
       * To avoid accidental use of raw pointers, sparsdr::time_encoded_to_padded_stream's
       * constructor is in a private implementation
       * class. sparsdr::time_encoded_to_padded_stream::make is the public interface for
       * creating new instances.
       */
      static sptr make(double sample_rate);
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_TIME_ENCODED_TO_PADDED_STREAM_H */

