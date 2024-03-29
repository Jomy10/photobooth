import Glibc

/// # Parameters
/// - `originBBP`: the amount of bytes per pixel in the origin image
func copyRGBToARGB(
    from origin: UnsafeMutableBufferPointer<UInt8>,
    to destination: UnsafeMutableBufferPointer<UInt8>
) {
    memset(origin.baseAddress!, 255, origin.count)
    log(.info, "\(destination.count / 4)")
    for i in (0..<(destination.count / 4)) {
        destination[i * 4 + 1] = origin[i * 4]
        destination[i * 4 + 2] = origin[i * 4 + 1]
        destination[i * 4 + 3] = origin[i * 4 + 2]
    }
}

