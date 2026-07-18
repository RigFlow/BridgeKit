import Foundation
import UserNotifications

#if canImport(UIKit)
import UIKit
#endif

#if canImport(AppKit)
import AppKit
#endif

private struct BridgeKitPushAuthorizationRequest: Decodable {
    let alert: Bool
    let badge: Bool
    let sound: Bool
    let provisional: Bool
}

private struct BridgeKitPushAuthorizationResponse: Encodable {
    let status: String
    let metadata: [String: String]
}

private struct BridgeKitPushRegistrationRequest: Decodable {
    let environment: String?
}

private struct BridgeKitPushRegistrationResponse: Encodable {
    let deviceToken: String
    let environment: String?
    let expiresAtMs: Int64?
    let raw: [String: String]
}

private final class BridgeKitAPNsTokenBroker {
    static let shared = BridgeKitAPNsTokenBroker()

    private let lock = NSLock()
    private var pendingSemaphore: DispatchSemaphore?
    private var deviceTokenHex: String?

    func storeDeviceTokenHex(_ tokenHex: String) {
        lock.lock()
        deviceTokenHex = tokenHex
        let semaphore = pendingSemaphore
        lock.unlock()
        semaphore?.signal()
    }

    func clearDeviceToken() {
        lock.lock()
        deviceTokenHex = nil
        lock.unlock()
    }

    func awaitDeviceToken(timeoutSeconds: Double) -> String? {
        lock.lock()
        if let existing = deviceTokenHex {
            lock.unlock()
            return existing
        }

        let semaphore = DispatchSemaphore(value: 0)
        pendingSemaphore = semaphore
        lock.unlock()

        requestRemoteNotificationRegistrationOnMainThread()

        let timeout = DispatchTime.now() + timeoutSeconds
        let result = semaphore.wait(timeout: timeout)
        lock.lock()
        pendingSemaphore = nil
        let token = result == .success ? deviceTokenHex : nil
        lock.unlock()
        return token
    }
}

@_cdecl("bridgekit_apns_forward_device_token_hex")
public func bridgekitApnsForwardDeviceTokenHex(_ tokenHex: UnsafePointer<CChar>?) {
    guard let tokenHex else {
        return
    }

    BridgeKitAPNsTokenBroker.shared.storeDeviceTokenHex(String(cString: tokenHex))
}

@_cdecl("bridgekit_apns_request_authorization_json")
public func bridgekitApnsRequestAuthorizationJson(
    _ requestJson: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>? {
    guard let requestJson else {
        return nil
    }

    let requestString = String(cString: requestJson)

    do {
        let requestData = Data(requestString.utf8)
        let request = try JSONDecoder().decode(
            BridgeKitPushAuthorizationRequest.self,
            from: requestData
        )
        let response = try requestAuthorizationSynchronously(request: request)
        let responseData = try JSONEncoder.bridgeKit.encode(response)
        guard let responseString = String(data: responseData, encoding: .utf8) else {
            return nil
        }
        return strdup(responseString)
    } catch {
        return nil
    }
}

@_cdecl("bridgekit_apns_register_json")
public func bridgekitApnsRegisterJson(_ requestJson: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>? {
    guard let requestJson else {
        return nil
    }

    let requestString = String(cString: requestJson)

    do {
        let requestData = Data(requestString.utf8)
        let request = try JSONDecoder().decode(
            BridgeKitPushRegistrationRequest.self,
            from: requestData
        )
        let response = try registerSynchronously(request: request)
        let responseData = try JSONEncoder.bridgeKit.encode(response)
        guard let responseString = String(data: responseData, encoding: .utf8) else {
            return nil
        }
        return strdup(responseString)
    } catch {
        return nil
    }
}

@_cdecl("bridgekit_apns_unregister")
public func bridgekitApnsUnregister() {
    unregisterSynchronously()
}

private func requestAuthorizationSynchronously(
    request: BridgeKitPushAuthorizationRequest
) throws -> BridgeKitPushAuthorizationResponse {
    let semaphore = DispatchSemaphore(value: 0)
    let lock = NSLock()
    var result: Result<BridgeKitPushAuthorizationResponse, Error>?

    var options: UNAuthorizationOptions = []
    if request.alert {
        options.insert(.alert)
    }
    if request.badge {
        options.insert(.badge)
    }
    if request.sound {
        options.insert(.sound)
    }
    if request.provisional {
        options.insert(.provisional)
    }

    UNUserNotificationCenter.current().requestAuthorization(options: options) { _, error in
        if let error {
            lock.lock()
            result = .failure(error)
            lock.unlock()
            semaphore.signal()
            return
        }

        UNUserNotificationCenter.current().getNotificationSettings { settings in
            let response = BridgeKitPushAuthorizationResponse(
                status: settings.authorizationStatus.bridgeKitStatus,
                metadata: [
                    "source": "apns",
                    "authorizationStatus": String(describing: settings.authorizationStatus)
                ]
            )
            lock.lock()
            result = .success(response)
            lock.unlock()
            semaphore.signal()
        }
    }

    semaphore.wait()
    lock.lock()
    defer { lock.unlock() }
    return try result!.get()
}

private func registerSynchronously(
    request: BridgeKitPushRegistrationRequest
) throws -> BridgeKitPushRegistrationResponse {
    guard let deviceToken = BridgeKitAPNsTokenBroker.shared.awaitDeviceToken(timeoutSeconds: 10) else {
        throw BridgeKitAPNsError.deviceTokenUnavailable
    }

    return BridgeKitPushRegistrationResponse(
        deviceToken: deviceToken,
        environment: request.environment,
        expiresAtMs: nil,
        raw: [
            "source": "apns"
        ]
    )
}

private func unregisterSynchronously() {
    let work = {
        #if canImport(UIKit)
        UIApplication.shared.unregisterForRemoteNotifications()
        #endif
    }

    if Thread.isMainThread {
        work()
    } else {
        DispatchQueue.main.sync(execute: work)
    }

    BridgeKitAPNsTokenBroker.shared.clearDeviceToken()
}

private func requestRemoteNotificationRegistrationOnMainThread() {
    let work = {
        #if canImport(UIKit)
        UIApplication.shared.registerForRemoteNotifications()
        #elseif canImport(AppKit)
        NSApplication.shared.registerForRemoteNotifications()
        #endif
    }

    if Thread.isMainThread {
        work()
    } else {
        DispatchQueue.main.sync(execute: work)
    }
}

private enum BridgeKitAPNsError: Error {
    case deviceTokenUnavailable
}

private extension UNAuthorizationStatus {
    var bridgeKitStatus: String {
        switch self {
        case .notDetermined:
            "not_determined"
        case .denied:
            "denied"
        case .authorized:
            "authorized"
        case .provisional:
            "provisional"
        case .ephemeral:
            "ephemeral"
        @unknown default:
            "unsupported"
        }
    }
}
