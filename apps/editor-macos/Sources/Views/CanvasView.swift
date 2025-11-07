import SwiftUI

struct CanvasView: View {
    @ObservedObject var plan: PlanDocument
    @EnvironmentObject var appState: AppState

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                if plan.days.isEmpty {
                    EmptyStateView()
                } else {
                    ForEach(plan.days) { day in
                        DayView(
                            day: day,
                            isSelected: appState.selectedDayIndex == day.id
                        )
                    }
                }
            }
            .padding()
        }
        .background(Color(NSColor.textBackgroundColor))
    }
}

struct EmptyStateView: View {
    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "calendar.badge.plus")
                .font(.system(size: 48))
                .foregroundColor(.secondary)

            Text("No Days Yet")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Add a day to get started")
                .foregroundColor(.secondary)

            Button("Add First Day") {
                // TODO: Add day action
            }
            .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity)
        .padding(40)
    }
}

struct DayView: View {
    let day: DayDisplay
    let isSelected: Bool
    @EnvironmentObject var appState: AppState

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Day header
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(day.label)
                        .font(.title2)
                        .fontWeight(.bold)

                    if let goal = day.goal {
                        Text(goal)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }

                Spacer()

                if let timeCap = day.timeCapMin {
                    Label("\(timeCap) min", systemImage: "clock")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Button(action: { appState.addSegment(to: day.id) }) {
                    Image(systemName: "plus.circle")
                }
                .buttonStyle(.borderless)
            }
            .padding(.vertical, 8)
            .padding(.horizontal, 12)
            .background(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
            .cornerRadius(8)
            .onTapGesture {
                appState.selectDay(day.id)
            }

            // Segments
            let segments = day.segments()
            if segments.isEmpty {
                Text("No segments yet")
                    .foregroundColor(.secondary)
                    .italic()
                    .padding(.leading, 12)
            } else {
                ForEach(segments) { segment in
                    SegmentRowView(
                        segment: segment,
                        isSelected: appState.selectedSegmentIndices.contains(segment.index)
                    )
                }
            }
        }
        .padding(.vertical, 8)
    }
}

struct SegmentRowView: View {
    let segment: SegmentDisplay
    let isSelected: Bool
    @EnvironmentObject var appState: AppState

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            // Icon
            Image(systemName: segment.icon)
                .font(.title3)
                .foregroundColor(colorForSegment(segment.color))
                .frame(width: 30)

            // Content
            VStack(alignment: .leading, spacing: 4) {
                Text(segment.displayText)
                    .font(.body)
            }

            Spacer()
        }
        .padding(12)
        .background(isSelected ? Color.accentColor.opacity(0.2) : Color(NSColor.controlBackgroundColor))
        .cornerRadius(6)
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
        )
        .onTapGesture {
            let modifiers = NSEvent.modifierFlags
            if modifiers.contains(.command) {
                appState.selectSegment(segment.index, multiSelect: true)
            } else if modifiers.contains(.shift) {
                appState.selectSegment(segment.index, rangeSelect: true)
            } else {
                appState.selectSegment(segment.index)
                appState.selectedDayIndex = segment.dayIndex
            }
        }
        .contextMenu {
            Button("Edit") {
                if let json = segment.toJSON() {
                    appState.editSegmentJSON(json, at: segment.index, in: segment.dayIndex)
                }
            }
            Button("Duplicate") {
                // TODO: Implement duplicate
            }
            Divider()
            Button("Delete", role: .destructive) {
                deleteSegment()
            }
        }
    }

    private func colorForSegment(_ colorName: String) -> Color {
        switch colorName {
        case "blue": return .blue
        case "orange": return .orange
        case "purple": return .purple
        case "red": return .red
        case "green": return .green
        case "teal": return .teal
        case "indigo": return .indigo
        case "pink": return .pink
        case "gray": return .gray
        default: return .primary
        }
    }

    private func deleteSegment() {
        guard let plan = segment.parent else { return }
        do {
            try plan.removeSegment(at: segment.index, fromDayAt: segment.dayIndex)
        } catch {
            print("Failed to delete segment: \(error)")
        }
    }
}
