import SwiftUI

struct SchemeSet: Identifiable {
    let id = UUID()
    var label: String = ""
    var sets: Int = 1
    var repsMin: Int = 8
    var repsMax: Int = 12
    var timeSec: Int? = nil
    var rpe: Double? = nil
    var restSec: Int? = nil
}

struct SegmentEditorView: View {
    @ObservedObject var plan: PlanDocument
    let segmentJSON: String?
    let dayIndex: Int
    let onSave: (String) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var selectedType = "straight"
    @State private var exerciseCode = ""
    @State private var exerciseName = ""
    @State private var sets = 3
    @State private var useRepsRange = false
    @State private var reps = 8
    @State private var repsMin = 10
    @State private var repsMax = 12
    @State private var straightRestSec = 120
    @State private var straightRpe = 8.5
    @State private var altGroup: String?
    @State private var commentText = ""

    // Superset/Circuit fields
    @State private var label = ""
    @State private var rounds = 2
    @State private var supersetRestSec = 60
    @State private var restBetweenRoundsSec = 60
    @State private var supersetItems: [[String: Any]] = []
    @State private var showingSupersetItemEditor = false
    @State private var editingSupersetItemIndex: Int?

    // Scheme fields
    @State private var schemeSets: [SchemeSet] = []
    @State private var schemeAltGroup: String?

    var body: some View {
        VStack(spacing: 20) {
            Text(segmentJSON == nil ? "Add Segment" : "Edit Segment")
                .font(.title2)
                .fontWeight(.bold)

            // Segment type picker
            Picker("Type", selection: $selectedType) {
                Text("Straight Sets").tag("straight")
                Text("RPE").tag("rpe")
                Text("Percentage").tag("percentage")
                Text("AMRAP").tag("amrap")
                Text("Superset").tag("superset")
                Text("Circuit").tag("circuit")
                Text("Scheme").tag("scheme")
                Text("Complex").tag("complex")
                Text("Comment").tag("comment")
                Text("Choose").tag("choose")
            }
            .pickerStyle(.menu)

            Divider()

            ScrollView {
                Form {
                    switch selectedType {
                    case "straight":
                        straightSegmentForm
                    case "superset", "circuit":
                        supersetSegmentForm
                    case "scheme":
                        schemeSegmentForm
                    case "comment":
                        commentSegmentForm
                    default:
                        Text("Editor for \(selectedType) segments coming soon...")
                            .foregroundColor(.secondary)
                    }
                }
                .frame(maxWidth: .infinity)
            }

            // Action buttons
            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.escape)

                Spacer()

                Button("Save") {
                    saveSegment(addAnother: false)
                }
                .keyboardShortcut(.return)
                .buttonStyle(.borderedProminent)

                if segmentJSON == nil {
                    Button("Save & Add Another") {
                        saveSegment(addAnother: true)
                    }
                    .keyboardShortcut(.return, modifiers: [.shift])
                }
            }
        }
        .padding()
        .frame(width: 520, height: 600)
        .onAppear {
            loadSegment()
        }
    }

    @ViewBuilder
    private var straightSegmentForm: some View {
        Section("Exercise") {
            ExercisePicker(
                plan: plan,
                selectedExerciseCode: $exerciseCode,
                selectedExerciseName: $exerciseName
            )
        }

        Section("Alternative Group (Optional)") {
            GroupPicker(plan: plan, selectedGroup: $altGroup)
        }

        Section("Sets & Reps") {
            Stepper("Sets: \(sets)", value: $sets, in: 1...20)

            Toggle("Use Rep Range", isOn: $useRepsRange)

            if useRepsRange {
                Stepper("Min: \(repsMin)", value: $repsMin, in: 1...100)
                Stepper("Max: \(repsMax)", value: $repsMax, in: 1...100)
            } else {
                Stepper("Reps: \(reps)", value: $reps, in: 1...100)
            }
        }

        Section("Rest & Effort") {
            Stepper("Rest Seconds: \(straightRestSec)", value: $straightRestSec, in: 30...600, step: 15)
            HStack {
                Text("RPE:")
                Slider(value: $straightRpe, in: 6...10, step: 0.5)
                Text(String(format: "%.1f", straightRpe))
                    .frame(width: 40)
            }
        }
    }

    @ViewBuilder
    private var schemeSegmentForm: some View {
        Section("Exercise") {
            ExercisePicker(
                plan: plan,
                selectedExerciseCode: $exerciseCode,
                selectedExerciseName: $exerciseName
            )
        }

        Section("Alternative Group (Optional)") {
            GroupPicker(plan: plan, selectedGroup: $schemeAltGroup)
        }

        Section("Scheme Sets") {
            List {
                ForEach(schemeSets.indices, id: \.self) { index in
                    DisclosureGroup {
                        VStack(alignment: .leading, spacing: 8) {
                            HStack {
                                Text("Sets:")
                                    .frame(width: 80, alignment: .trailing)
                                Stepper("\(schemeSets[index].sets)", value: $schemeSets[index].sets, in: 1...20)
                            }

                            HStack {
                                Text("Reps Range:")
                                    .frame(width: 80, alignment: .trailing)
                                Stepper("Min: \(schemeSets[index].repsMin)", value: $schemeSets[index].repsMin, in: 1...50)
                                Stepper("Max: \(schemeSets[index].repsMax)", value: $schemeSets[index].repsMax, in: 1...50)
                            }

                            HStack {
                                Text("Time (sec):")
                                    .frame(width: 80, alignment: .trailing)
                                Stepper("\(schemeSets[index].timeSec ?? 0)", value: Binding(
                                    get: { schemeSets[index].timeSec ?? 0 },
                                    set: { schemeSets[index].timeSec = $0 > 0 ? $0 : nil }
                                ), in: 0...600, step: 5)
                                if schemeSets[index].timeSec ?? 0 > 0 {
                                    Button("Clear") {
                                        schemeSets[index].timeSec = nil
                                    }
                                    .buttonStyle(.borderless)
                                }
                            }

                            HStack {
                                Text("RPE:")
                                    .frame(width: 80, alignment: .trailing)
                                Stepper(String(format: "%.1f", schemeSets[index].rpe ?? 0.0), value: Binding(
                                    get: { schemeSets[index].rpe ?? 0.0 },
                                    set: { schemeSets[index].rpe = $0 > 0 ? $0 : nil }
                                ), in: 0...10, step: 0.5)
                                if schemeSets[index].rpe ?? 0.0 > 0 {
                                    Button("Clear") {
                                        schemeSets[index].rpe = nil
                                    }
                                    .buttonStyle(.borderless)
                                }
                            }

                            HStack {
                                Text("Rest (sec):")
                                    .frame(width: 80, alignment: .trailing)
                                Stepper("\(schemeSets[index].restSec ?? 0)", value: Binding(
                                    get: { schemeSets[index].restSec ?? 0 },
                                    set: { schemeSets[index].restSec = $0 > 0 ? $0 : nil }
                                ), in: 0...600, step: 5)
                                if schemeSets[index].restSec ?? 0 > 0 {
                                    Button("Clear") {
                                        schemeSets[index].restSec = nil
                                    }
                                    .buttonStyle(.borderless)
                                }
                            }
                        }
                        .padding(.vertical, 4)
                    } label: {
                        HStack {
                            Text(schemeSets[index].label.isEmpty ? "Set \(index + 1)" : schemeSets[index].label)
                            Spacer()
                            Button(action: {
                                schemeSets.remove(at: index)
                            }) {
                                Image(systemName: "trash")
                                    .foregroundColor(.red)
                            }
                            .buttonStyle(.borderless)
                        }
                    }
                }
            }
            .frame(height: 250)

            Button("+ Add Set") {
                schemeSets.append(SchemeSet())
            }
        }
    }

    @ViewBuilder
    private var commentSegmentForm: some View {
        Section("Comment") {
            TextEditor(text: $commentText)
                .frame(height: 200)
        }
    }

    @ViewBuilder
    private var supersetSegmentForm: some View {
        Section("Superset/Circuit Info") {
            TextField("Label (optional)", text: $label)
            Stepper("Rounds: \(rounds)", value: $rounds, in: 1...10)
            Stepper("Rest between exercises: \(supersetRestSec)s", value: $supersetRestSec, in: 0...180, step: 15)
            Stepper("Rest between rounds: \(restBetweenRoundsSec)s", value: $restBetweenRoundsSec, in: 0...600, step: 30)
        }

        Section("Exercises") {
            List {
                ForEach(supersetItems.indices, id: \.self) { index in
                    HStack {
                        Text("\(index + 1).")
                            .foregroundColor(.secondary)
                            .frame(width: 20)

                        Button(action: {
                            editingSupersetItemIndex = index
                            showingSupersetItemEditor = true
                        }) {
                            if let ex = supersetItems[index]["ex"] as? String,
                               let sets = supersetItems[index]["sets"] as? Int {
                                let repsStr = formatRepsFromDict(supersetItems[index])
                                Text("\(ex) • \(sets) × \(repsStr)")
                                    .foregroundColor(.primary)
                            }
                        }
                        .buttonStyle(.plain)

                        Spacer()

                        // Move buttons
                        HStack(spacing: 4) {
                            Button(action: { moveItemUp(index) }) {
                                Image(systemName: "arrow.up")
                            }
                            .buttonStyle(.borderless)
                            .disabled(index == 0)

                            Button(action: { moveItemDown(index) }) {
                                Image(systemName: "arrow.down")
                            }
                            .buttonStyle(.borderless)
                            .disabled(index == supersetItems.count - 1)
                        }
                    }
                }
                .onDelete { offsets in
                    supersetItems.remove(atOffsets: offsets)
                }
            }
            .frame(height: 200)

            Button("+ Add Exercise") {
                editingSupersetItemIndex = nil
                showingSupersetItemEditor = true
            }
        }
        .sheet(isPresented: $showingSupersetItemEditor) {
            SupersetItemEditorView(
                plan: plan,
                item: editingSupersetItemIndex != nil ? supersetItems[editingSupersetItemIndex!] : nil,
                onSave: { itemDict in
                    if let index = editingSupersetItemIndex {
                        supersetItems[index] = itemDict
                    } else {
                        supersetItems.append(itemDict)
                    }
                    showingSupersetItemEditor = false
                }
            )
        }
    }

    private func moveItemUp(_ index: Int) {
        guard index > 0 else { return }
        supersetItems.swapAt(index, index - 1)
    }

    private func moveItemDown(_ index: Int) {
        guard index < supersetItems.count - 1 else { return }
        supersetItems.swapAt(index, index + 1)
    }

    private func formatRepsFromDict(_ dict: [String: Any]) -> String {
        if let repsObj = dict["reps"] as? [String: Any],
           let min = repsObj["min"] as? Int,
           let max = repsObj["max"] as? Int {
            return "\(min)-\(max)"
        }
        if let reps = dict["reps"] as? Int {
            return "\(reps)"
        }
        return "?"
    }

    private func loadSegment() {
        guard let json = segmentJSON,
              let data = json.data(using: .utf8),
              let dict = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let type = dict["type"] as? String else {
            return
        }

        selectedType = type

        switch type {
        case "straight", "rpe", "percentage":
            exerciseCode = dict["ex"] as? String ?? ""
            sets = dict["sets"] as? Int ?? 3
            straightRestSec = dict["rest_sec"] as? Int ?? 120
            straightRpe = dict["rpe"] as? Double ?? 8.5

            // Load exercise name from dictionary
            if !exerciseCode.isEmpty {
                exerciseName = plan.dictionary[exerciseCode] ?? exerciseCode
            }

            // Load alt_group (optional)
            altGroup = dict["alt_group"] as? String

            // Check if reps is an object with min/max
            if let repsObj = dict["reps"] as? [String: Any],
               let min = repsObj["min"] as? Int,
               let max = repsObj["max"] as? Int {
                useRepsRange = true
                repsMin = min
                repsMax = max
            } else if let repsValue = dict["reps"] as? Int {
                useRepsRange = false
                reps = repsValue
            }

        case "superset", "circuit":
            label = dict["label"] as? String ?? ""
            rounds = dict["rounds"] as? Int ?? 2
            supersetRestSec = dict["rest_sec"] as? Int ?? 0
            restBetweenRoundsSec = dict["rest_between_rounds_sec"] as? Int ?? 60

            // Load items
            if let items = dict["items"] as? [[String: Any]] {
                supersetItems = items.map { item in
                    var entry = item
                    if entry["rest_sec"] == nil {
                        entry["rest_sec"] = supersetRestSec
                    }
                    if entry["rpe"] == nil {
                        entry["rpe"] = straightRpe
                    }
                    return entry
                }
            }

        case "scheme":
            exerciseCode = dict["ex"] as? String ?? ""

            // Load exercise name from dictionary
            if !exerciseCode.isEmpty {
                exerciseName = plan.dictionary[exerciseCode] ?? exerciseCode
            }

            // Load label (optional)
            label = dict["label"] as? String ?? ""

            // Load alt_group (optional)
            schemeAltGroup = dict["alt_group"] as? String

            // Load sets array
            if let setsArray = dict["sets"] as? [[String: Any]] {
                schemeSets = setsArray.map { setDict in
                    var schemeSet = SchemeSet()
                    schemeSet.label = setDict["label"] as? String ?? ""
                    schemeSet.sets = setDict["sets"] as? Int ?? 1

                    // Parse reps (can be a range object or fixed value)
                    if let repsObj = setDict["reps"] as? [String: Any],
                       let min = repsObj["min"] as? Int,
                       let max = repsObj["max"] as? Int {
                        schemeSet.repsMin = min
                        schemeSet.repsMax = max
                    } else if let repsValue = setDict["reps"] as? Int {
                        schemeSet.repsMin = repsValue
                        schemeSet.repsMax = repsValue
                    }

                    // Parse time_sec (can be fixed or range, we'll just take fixed for now)
                    if let timeValue = setDict["time_sec"] as? Int {
                        schemeSet.timeSec = timeValue > 0 ? timeValue : nil
                    }

                    // Parse rpe (can be fixed or range, we'll just take fixed for now)
                    if let rpeValue = setDict["rpe"] as? Double {
                        schemeSet.rpe = rpeValue > 0 ? rpeValue : nil
                    }

                    // Parse rest_sec (can be fixed or range, we'll just take fixed for now)
                    if let restValue = setDict["rest_sec"] as? Int {
                        schemeSet.restSec = restValue > 0 ? restValue : nil
                    }

                    return schemeSet
                }
            }

        case "comment":
            commentText = dict["text"] as? String ?? ""

        default:
            break
        }
    }

    private func saveSegment(addAnother: Bool) {
        var segmentDict: [String: Any] = ["type": selectedType]

        switch selectedType {
        case "straight":
            segmentDict["ex"] = exerciseCode
            segmentDict["sets"] = sets
            segmentDict["rest_sec"] = max(straightRestSec, 30)
            segmentDict["rpe"] = straightRpe

            // Add alt_group if selected
            if let altGroup = altGroup {
                segmentDict["alt_group"] = altGroup
            }

            if useRepsRange {
                segmentDict["reps"] = [
                    "min": repsMin,
                    "max": repsMax
                ]
            } else {
                segmentDict["reps"] = reps
            }

        case "superset", "circuit":
            if !label.isEmpty {
                segmentDict["label"] = label
            }
            segmentDict["rounds"] = rounds
            segmentDict["rest_sec"] = max(supersetRestSec, 0)
            segmentDict["rest_between_rounds_sec"] = max(restBetweenRoundsSec, 30)
            segmentDict["items"] = supersetItems.map { item in
                var entry = item
                if entry["sets"] == nil { entry["sets"] = 1 }
                if entry["reps"] == nil { entry["reps"] = useRepsRange ? ["min": repsMin, "max": repsMax] : reps }
                if entry["rest_sec"] == nil { entry["rest_sec"] = max(supersetRestSec, 0) }
                if entry["rpe"] == nil { entry["rpe"] = straightRpe }
                return entry
            }

        case "scheme":
            segmentDict["ex"] = exerciseCode

            // Add label if not empty
            if !label.isEmpty {
                segmentDict["label"] = label
            }

            // Add alt_group if selected
            if let altGroup = schemeAltGroup {
                segmentDict["alt_group"] = altGroup
            }

            // Convert schemeSets to array of dictionaries
            var setsArray: [[String: Any]] = []
            for schemeSet in schemeSets {
                var setDict: [String: Any] = [:]

                // Add label if not empty
                if !schemeSet.label.isEmpty {
                    setDict["label"] = schemeSet.label
                }

                // Add sets count
                setDict["sets"] = schemeSet.sets

                // Add reps as a range if min != max, otherwise as a fixed value
                if schemeSet.repsMin == schemeSet.repsMax {
                    setDict["reps"] = schemeSet.repsMin
                } else {
                    setDict["reps"] = [
                        "min": schemeSet.repsMin,
                        "max": schemeSet.repsMax
                    ]
                }

                // Add time_sec if set
                if let timeSec = schemeSet.timeSec {
                    setDict["time_sec"] = timeSec
                }

                // Add rpe if set
                if let rpeValue = schemeSet.rpe {
                    setDict["rpe"] = rpeValue
                } else {
                    setDict["rpe"] = straightRpe
                }

                // Add rest_sec if set
                if let restSec = schemeSet.restSec {
                    setDict["rest_sec"] = restSec
                } else {
                    setDict["rest_sec"] = max(straightRestSec, 30)
                }

                setsArray.append(setDict)
            }

            segmentDict["sets"] = setsArray

        case "comment":
            segmentDict["text"] = commentText

        default:
            // For now, create a placeholder comment
            segmentDict = [
                "type": "comment",
                "text": "Placeholder for \(selectedType) segment"
            ]
        }

        // Convert to JSON
        if let data = try? JSONSerialization.data(withJSONObject: segmentDict, options: .prettyPrinted),
           let json = String(data: data, encoding: .utf8) {
            onSave(json)
            if addAnother && segmentJSON == nil {
                resetForm()
            } else {
                dismiss()
            }
        } else {
            print("Failed to create segment JSON")
        }
    }

    private func resetForm() {
        exerciseCode = ""
        exerciseName = ""
        sets = 3
        useRepsRange = false
        reps = 8
        repsMin = 10
        repsMax = 12
        straightRestSec = 120
        straightRpe = 8.5
        altGroup = nil
        commentText = ""
        label = ""
        rounds = 2
        supersetRestSec = 60
        restBetweenRoundsSec = 60
        supersetItems = []
        schemeSets = []
        schemeAltGroup = nil
        selectedType = "straight"
    }
}