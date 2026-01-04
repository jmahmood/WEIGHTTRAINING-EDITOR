import SwiftUI

struct SegmentOverrideFields: View {
    @Binding var overrides: [String: JSONValue]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            repsOverride
            rpeOverride
            restOverride
            tempoOverride
            notesOverride
        }
    }

    private var repsOverride: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Reps Override")
                .font(.caption)
            if let reps = overrides["reps"] {
                switch reps {
                case .object(let obj):
                    let minValue = obj["min"]?.intValue ?? 8
                    let maxValue = obj["max"]?.intValue ?? 12
                    HStack {
                        Stepper("Min: \(minValue)", value: bindingRepsRange(key: "min", fallback: minValue), in: 1...100)
                        Stepper("Max: \(maxValue)", value: bindingRepsRange(key: "max", fallback: maxValue), in: 1...100)
                    }
                    Button("Clear Reps Override") {
                        overrides.removeValue(forKey: "reps")
                    }
                    .buttonStyle(.borderless)
                case .number:
                    let value = reps.intValue ?? 8
                    Stepper("Reps: \(value)", value: bindingInt(key: "reps", fallback: value), in: 1...100)
                    Button("Clear Reps Override") {
                        overrides.removeValue(forKey: "reps")
                    }
                    .buttonStyle(.borderless)
                default:
                    Button("Clear Reps Override") {
                        overrides.removeValue(forKey: "reps")
                    }
                    .buttonStyle(.borderless)
                }
            } else {
                HStack(spacing: 8) {
                    Button("Add Reps") {
                        overrides["reps"] = .number(8)
                    }
                    Button("Add Rep Range") {
                        overrides["reps"] = .object([
                            "min": .number(8),
                            "max": .number(12)
                        ])
                    }
                }
            }
        }
    }

    private var rpeOverride: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("RPE Override")
                .font(.caption)
            if let rpe = overrides["rpe"]?.doubleValue {
                HStack {
                    Slider(value: bindingDouble(key: "rpe", fallback: rpe), in: 6...10, step: 0.5)
                    Text(String(format: "%.1f", rpe))
                        .frame(width: 40)
                    Button("Clear") {
                        overrides.removeValue(forKey: "rpe")
                    }
                    .buttonStyle(.borderless)
                }
            } else {
                Button("Add RPE Override") {
                    overrides["rpe"] = .number(8.0)
                }
                .buttonStyle(.borderless)
            }
        }
    }

    private var restOverride: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Rest (sec) Override")
                .font(.caption)
            if let rest = overrides["rest_sec"]?.intValue {
                Stepper("Rest: \(rest)", value: bindingInt(key: "rest_sec", fallback: rest), in: 0...600, step: 5)
                Button("Clear") {
                    overrides.removeValue(forKey: "rest_sec")
                }
                .buttonStyle(.borderless)
            } else {
                Button("Add Rest Override") {
                    overrides["rest_sec"] = .number(90)
                }
                .buttonStyle(.borderless)
            }
        }
    }

    private var tempoOverride: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Tempo Override")
                .font(.caption)
            if let tempo = overrides["tempo"]?.stringValue {
                TextField("Tempo", text: bindingString(key: "tempo", fallback: tempo))
                    .textFieldStyle(.roundedBorder)
                Button("Clear") {
                    overrides.removeValue(forKey: "tempo")
                }
                .buttonStyle(.borderless)
            } else {
                Button("Add Tempo Override") {
                    overrides["tempo"] = .string("30X1")
                }
                .buttonStyle(.borderless)
            }
        }
    }

    private var notesOverride: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Notes Override")
                .font(.caption)
            if let note = overrides["note"]?.stringValue {
                TextEditor(text: bindingString(key: "note", fallback: note))
                    .frame(height: 60)
                    .border(Color.gray.opacity(0.3))
                Button("Clear") {
                    overrides.removeValue(forKey: "note")
                }
                .buttonStyle(.borderless)
            } else {
                Button("Add Notes Override") {
                    overrides["note"] = .string("")
                }
                .buttonStyle(.borderless)
            }
        }
    }

    private func bindingString(key: String, fallback: String) -> Binding<String> {
        Binding(
            get: { overrides[key]?.stringValue ?? fallback },
            set: { overrides[key] = .string($0) }
        )
    }

    private func bindingInt(key: String, fallback: Int) -> Binding<Int> {
        Binding(
            get: { overrides[key]?.intValue ?? fallback },
            set: { overrides[key] = .number(Double($0)) }
        )
    }

    private func bindingRepsRange(key: String, fallback: Int) -> Binding<Int> {
        Binding(
            get: {
                if case .object(let obj) = overrides["reps"], let value = obj[key]?.intValue {
                    return value
                }
                return fallback
            },
            set: { newValue in
                var minValue = fallback
                var maxValue = fallback
                if case .object(let obj) = overrides["reps"] {
                    minValue = obj["min"]?.intValue ?? minValue
                    maxValue = obj["max"]?.intValue ?? maxValue
                }
                if key == "min" {
                    minValue = newValue
                } else {
                    maxValue = newValue
                }
                overrides["reps"] = .object([
                    "min": .number(Double(minValue)),
                    "max": .number(Double(maxValue))
                ])
            }
        )
    }

    private func bindingDouble(key: String, fallback: Double) -> Binding<Double> {
        Binding(
            get: { overrides[key]?.doubleValue ?? fallback },
            set: { overrides[key] = .number($0) }
        )
    }
}
