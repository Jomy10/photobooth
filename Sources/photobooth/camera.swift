import Foundation

enum TimeUnit {
    case ms
    case sec
}

enum Metering {
    case centre
    case spot
    case average
    case custom
}

enum WhiteBalance {
    case auto
    case incandescent
    case tungsten
    case fluorescent
    case indoor
    case daylight
    case cloudy
    case custom
}

enum Encoding {
    case jpg
    case png
    case rgb
    case bmp
    case yuv420
}

func cameraCapture(
    width: Int = 0, height: Int = 0, filename: String,
    metering: Metering = .centre,
    ev: Int = 0,
    whiteBalance: WhiteBalance = .auto,
    brightness: Int = 0,
    contrast: Int = 1,
    saturation: Int = 1,
    sharpness: Int = 1,
    framerate: Int = -1,
    timeout: Int = 5, timeoutUnit: TimeUnit = .sec,
    encoding: Encoding = .jpg,
    previewWidth: Int = -1, previewHeight: Int = -1
) throws {
    let task = Process()
    let stdout = Pipe()
    let stderr = Pipe()
    task.standardOutput = stdout
    task.standardError = stderr
    task.executableURL = URL(fileURLWithPath: LIBCAMERA_STILL_LOCATION)
    task.arguments = [
        "--width", "\(width)",
        "--height", "\(height)",
        "-o", filename,
        "--metering", "\(metering)",
        "--ev", "\(ev)",
        "--timeout", "\(timeout)\(timeoutUnit)",
        "--preview", "0,0,\(previewWidth == -1 ? width : previewWidth),\(previewHeight == -1 ? height : previewHeight)",
        "--fullscreen",
        "--brightness", "\(brightness)",
        "--contrast", "\(contrast)",
        "--saturation", "\(saturation)",
        "--sharpness", "\(sharpness)",
        "--framerate", "\(framerate)",
        "--encoding", "\(encoding)"
    ]
    task.standardInput = nil
    try task.run()

    let stdoutData = stdout.fileHandleForReading.readDataToEndOfFile()
    let output = String(data: stdoutData, encoding: .utf8)!
    log(.info, "Stdout of libcamera-still: \(output)")
    let stderrData = stderr.fileHandleForReading.readDataToEndOfFile()
    let err = String(data: stderrData, encoding: .utf8)!
    log(.info, "Stderr of libcamera-still: \(err)")
}

