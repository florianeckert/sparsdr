#ifndef INCLUDED_SPARSDR_SAMPLE_PRINT_H
#define INCLUDED_SPARSDR_SAMPLE_PRINT_H

#include <sparsdr/api.h>
#include <gnuradio/sync_block.h>

namespace gr {
  namespace sparsdr {

    /*!
     * \brief Prints Info About Incoming iqz Samples
     * \ingroup sparsdr
     *
     */
    class SPARSDR_API sample_print : virtual public gr::sync_block
    {
     public:
      typedef boost::shared_ptr<sample_print> sptr;

      /*!
       * \brief Return a shared_ptr to a new instance of sparsdr::sample_print.
       *
       * To avoid accidental use of raw pointers, sparsdr::sample_print's
       * constructor is in a private implementation
       * class. sparsdr::sample_print::make is the public interface for
       * creating new instances.
       */
      static sptr make();
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_SAMPLE_PRINT_H */
