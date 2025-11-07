import SwiftUI

struct ValidationView: View {
    let errors: [ValidationErrorInfo]
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 16) {
            HStack {
                Image(systemName: errors.isEmpty ? "checkmark.circle.fill" : "exclamationmark.triangle.fill")
                    .font(.largeTitle)
                    .foregroundColor(errors.isEmpty ? .green : .orange)

                Text(errors.isEmpty ? "Plan is Valid" : "Validation Issues")
                    .font(.title)
                    .fontWeight(.bold)
            }
            .padding(.top)

            if errors.isEmpty {
                Text("No validation errors or warnings found.")
                    .foregroundColor(.secondary)
                    .padding()
            } else {
                List(errors) { error in
                    ValidationErrorRow(error: error)
                }
            }

            Button("Close") {
                dismiss()
            }
            .keyboardShortcut(.escape)
            .buttonStyle(.borderedProminent)
            .padding(.bottom)
        }
        .frame(width: 500, height: 400)
    }
}

struct ValidationErrorRow: View {
    let error: ValidationErrorInfo

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: "xmark.circle.fill")
                .foregroundColor(.red)

            VStack(alignment: .leading, spacing: 4) {
                Text(error.displayMessage)
                    .font(.body)

                Text(error.path)
                    .font(.caption)
                    .foregroundColor(.secondary)

                if let context = error.context {
                    Text(context)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(.vertical, 4)
    }
}
