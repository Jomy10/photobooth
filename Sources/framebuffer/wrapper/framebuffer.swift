import Foundation
import drm
import Glibc

public final class Framebuffer {
    public final var fd: Int32 = -1
    public final let dev: UnsafeMutablePointer<modeset_dev>
    public final let bufferSize: Int
    public final let gpuCard = "/dev/dri/by-path/platform-gpu-card"
    public final let size:  (width: Int32, height: Int32)

    public enum Error: Swift.Error {
        case DRMDeviceCreationFailed
        case DRMDevicePrepareFailed
        case noBuffer(for: UnsafeMutablePointer<modeset_dev>)
    }

    public init() throws {
        var ret: Int32 = -1
        var iter: UnsafeMutablePointer<modeset_dev>? = nil

        ret = modeset_open(&self.fd, card)
        if (ret != 0) {
            throw Self.Error.DRMDeviceCreationFailed
        }

        ret = modeset_prepare(self.fd)
        if (ret != 0) {
            close(self.fd)
            throw Self.Error.DRMDevicePrepareFailed
        }

        iter = get_modeset_list()
        while let dev = iter {
            print("Device: \(dev)")
            dev.pointee.saved_crtc = drmModeGetCrtc(self.fd, dev.pointee.crtc)
            guard let buf = modeset_dev_front_buf(&dev) else {
                throw Self.Error.noBuffer(for: dev)
            }
            ret = drmModeSetCrtc(
                self.fd,
                dev.pointee.crtc,
                dev.pointee.fb,
                0, 0, // x, y
                withUnsafeMutablePointer(to: &dev.pointee.conn) { $0 },
                1,
                withUnsafeMutablePointer(to: &dev.pointee.mode) { $0 }
            )
            if ret != 0 {
                print("[WARN] Cannot set crtc for connector: \(dev.pointee.conn) (\(errno): \(String(cString: strerror(errno))))")
            }
            iter = dev.pointee.next
        }
        self.dev = get_modeset_list()
        self.size = (
            width: Int32(self.dev.pointee.bufs.0.width),
            height: Int32(self.dev.pointee.bufs.0.height)
        )
        self.bufferSize = Int(self.size.w * self.size.h)
    }

    public func swapBuffers() throws {
        let ret = drmModeSetCrtc(
            self.fd,
            self.dev.pointee.crtc,
            modeset_dev_back_buf(&dev).pointee.fb,
            0, 0,
            withUnsafeMutablePointer(to: &dev.pointee.conn) { $0 },
            1,
            withUnsafeMutablePointer(to: &dev.pointee.mode) { $0 }
        )
        if ret != 0 {
            print("Cannot flip CRTC for connector \(self.dev.pointee.conn) (\(errno): \(String(cString: strerror(errno))))")
        } else {
            self.dev.pointee.front_buf ^= 1
        }
    }

    public var backBuffer: UnsafeMutablePointer<modeset_buf> {
        modeset_dev_back_buf(&dev)
    }

    deinit {
        modeset_cleanup(self.fd)
        close(self.fd)
    }
}

