struct modeset_dev* modeset_dev_front_buf(struct modeset_dev* dev) {
  return &dev->bufs[dev->front_buf];
}

struct modeset_dev* modeset_dev_back_buf(struct modeset_dev* dev) {
  return &dev->bufs[!dev->front_buf];
}

