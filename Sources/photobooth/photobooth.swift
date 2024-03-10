import Framebuffer
import Glibc

@main
public struct photobooth {
    public static func main() {
        let fb = Framebuffer()

        memset(fb.backBuf.pointee.map, 100, fb.pointee.size)
        fb.swapBuffers()
        memset(fb.backBuf.pointee.map, 200, fb.pointee.size)
        
        while (true) {
            fb.swapBuffers()
        }
    }
}

