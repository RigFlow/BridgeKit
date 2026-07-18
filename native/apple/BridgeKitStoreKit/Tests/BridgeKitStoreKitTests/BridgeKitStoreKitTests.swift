import XCTest
@testable import BridgeKitStoreKit

final class BridgeKitStoreKitTests: XCTestCase {
    func testEmptyProductRequestReturnsEmptyArray() {
        let response = #"{"productIds":[]}"#.withCString { pointer in
            bridgekitStoreKitProductsJson(pointer)
        }

        XCTAssertNotNil(response)
        defer {
            bridgekitStringFree(response)
        }
        XCTAssertEqual(String(cString: response!), "[]")
    }
}
