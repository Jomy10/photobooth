#include <input.h>

#include <fcntl.h>
#include <unistd.h>
#include <linux/input.h>
#include <stdbool.h>
#include <stdint.h>
#include <errno.h>
#include <assert.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

struct input_touch_context input_touch_context_create(void) {
  return (struct input_touch_context){0,0,0,0};
}

int input_init(int* fd_out, struct input_touch_context* touch_context_out, const char* touch_device) {
  int fd = open(touch_device, O_RDONLY);
  if (fd == -1) {
    fprintf(stderr, "Error opening device %s\n (%i): %m", touch_device, errno);
    return -1;
  }
  *fd_out = fd;

  struct input_absinfo abs_info;
  int retval = ioctl(fd, EVIOCGABS(ABS_X), &abs_info);
  if (retval < 0) {
    fprintf(stderr, "Could not get max X\n");
    return -1;
  }
  touch_context_out->currentx = abs_info.value;
  touch_context_out->minx = abs_info.minimum;
  touch_context_out->maxx = abs_info.maximum;

  retval = ioctl(fd, EVIOCGABS(ABS_Y), &abs_info);
  if (retval < 0) {
    fprintf(stderr, "Could not get max Y\n");
    return -1;
  }
  touch_context_out->currenty = abs_info.value;
  touch_context_out->miny = abs_info.minimum;
  touch_context_out->maxy = abs_info.maximum;

  return 0;
}

/// Convert absolute touch position to relative screen position
struct rel_touch_position input_abs_to_rel_screen(
  struct input_touch_context* ctx
) {
  return (struct rel_touch_position) {
    .x = ((double)(ctx->currentx - ctx->minx)) / ((double)ctx->maxx),
    .y = ((double)(ctx->currenty - ctx->miny)) / ((double)ctx->maxy),
  };
}

/// max data size: 32 bits
enum InputEventType input_read(int input_fd, void* data) {
  struct input_event ie;
  if (read(input_fd, &ie, sizeof(struct input_event)) > 0) {
    if (ie.type == EV_KEY && ie.code == 330) {
      if (ie.value == 1) {
        return IE_PRESS;
      } else if (ie.value == 0) {
        return IE_RELEASE;
      }
    } else if (ie.type == EV_ABS) { // absolute touch position
      if (ie.code == ABS_X) {
        *((uint32_t*)data) = ie.value;
        return IE_MOVE_X;
      } else if (ie.code == ABS_Y) {
        *((uint32_t*)data) = ie.value;
        return IE_MOVE_Y;
      }
    }
  } else {
    return IE_END;
  }

  //fprintf(stderr, "IE_IGNORE: 0x%02x\n", ie.type);
  return IE_IGNORE;
}

// reference: https://android.googlesource.com/kernel/msm.git/+/android-msm-hammerhead-3.4-kk-r1/include/linux/input.h
// see also: https://cpp.hotexamples.com/site/file?hash=0xf56d603bcbf698cf6ed42a74370e878e98413865dca8c2f00ec9eb9555009a17&fullName=vmd-python-master/vmd/vmd_src/eventio.c&project=Eigenstate/vmd-python

