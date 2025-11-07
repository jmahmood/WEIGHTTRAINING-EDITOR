import SwiftUI
import UniformTypeIdentifiers

/// Document wrapper for SwiftUI DocumentGroup
struct WeightliftingDocument: FileDocument {
    static var readableContentTypes: [UTType] { [.json] }

    var planDocument: PlanDocument

    init() {
        self.planDocument = PlanDocument()
    }

    init(configuration: ReadConfiguration) throws {
        guard let data = configuration.file.regularFileContents,
              let json = String(data: data, encoding: .utf8) else {
            throw CocoaError(.fileReadCorruptFile)
        }

        self.planDocument = PlanDocument(json: json)
    }

    func fileWrapper(configuration: WriteConfiguration) throws -> FileWrapper {
        guard let data = planDocument.planJSON.data(using: .utf8) else {
            throw CocoaError(.fileWriteInapplicableStringEncoding)
        }

        return FileWrapper(regularFileWithContents: data)
    }
}
