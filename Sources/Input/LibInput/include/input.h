#ifndef _PHB_INPUT_H
#define _PHB_INPUT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

struct input_touch_context {
  int currentx;
  int minx;
  int maxx;
  int currenty;
  int miny;
  int maxy;
};

struct input_touch_context input_touch_context_create(void);

/// Open the touch input device
int input_init(
  int* fd_out,
  struct input_touch_context* touch_context_out,
  const char* touch_device
);
void input_close(int fd);

/// Positionss in min..max of input_touch_context
/// moved to input_touch_context->current[x/y]
// struct abs_touch_position {
//   uint32_t x;
//   uint32_t y;
// };

/// Relative positions from 0 to 1
struct rel_touch_position {
  double x;
  double y;
};

/// convert abssolute touch pos to rel screen pos
struct rel_touch_position input_abs_to_rel_screen(
  struct input_touch_context* ctx
);

enum InputEventType {
  IE_PRESS,
  IE_RELEASE,
  /// Data: uint32_t
  IE_MOVE_X,
  /// Data: uint32_t
  IE_MOVE_Y,
  IE_IGNORE,
  IE_END
};

/// Read the next input event
///
/// Returns IE_END when no more input is present
enum InputEventType input_read(
  int input_fd,
  void* data
);

#ifdef __cplusplus
}
#endif

#endif

