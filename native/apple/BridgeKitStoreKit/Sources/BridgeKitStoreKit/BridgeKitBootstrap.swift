import Foundation

#if canImport(AppKit)
import AppKit
#endif

#if canImport(UIKit)
import UIKit
#endif

private final class BridgeKitBootstrapState {
    static let shared = BridgeKitBootstrapState()

    #if canImport(AppKit)
    var appKitDelegate: BridgeKitAppKitApplicationDelegate?
    #endif

    #if canImport(UIKit)
    var uiKitDelegate: BridgeKitUIKitApplicationDelegate?
    #endif
}

#if canImport(AppKit)
@objc private final class BridgeKitAppKitApplicationDelegate: NSObject, NSApplicationDelegate {
    func application(
        _ application: NSApplication,
        didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
    ) {
        forwardDeviceToken(deviceToken)
    }

    func application(
        _ application: NSApplication,
        didFailToRegisterForRemoteNotificationsWithError error: Error
    ) {
        NSLog("BridgeKit APNs registration failed: \(error.localizedDescription)")
    }
}
#endif

#if canImport(UIKit)
private final class BridgeKitUIKitApplicationDelegate: NSObject, UIApplicationDelegate {
    func application(
        _ application: UIApplication,
        didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
    ) {
        forwardDeviceToken(deviceToken)
    }

    func application(
        _ application: UIApplication,
        didFailToRegisterForRemoteNotificationsWithError error: Error
    ) {
        NSLog("BridgeKit APNs registration failed: \(error.localizedDescription)")
    }
}
#endif

private func forwardDeviceToken(_ deviceToken: Data) {
    let tokenHex = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
    bridgekitApnsForwardDeviceTokenHex(tokenHex)
}

@_cdecl("bridgekit_apple_bootstrap")
public func bridgekitAppleBootstrap() {
    #if canImport(AppKit)
    let delegate = BridgeKitAppKitApplicationDelegate()
    BridgeKitBootstrapState.shared.appKitDelegate = delegate
    NSApplication.shared.delegate = delegate
    #elseif canImport(UIKit)
    let delegate = BridgeKitUIKitApplicationDelegate()
    BridgeKitBootstrapState.shared.uiKitDelegate = delegate
    UIApplication.shared.delegate = delegate
    #endif
}
