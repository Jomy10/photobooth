import CairoGraphics
import Cairo
import Utils
import CairoJPG
import SCCCairo

@_exported import class CairoGraphics.CairoContext

public struct PhotoboothGraphicsContext {
    public let innerContext: CairoContext
    public let width: Int
    public let height: Int
    let clearColor: UInt32
    let foregroundColor: UInt32

    let fontFamily = "Sans"
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

    public func drawText(_ message: String, textSize: Double = 108, fontWeight: FontWeight = .bold) {
        self.innerContext.markImageAsUnflushed()

        self.innerContext.context.setSource(color: Color(rgb: self.foregroundColor).asDoubleTuple)
        self.innerContext.context.setFont(size: textSize)
        self.innerContext.context.setFont(face: (family: self.fontFamily, slant: .normal, weight: fontWeight))

        // Support multiple lines
        let messages: [Substring] = message.split(separator: "\n")

        for i in (0..<messages.count) {
            let messageSubstring: Substring = messages[i]
            let _message = String(messageSubstring)

            var extents = cairo_text_extents_t()
            cairo_text_extents(self.innerContext.context.internalPointer, _message, &extents)


            let textPosX: Double = Double(self.width) / 2 - (extents.width / 2 + extents.x_bearing)
            let middleOfScreen: Double = Double(self.height) / 2
            // TODO: suck less at math
            let textYOffset: Double
            switch messages.count {
                case 1:
                    textYOffset = -(extents.height / 2 + extents.y_bearing)
                case 2:
                    textYOffset = -((i == 0 ? extents.height : 0) + extents.y_bearing)
                case 3:
                    textYOffset = -((i == 0 ? extents.height / 2 + extents.height : (i == 1 ? extents.height / 2 : -extents.height / 2)) + extents.y_bearing)
                default:
                    textYOffset = 0
            }
            let textPosY: Double = middleOfScreen + textYOffset
            self.innerContext.context.move(to: (x: textPosX, y: textPosY))

            self.innerContext.context.show(text: _message)
        }
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

