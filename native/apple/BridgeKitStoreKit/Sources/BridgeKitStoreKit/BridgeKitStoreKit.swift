import Foundation

#if canImport(StoreKit)
import StoreKit
#endif

#if os(Linux)
import Glibc
#else
import Darwin
#endif

private struct BridgeKitProductRequest: Decodable {
    let productIds: [String]
}

private struct BridgeKitProductResponse: Encodable {
    let id: String
    let localizedTitle: String
    let localizedDescription: String
    let displayPrice: String
    let currencyCode: String
    let subscriptionPeriod: String?
    let introductoryPrice: String?
    let trialPeriod: String?
    let raw: [String: String]
}

@_cdecl("bridgekit_storekit_products_json")
public func bridgekitStoreKitProductsJson(_ requestJson: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>? {
    guard let requestJson else {
        return nil
    }

    let requestString = String(cString: requestJson)

    do {
        let requestData = Data(requestString.utf8)
        let request = try JSONDecoder().decode(BridgeKitProductRequest.self, from: requestData)
        let products = try fetchProductsSynchronously(productIds: request.productIds)
        let responseData = try JSONEncoder.bridgeKit.encode(products)
        guard let responseString = String(data: responseData, encoding: .utf8) else {
            return nil
        }
        return strdup(responseString)
    } catch {
        return nil
    }
}

@_cdecl("bridgekit_string_free")
public func bridgekitStringFree(_ value: UnsafeMutablePointer<CChar>?) {
    free(value)
}

private func fetchProductsSynchronously(productIds: [String]) throws -> [BridgeKitProductResponse] {
    if productIds.isEmpty {
        return []
    }

    #if canImport(StoreKit)
    if #available(iOS 15.0, macOS 12.0, *) {
        let semaphore = DispatchSemaphore(value: 0)
        let lock = NSLock()
        var result: Result<[BridgeKitProductResponse], Error>?

        Task.detached {
            do {
                let products = try await Product.products(for: productIds)
                let response = products.map(BridgeKitProductResponse.init(storeKitProduct:))
                lock.lock()
                result = .success(response)
                lock.unlock()
            } catch {
                lock.lock()
                result = .failure(error)
                lock.unlock()
            }
            semaphore.signal()
        }

        semaphore.wait()
        lock.lock()
        defer { lock.unlock() }
        return try result!.get()
    }
    #endif

    throw BridgeKitStoreKitError.storeKitUnavailable
}

private enum BridgeKitStoreKitError: Error {
    case storeKitUnavailable
}

private extension JSONEncoder {
    static var bridgeKit: JSONEncoder {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        return encoder
    }
}

#if canImport(StoreKit)
@available(iOS 15.0, macOS 12.0, *)
private extension BridgeKitProductResponse {
    init(storeKitProduct product: Product) {
        let subscription = product.subscription
        let introductoryOffer = subscription?.introductoryOffer

        self.init(
            id: product.id,
            localizedTitle: product.displayName,
            localizedDescription: product.description,
            displayPrice: product.displayPrice,
            currencyCode: product.priceFormatStyle.currencyCode,
            subscriptionPeriod: subscription?.subscriptionPeriod.bridgeKitISO8601,
            introductoryPrice: introductoryOffer?.displayPrice,
            trialPeriod: introductoryOffer?.period.bridgeKitISO8601,
            raw: [
                "source": "storekit",
                "type": product.type.bridgeKitRawValue
            ]
        )
    }
}

@available(iOS 15.0, macOS 12.0, *)
private extension Product.SubscriptionPeriod {
    var bridgeKitISO8601: String {
        switch unit {
        case .day:
            "P\(value)D"
        case .week:
            "P\(value)W"
        case .month:
            "P\(value)M"
        case .year:
            "P\(value)Y"
        @unknown default:
            "P\(value)D"
        }
    }
}

@available(iOS 15.0, macOS 12.0, *)
private extension Product.ProductType {
    var bridgeKitRawValue: String {
        switch self {
        case .consumable:
            "consumable"
        case .nonConsumable:
            "non_consumable"
        case .nonRenewable:
            "non_renewable"
        case .autoRenewable:
            "auto_renewable"
        @unknown default:
            "unknown"
        }
    }
}
#endif
