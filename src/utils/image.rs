use std::mem::MaybeUninit;

/// SAFETY: expects a correct abgr buffer and width and height to be correct
pub unsafe fn abgr_to_rgb(abgr_buffer: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgb_buffer = vec![MaybeUninit::uninit(); width * height * 3];

    // TODO: rayon par_iter?
    for (i, chunk) in abgr_buffer.chunks_exact(4).enumerate() {
        let rgb_index = i * 3;
        rgb_buffer[rgb_index    ] = MaybeUninit::new(chunk[2]); // R
        rgb_buffer[rgb_index + 1] = MaybeUninit::new(chunk[1]); // G
        rgb_buffer[rgb_index + 2] = MaybeUninit::new(chunk[0]); // B
    }

    return unsafe { std::mem::transmute(rgb_buffer) };
}
