// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "photobooth",
    dependencies: [
        .package(url: "https://github.com/jomy10/swift-graphics", branch: "main"),
        .package(url: "https://github.com/jomy10/swift-cairo", branch: "main"),
        .package(url: "https://github.com/fwcd/swift-utils.git", from: "3.0.0"),
        .package(url: "https://github.com/jpsim/Yams.git", from: "5.1.0")

        // .package(url: "https://github.com/troughton/Cstb", branch: "main"),
    ],
    targets: [
        .executableTarget(
            name: "Photobooth",
            dependencies: [
                "Framebuffer",
                "drm",

                "PhotoboothGraphics",

                "Input",
                
                "Yams",
            ]
        ),

        // Framebuffer //
        .target(
            name: "Framebuffer",
            dependencies: ["drm"],
            path: "Sources/Framebuffer/wrapper"
        ),
        .target(
            name: "drm",
            path: "Sources/Framebuffer/drm"
        ),

        // Input //
        .target(
            name: "Input",
            dependencies: [
                "LibInput",
                "Thread"
            ],
            path: "Sources/Input/wrapper"
        ),
        .target(
            name: "LibInput",
            path: "Sources/Input/LibInput"
        ),

        // Graphics //
        .target(
            name: "CairoJPG",
            path: "Sources/Graphics/CairoJPG"
        ),
        .target(
            name: "PhotoboothGraphics",
            dependencies: [
                .product(name: "CairoGraphics", package: "swift-graphics"),
                .product(name: "Cairo", package: "swift-cairo"),
                .product(name: "SCCCairo", package: "swift-cairo"),
                .product(name: "Utils", package: "swift-utils"),
                "CairoJPG",

                // .product(name: "stb_image", package: "Cstb"),
                // .product(name: "stb_image_resize", package: "Cstb"),
            ],
            path: "Sources/Graphics/Graphics"
        ),

        // Multithreading //
        .target(
            name: "Thread"
        ),
    ]
)

