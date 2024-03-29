import Framebuffer
import drm
import Glibc
import Input
import Foundation
// import CairoGraphics
// import Utils
// import Cairo
// import CairoJPG
// import stb_image
// import stb_image_resize
import PhotoboothGraphics

enum LogType {
    case verbose
    case info
    case warning
    case error
}

extension LogType: CustomStringConvertible {
    var description: String {
        switch (self) {
            case .info: "INFO"
            case .error: "ERROR"
            case .warning: "WARN"
            case .verbose: "VERBOSE"
        }
    }
    
    var level: Int {
        switch (self) {
            case .verbose: 0
            case .info: 1
            case .warning: 2
            case .error: 3
        }
    }
}

var logging: [(type: LogType, date: Date, message: String)] = []

let minLoggingLevel: Int = LogType.info.level

func log(_ type: LogType, _ text: String) {
    if (type.level < minLoggingLevel) { return }

    let date = Date()
    print("[\(type) \(date)] \(text)")
    logging.append((type: type, date: date, message: text))
}

enum State {
    case idle
    case home
    case readyToTakePicture
    case clearScreenBeforeTakingPicture
    case takingPicture
    case preview
    case error
}

extension State {
    /// Determines how long the main thread will wait until going to the next state
    /// after rendering the current one
    var sleepTime: UInt32? {
        switch (self) {
            case .readyToTakePicture: 1
            case .preview: 10
            case .error: 10
            default: nil
        }
    }
}

@main
public struct Photobooth {
    public static func main() throws {
        var quit = false
        
        let logFile = "photobooth_log.txt"
        if !FileManager.default.fileExists(atPath: logFile) {
            FileManager.default.createFile(atPath: logFile, contents: nil, attributes: nil)
        }

        // Write logging to a file
        let handle: FileHandle
        do {
            handle = try FileHandle(forWritingTo: URL(fileURLWithPath: logFile))
        } catch let error {
            log(.error, "\(error)")
            exit(1)
        }
        var loggingBuffer: [UInt8] = []
        loggingBuffer.reserveCapacity(1024)
        let loggingGroup = DispatchGroup()
        loggingGroup.enter()
        DispatchQueue.global(qos: .background).async {
            while (!quit) {
                sleep(1) // TODO: bigger frequency
                for entry in logging {
                    loggingBuffer.append(contentsOf: "[\(entry.type) \(entry.date)] \(entry.message)\n".utf8)
                }
                do {
                    try handle.write(contentsOf: loggingBuffer)
                } catch let error {
                    log(.error, "Couldn't write logging \(error)")
                }
                loggingBuffer.removeAll(keepingCapacity: true)
            }
            try? handle.write(contentsOf: loggingBuffer)
            loggingGroup.leave()
        }
        defer {
            handle.closeFile()
        }
        log(.info, "Log initialized")

        do {
            try Photobooth.run(quit: &quit)
        } catch let error {
            log(.error, "Error in main: \(error)")
        }

        quit = true
        log(.info, "Quitting application...")
        loggingGroup.wait()

        log(.info, "Goodbye!")
    }

    public static func run(quit: inout Bool) throws {
        // initialize framebuffer and input //
        let fb = try Framebuffer()
        let input = try Input(windowSize: (w: Int(fb.size.width), h: Int(fb.size.height)))

        // Fill buffers with initial color //
        // // TODO: fill with splash screen
        // buffer = fb.backBuffer
        // memset(buffer.pointee.map, 200, Int(buffer.pointee.size))

        drmSetMaster(fb.fd)
        
        var state: State = .home
        var nextState: State = .idle

        var errorString: String = ""

        // Colors //
        //                      AARRGGBB
        let bgPixel: UInt32 = 0xFF32a8a8
        let fgPixel: UInt32 = ~bgPixel

        // Graphics initialization //
        // Map the screen buffers to a cairo surface & create graphics contexts to draw to
        var graphicsContexts: [UnsafeMutablePointer<modeset_buf>: PhotoboothGraphicsContext] = [:]
        graphicsContexts[fb.backBuffer] = try PhotoboothGraphicsContext(
            from: fb.backBuffer.pointee.map!,
            width: Int(fb.size.width), height: Int(fb.size.height),
            defaultClearColor: bgPixel
        )
        try fb.swapBuffers()
        graphicsContexts[fb.backBuffer] = try PhotoboothGraphicsContext(
            from: fb.backBuffer.pointee.map!,
            width: Int(fb.size.width), height: Int(fb.size.height),
            defaultClearColor: bgPixel
        )

        mainLoop: while (!quit) {
            log(.verbose, "Thick")

            let graphicsCtx = graphicsContexts[fb.backBuffer]!

            // Process input //
            log(.verbose, "Processing input")
            input.eventQueue.forEachSync { event in
                log(.info, "Input event: \(event)")
                if state == .idle {
                    log(.info, "Click input received, changing state to readyToTakePicture")
                    state = .readyToTakePicture
                }
            }
            input.eventQueue.removeAll(keepingCapacity: true)
            log(.verbose, "Input processedd")

            // Update UI //
            let buffer = fb.backBuffer // buffer object

            switch (state) {
            case .idle:
                break
            case .home:
                log(.info, "Going back to home...")

                graphicsCtx.clearBackground()
                graphicsCtx.drawText("Tik om een foto te nemen")

                nextState = .idle
            case .readyToTakePicture:
                log(.info, "In positie...")

                graphicsCtx.clearBackground()
                graphicsCtx.drawText("In positie!")

                nextState = .clearScreenBeforeTakingPicture
            case .clearScreenBeforeTakingPicture:
                graphicsCtx.clearBackground()

                nextState = .takingPicture
            case .takingPicture:
                log(.info, "Taking picture...")

                drmDropMaster(fb.fd) // leave drm so libcamera-apps can take over

                let filename = "out.jpg" // TODO

                do {
                    try cameraCapture(
                        filename: filename,
                        previewWidth: Int(fb.size.width), previewHeight: Int(fb.size.height)
                    )
                } catch (let error) {
                    log(.error, "Error executing libcamera-still: \(error)")
                    errorString = "\(error)"
                    state = .error
                    continue mainLoop
                }

                log(.info, "Image captured and saved to \(filename)")
                
                drmSetMaster(fb.fd)
                
                // Show confirmation message
                graphicsCtx.clearBackground()
                let text: String
                switch (Int.random(in: 0...4)) {
                case 0:
                    text = "Ziet er goed uit!"
                case 1: 
                    text = "All done!"
                case 2: 
                    text = "Kom weer wat dichterbij"
                case 3:
                    text = "Kijk eens wat een mooie foto!"
                case 4:
                    text = "Nog nooit zo'n mooie foto gezien!"
                default:
                    // unreachable
                    exit(1)
                }
                graphicsCtx.drawText(text)

                nextState = .preview
            case .preview:
                log(.info, "Previewing image...")

                let filename = "out.jpg"
                try graphicsCtx.drawJPEG(filename)
                log(.info, "Image has been copied to screen")

                nextState = .home
            case .error:
                log(.info, "Showing error on screen...")
                
                // Show error
                graphicsCtx.clearBackground()

                nextState = .home
            }

            if (state != .idle) {
                log(.verbose, "Swapping buffers")
                if (try? fb.swapBuffers()) == nil {
                    log(.warning, "Couldn't swap buffers")
                }
            }

            if let sleepTime = state.sleepTime {
                sleep(sleepTime)
            }

            state = nextState

            if (nextState != state) {
                log(.info, "State changed to \(state)")
            }
        }
    }
}

