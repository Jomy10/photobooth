import LibInput
import Thread
// import Atomics
import Foundation
import Glibc

public typealias Position = (x: Int, y: Int)
public typealias Size = (w: Int, h: Int)

public final class Input {
    var inputFd: Int32 = -1
    var touchContext: input_touch_context = input_touch_context_create()
    var absTouchPos: Position
    // public var eventQueue: Mutex<[Event]>
    public var eventQueue: SynchronizedArray<Event>
    public var windowSize: Size
    var inputListenerContinue = true

    public enum Event {
        case mouseDown(pos: Position)
        case mouseUp(pos: Position)
    }

    public enum Error: Swift.Error {
        case inputInitError
    }

    public init(windowSize: Size) throws {
        let inputDevice = String("/dev/input/by-id/usb-QDtech_MPI7003-event-if00")
        let inputDeviceCStr = inputDevice.withCString { $0 }
        if (input_init(&self.inputFd, &self.touchContext, inputDeviceCStr) < 0) {
            throw Self.Error.inputInitError
        }

        self.windowSize = windowSize

        let rel = input_abs_to_rel_screen(&self.touchContext)
        self.absTouchPos = (
            x: Int(rel.x * Double(self.windowSize.w)),
            y: Int(rel.y * Double(self.windowSize.h))
        )
        
        // self.eventQueue = try Mutex([])
        self.eventQueue = SynchronizedArray()       
        self.startInputListener()
    }

    private func startInputListener() {
        DispatchQueue(label: "input-events").async {
            var data: UInt32 = 0
            var inputEventType: InputEventType = input_read(self.inputFd, &data)
            while self.inputListenerContinue {
                switch (inputEventType) {
                case IE_PRESS:
                    self.eventQueue.append(.mouseDown(pos: self.absTouchPos))
                case IE_RELEASE:
                    self.eventQueue.append(.mouseUp(pos: self.absTouchPos))
                case IE_MOVE_X:
                    // win.touchPos.x = Int(data)
                    self.touchContext.currentx = Int32(data)
                case IE_MOVE_Y:
                    self.touchContext.currenty = Int32(data)
                    let rel = input_abs_to_rel_screen(&self.touchContext)
                    assert(!rel.x.isNaN, "rel.x is nan")
                    assert(!rel.y.isNaN, "rel.y is nan")
                    assert(!rel.x.isInfinite, "rel.x is infinite")
                    assert(!rel.y.isInfinite, "rel.y is infinite")
                    self.absTouchPos = (
                        x: Int(rel.x * Double(self.windowSize.w)),
                        y: Int(rel.y * Double(self.windowSize.h))
                    )
                case IE_IGNORE:
                    break
                default:
                    // IE_END never happens?
                    fatalError("Unreachable")
                }
                inputEventType = input_read(self.inputFd, &data)
            }
        }
    }

    deinit {
        self.inputListenerContinue = false
        close(self.inputFd)
    }
}

