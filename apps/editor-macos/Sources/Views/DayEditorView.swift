import SwiftUI

struct DayEditorView: View {
    @ObservedObject var plan: PlanDocument
    @Environment(\.dismiss) private var dismiss
    @EnvironmentObject var appState: AppState

    @State private var label: String = ""
    @State private var goal: String = ""
    @State private var timeCapMin: String = ""

    var body: some View {
        VStack(spacing: 20) {
            Text("Add New Day")
                .font(.title2)
                .fontWeight(.semibold)

            Form {
                TextField("Day Label (e.g., 'Upper A', 'Day 1')", text: $label)
                    .textFieldStyle(.roundedBorder)

                TextField("Goal (optional, e.g., 'Hypertrophy', 'Strength')", text: $goal)
                    .textFieldStyle(.roundedBorder)

                TextField("Time Cap in Minutes (optional)", text: $timeCapMin)
                    .textFieldStyle(.roundedBorder)
            }
            .padding()

            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)

                Spacer()

                Button("Add Day") {
                    saveDay()
                }
                .keyboardShortcut(.defaultAction)
                .disabled(label.isEmpty)
            }
            .padding()
        }
        .frame(width: 500, height: 300)
        .onAppear {
            // Generate default label based on current number of days
            let dayNumber = plan.days.count + 1
            label = "Day \(dayNumber)"
        }
    }

    private func saveDay() {
        var dayDict: [String: Any] = [
            "day": plan.days.count + 1,
            "label": label,
            "segments": []
        ]

        if !goal.isEmpty {
            dayDict["goal"] = goal
        }

        if let timeCap = Int(timeCapMin), timeCap > 0 {
            dayDict["time_cap_min"] = timeCap
        }

        do {
            let jsonData = try JSONSerialization.data(withJSONObject: dayDict, options: .sortedKeys)
            if let jsonString = String(data: jsonData, encoding: .utf8) {
                appState.pushUndo(plan.planJSON, label: "Add Day")
                try plan.addDay(jsonString)
                dismiss()

                // Auto-select and highlight the new day (Raskin: immediate feedback)
                let newDayIndex = plan.days.count - 1
                appState.selectedDayIndex = newDayIndex
                appState.markRecentlyAddedDay(dayIndex: newDayIndex)
                appState.shouldFocusCanvas = true
            }
        } catch {
            print("Failed to create day: \(error)")
        }
    }
}
