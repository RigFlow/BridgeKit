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

    func testRestorePurchasesReturnsEmptyArrayWhenNoEntitlements() {
        let response = bridgekitStoreKitRestorePurchasesJson()

        XCTAssertNotNil(response)
        defer {
            bridgekitStringFree(response)
        }
        XCTAssertEqual(String(cString: response!), "[]")
    }

    func testValidateReceiptReturnsNotFoundForUnknownReceipt() {
        let request = #"{"receipt":"unknown-receipt"}"#.withCString { pointer in
            bridgekitStoreKitValidateReceiptJson(pointer)
        }

        XCTAssertNotNil(request)
        defer {
            bridgekitStringFree(request)
        }

        let response = String(cString: request!)
        XCTAssertTrue(response.contains("\"isValid\":false"))
        XCTAssertTrue(response.contains("\"verification\":\"not_found\""))
    }

    func testRegisterReturnsTokenAfterForwarding() {
        bridgekitApnsForwardDeviceTokenHex("abc123")

        let response = #"{"environment":"production"}"#.withCString { pointer in
            bridgekitApnsRegisterJson(pointer)
        }

        XCTAssertNotNil(response)
        defer {
            bridgekitStringFree(response)
        }

        let registration = String(cString: response!)
        XCTAssertTrue(registration.contains("\"deviceToken\":\"abc123\""))
    }
}
