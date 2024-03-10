#include <xf86drm.h>
#include <xf86drmMode.h>

int modeset_open(int *out, const char *node);
int modeset_prepare(int fd);
void modeset_draw(int fd);
void modeset_cleanup(int fd);

struct modeset_dev* get_modeset_list();

struct modeset_buf {
	uint32_t width;
	uint32_t height;
	uint32_t stride;
	uint32_t size;
	uint32_t handle;
	uint8_t *map;
	uint32_t fb;
};

struct modeset_dev {
	struct modeset_dev *next;

	unsigned int front_buf;
	struct modeset_buf bufs[2];

	drmModeModeInfo mode;
	uint32_t conn;
	uint32_t crtc;
	drmModeCrtc *saved_crtc;
};

struct modeset_buf* modeset_dev_front_buf(struct modeset_dev*);
struct modeset_buf* modeset_dev_back_buf(struct modeset_dev*);

