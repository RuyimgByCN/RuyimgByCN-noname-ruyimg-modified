import AppKit
import ApplicationServices
import CoreGraphics
import Darwin
import Foundation

func emit(_ value: [String: Any]) {
    do {
        let data = try JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
        print(String(decoding: data, as: UTF8.self))
        fflush(stdout)
    } catch {
        fputs("Cannot encode HID controller response: \(error)\n", stderr)
        exit(3)
    }
}

func frameFor(_ application: AXUIElement) throws -> CGRect {
    var windows: CFTypeRef?
    guard AXUIElementCopyAttributeValue(application, kAXWindowsAttribute as CFString, &windows) == .success,
          let entries = windows as? [AXUIElement], let window = entries.first else {
        throw NSError(domain: "PA09HID", code: 1, userInfo: [NSLocalizedDescriptionKey: "No accessible target window"])
    }
    var position: CFTypeRef?
    var size: CFTypeRef?
    guard AXUIElementCopyAttributeValue(window, kAXPositionAttribute as CFString, &position) == .success,
          AXUIElementCopyAttributeValue(window, kAXSizeAttribute as CFString, &size) == .success,
          let position, let size,
          CFGetTypeID(position) == AXValueGetTypeID(), CFGetTypeID(size) == AXValueGetTypeID() else {
        throw NSError(domain: "PA09HID", code: 2, userInfo: [NSLocalizedDescriptionKey: "Target geometry unavailable"])
    }
    var point = CGPoint.zero
    var dimensions = CGSize.zero
    guard AXValueGetValue(unsafeBitCast(position, to: AXValue.self), .cgPoint, &point),
          AXValueGetValue(unsafeBitCast(size, to: AXValue.self), .cgSize, &dimensions) else {
        throw NSError(domain: "PA09HID", code: 3, userInfo: [NSLocalizedDescriptionKey: "Target geometry has wrong type"])
    }
    return CGRect(origin: point, size: dimensions)
}

// Preflight never invokes a permission-request API or captures the screen.
guard CommandLine.arguments.count == 2, let processID = Int32(CommandLine.arguments[1]),
      AXIsProcessTrusted(), CGPreflightPostEventAccess(),
      let application = NSRunningApplication(processIdentifier: processID) else {
    emit(["error": "Usage: hid-tool target-pid; existing AX/post-event permission required"])
    exit(2)
}
application.activate(options: [.activateIgnoringOtherApps])
RunLoop.main.run(until: Date(timeIntervalSinceNow: 0.3))
let element = AXUIElementCreateApplication(processID)
do {
    let frame = try frameFor(element)
    emit(["frame": ["x": frame.minX, "y": frame.minY, "width": frame.width, "height": frame.height], "pid": processID])
    while let line = readLine() {
        guard let data = line.data(using: .utf8),
              let request = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let pointX = request["x"] as? Double, let pointY = request["y"] as? Double,
              let sequence = request["sequence"] as? Int, pointX.isFinite, pointY.isFinite else {
            throw NSError(domain: "PA09HID", code: 4, userInfo: [NSLocalizedDescriptionKey: "Invalid click request"])
        }
        let current = try frameFor(element)
        guard NSWorkspace.shared.frontmostApplication?.processIdentifier == processID,
              abs(current.minX - frame.minX) < 1, abs(current.minY - frame.minY) < 1,
              abs(current.width - frame.width) < 1, abs(current.height - frame.height) < 1,
              frame.contains(CGPoint(x: pointX, y: pointY)) else {
            throw NSError(domain: "PA09HID", code: 5, userInfo: [NSLocalizedDescriptionKey: "Target focus/geometry changed; no input posted"])
        }
        let point = CGPoint(x: pointX, y: pointY)
        guard let down = CGEvent(mouseEventSource: nil, mouseType: .leftMouseDown, mouseCursorPosition: point, mouseButton: .left),
              let up = CGEvent(mouseEventSource: nil, mouseType: .leftMouseUp, mouseCursorPosition: point, mouseButton: .left) else {
            throw NSError(domain: "PA09HID", code: 6, userInfo: [NSLocalizedDescriptionKey: "Cannot construct input events"])
        }
        down.post(tap: .cghidEventTap)
        usleep(8_000)
        up.post(tap: .cghidEventTap)
        emit(["posted": true, "sequence": sequence])
    }
} catch {
    emit(["error": error.localizedDescription])
    exit(1)
}
