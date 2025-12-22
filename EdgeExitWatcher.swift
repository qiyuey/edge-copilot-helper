import Cocoa
import Foundation

class AppObserver: NSObject {
    let scriptPath: String

    init(scriptPath: String) {
        self.scriptPath = scriptPath
        super.init()
        // Watch for app termination
        NSWorkspace.shared.notificationCenter.addObserver(
            self,
            selector: #selector(appDidTerminate(_:)),
            name: NSWorkspace.didTerminateApplicationNotification,
            object: nil
        )
        print("👀 EdgeExitWatcher started.")
        print("   Monitoring for: Microsoft Edge")
        print("   Action script: \(scriptPath)")
    }

    @objc func appDidTerminate(_ notification: Notification) {
        guard let app = notification.userInfo?[NSWorkspace.applicationUserInfoKey] as? NSRunningApplication else { return }
        
        let name = app.localizedName ?? "Unknown"
        let bundleId = app.bundleIdentifier ?? "Unknown"

        // Check for Edge
        if bundleId.contains("com.microsoft.edgemac") || name == "Microsoft Edge" {
            print("🛑 Detected termination of: \(name) (\(bundleId))")
            print("🚀 Triggering fix script...")
            runFixScript()
        }
    }

    func runFixScript() {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: scriptPath)
        
        // Inherit IO
        process.standardOutput = FileHandle.standardOutput
        process.standardError = FileHandle.standardError

        do {
            try process.run()
            process.waitUntilExit()
            print("✅ Script finished with exit code: \(process.terminationStatus)")
        } catch {
            print("❌ Failed to launch script: \(error)")
        }
    }
}

// Ensure we have the script path argument
guard CommandLine.arguments.count > 1 else {
    print("Usage: EdgeExitWatcher <absolute_path_to_script>")
    exit(1)
}

let scriptPath = CommandLine.arguments[1]

// Create the observer
let observer = AppObserver(scriptPath: scriptPath)

// Start the run loop to listen for events
RunLoop.main.run()

