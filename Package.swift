// swift-tools-version: 5.7

import PackageDescription

let package = Package(
    name: "photobooth",
    dependencies: [],
    targets: [
        .executableTarget(
            name: "photobooth-new",
            dependencies: [
                "framebuffer"
            ]
        ),
        .target(
            name: "drm",
            path: "Sources/framebuffer/drm"
        ),
        .target(
            name: "Framebuffer",
            dependencies: ["drm"],
            path: "Sources/framebuffer/wrapper"
        )
    ]
)

