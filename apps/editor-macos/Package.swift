// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "WeightliftingEditor",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(
            name: "WeightliftingEditor",
            targets: ["WeightliftingEditor"]
        )
    ],
    targets: [
        .systemLibrary(
            name: "CFFIBridge",
            path: "Sources/CFFIBridge"
        ),
        .executableTarget(
            name: "WeightliftingEditor",
            dependencies: ["CFFIBridge"],
            path: "Sources",
            exclude: ["CFFIBridge"],
            linkerSettings: [
                .unsafeFlags(["-L../../target/release"]),
                .unsafeFlags(["-Xlinker", "-rpath", "-Xlinker", "@executable_path/../Frameworks"]),
                .linkedLibrary("weightlifting_ffi")
            ]
        )
    ]
)
