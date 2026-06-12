// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ExifRmRust",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "ExifRmRust", targets: ["ExifRmRust"]),
    ],
    targets: [
        .binaryTarget(
            name: "exif_rmFFI",
            path: "Sources/exif_rmFFI/exif_rmFFI.xcframework"
        ),
        .target(
            name: "ExifRmRust",
            dependencies: ["exif_rmFFI"],
            path: "Sources/ExifRmRust"
        ),
    ]
)
