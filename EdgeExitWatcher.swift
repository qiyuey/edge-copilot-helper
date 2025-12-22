import Cocoa
import Foundation

class AppObserver: NSObject {
    let scriptPath: String

    init(scriptPath: String) {
        self.scriptPath = scriptPath
        super.init()
        // Register for application termination notifications
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

        // Check if the terminated app is Microsoft Edge
        // Common Bundle IDs: com.microsoft.edgemac, com.microsoft.edgemac.Canary, com.microsoft.edgemac.Beta, com.microsoft.edgemac.Dev
        if bundleId.contains("com.microsoft.edgemac") || name == "Microsoft Edge" {
            print("🛑 Detected termination of: \(name) (\(bundleId))")
            print("🚀 Triggering fix script...")
            runFixScript()
        }
    }

    func runFixScript() {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: scriptPath)
        
        // Since this runs in background, we might want to log output
        // For now, we inherit stdout/stderr so it goes to the system log/console if viewing
        process.standardOutput = FileHandle.standardOutput
        process.standardError = FileHandle.standardError

        do {
            try process.run()
            // We don't wait until exit to avoid blocking the main thread for too long, 
            // but since we are just a watcher, blocking is fine as long as we process the next event.
            // Actually, waiting is better to ensure we don't spawn multiple overlapping instances if user rapidly opens/closes.
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

