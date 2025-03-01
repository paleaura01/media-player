// wrapper.h
#ifndef WRAPPER_H
#define WRAPPER_H

// Define standard macros required by FFmpeg headers.
#ifndef __STDC_CONSTANT_MACROS
#define __STDC_CONSTANT_MACROS
#endif
#ifndef __STDC_FORMAT_MACROS
#define __STDC_FORMAT_MACROS
#endif

// Include standard headers so that types like int8_t/uint8_t are defined.
#include <stdint.h>
#include <stddef.h>

// Neutralize problematic macros
#define __attribute__(x)
#define __extension__

#ifdef __cplusplus
extern "C" {
#endif

// Include the FFmpeg headers you need
#include <libavutil/common.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
// add any other headers as needed

#ifdef __cplusplus
}
#endif

#endif // WRAPPER_H
