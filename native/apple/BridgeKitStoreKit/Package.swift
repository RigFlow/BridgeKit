// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "BridgeKitStoreKit",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "BridgeKitStoreKit",
            type: .dynamic,
            targets: ["BridgeKitStoreKit"]
        )
    ],
    targets: [
        .target(name: "BridgeKitStoreKit"),
        .testTarget(
            name: "BridgeKitStoreKitTests",
            dependencies: ["BridgeKitStoreKit"]
        )
    ]
)
