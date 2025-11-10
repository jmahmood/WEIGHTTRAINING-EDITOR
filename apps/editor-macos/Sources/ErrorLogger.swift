import Foundation

class ErrorLogger {
    static let shared = ErrorLogger()

    private let logFileURL: URL

    private init() {
        let logsDir = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("com.weightlifting.editor")
            .appendingPathComponent("Logs")

        try? FileManager.default.createDirectory(at: logsDir, withIntermediateDirectories: true)

        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd"
        let dateString = dateFormatter.string(from: Date())

        logFileURL = logsDir.appendingPathComponent("error-\(dateString).log")

        // Log the file location on startup
        log("=== Application Started ===")
        log("Log file location: \(logFileURL.path)")
        NSLog("ERROR LOG FILE: %@", logFileURL.path)
    }

    func log(_ message: String, level: String = "INFO") {
        let timestamp = ISO8601DateFormatter().string(from: Date())
        let logMessage = "[\(timestamp)] [\(level)] \(message)\n"

        // Also log to console
        NSLog("[%@] %@", level, message)

        // Write to file
        if let data = logMessage.data(using: .utf8) {
            if FileManager.default.fileExists(atPath: logFileURL.path) {
                if let fileHandle = try? FileHandle(forWritingTo: logFileURL) {
                    fileHandle.seekToEndOfFile()
                    fileHandle.write(data)
                    fileHandle.closeFile()
                }
            } else {
                try? data.write(to: logFileURL)
            }
        }
    }

    func error(_ message: String) {
        log(message, level: "ERROR")
    }

    func warning(_ message: String) {
        log(message, level: "WARN")
    }

    func info(_ message: String) {
        log(message, level: "INFO")
    }

    func getLogFilePath() -> String {
        return logFileURL.path
    }
}
