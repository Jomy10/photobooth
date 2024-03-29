import CairoGraphics
import Cairo
import Utils
import CairoJPG

@_exported import class CairoGraphics.CairoContext

public struct PhotoboothGraphicsContext {
    public let innerContext: CairoContext
    public let width: Int
    public let height: Int
    internal let clearColor: UInt32
    internal let foregroundColor: UInt32
}

extension PhotoboothGraphicsContext {
    public init(
        from buffer: UnsafeMutablePointer<UInt8>,
        width: Int, height: Int, stride: Int? = nil,
        format: ImageFormat = .argb32,
        defaultClearColor: UInt32 = 0xFF000000,
        defaultForegroundColor: UInt32? = nil
    ) throws {
        let surface: Cairo.Surface.Image = try Cairo.Surface.Image(
            mutableBytes: buffer,
            format: format,
            width: width, height: height,
            stride: stride ?? format.stride(for: width)
        )
        let image = CairoGraphics.CairoImage(rawSurface: surface)

        self.innerContext = CairoGraphics.CairoContext(image: image)
        self.width = width
        self.height = height
        self.clearColor = defaultClearColor
        self.foregroundColor = defaultForegroundColor ?? ~defaultClearColor
    }

    public func clearBackground(_ color: UInt32? = nil) {
        self.innerContext.draw(rect: Rectangle(
            size: Vec2(x: Double(self.width), y: Double(self.height)),
            color: Color(rgb: color ?? self.clearColor),
            isFilled: true
        ))
    }

    public func drawText(_ message: String, textSize: Double = 108) {
                let textPosX: Double = (Double(self.width) - 1400) / 2.0
                let textPosY: Double = (Double(self.height) + textSize) / 2.0
                let textPos: Vec2 = Vec2(
                    x: textPosX,
                    y: textPosY 
                )

        self.innerContext.draw(text: Text(
            message,
            withSize: textSize,
            at: textPos,
            color: Color(rgb: self.foregroundColor)
        ))
    }

    public func drawJPEG(_ filename: String) throws {
        let surface = try Surface.Image(internalPointer: cairo_image_surface_create_from_jpeg(filename))
        let image = CairoImage(rawSurface: surface)
        self.innerContext.draw(
            image: image,
            at: Vec2(x: 0, y: 0),
            withSize: Vec2(x: self.width, y: self.height)
        )
    }
}

