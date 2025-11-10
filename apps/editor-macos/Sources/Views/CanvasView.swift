import SwiftUI

struct SegmentActions {
    let edit: () -> Void
    let preview: () -> Void
}

struct SegmentActionsKey: FocusedValueKey {
    typealias Value = SegmentActions
}

extension FocusedValues {
    var segmentActions: SegmentActions? {
        get { self[SegmentActionsKey.self] }
        set { self[SegmentActionsKey.self] = newValue }
    }
}

struct CanvasView: View {
    @ObservedObject var plan: PlanDocument
    @EnvironmentObject var appState: AppState
    @FocusState private var canvasFocused: Bool

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 20) {
                    header
                    if plan.days.isEmpty {
                        EmptyStateView {
                            canvasFocused = true
                            appState.addDay()
                        }
                    } else {
                        ForEach(plan.days) { day in
                            DayView(
                                plan: plan,
                                day: day,
                                isSelected: appState.selectedDayIndex == day.id,
                                focusCanvas: { canvasFocused = true }
                            )
                            .id("day_\(day.id)")
                        }
                    }
                }
                .padding()
            }
            .onChange(of: appState.recentlyAddedSegmentID) { identifier in
                guard let identifier else { return }
                withAnimation(.easeInOut(duration: 0.3)) {
                    proxy.scrollTo(identifier, anchor: .center)
                }
            }
            .onChange(of: appState.selectedSegmentIds) { selectedIds in
                // Scroll to keep selected segment visible when navigating with keyboard
                guard canvasFocused, let firstSelected = selectedIds.first else { return }
                withAnimation(.easeInOut(duration: 0.2)) {
                    proxy.scrollTo(firstSelected, anchor: .center)
                }
            }
        }
        .focusable(true)
        .focused($canvasFocused)
        .focusedValue(
            \.segmentActions,
            canvasFocused
                ? SegmentActions(
                    edit: { appState.editSelectedSegment(in: plan) },
                    preview: { appState.previewSelectedSegment(in: plan) }
                )
                : nil
        )
        .onAppear { canvasFocused = true }
        .onMoveCommand { direction in
            guard canvasFocused else { return }
            switch direction {
            case .up:
                appState.selectAdjacentSegment(delta: -1, in: plan)
            case .down:
                appState.selectAdjacentSegment(delta: 1, in: plan)
            default:
                break
            }
        }
        .background(Color(NSColor.textBackgroundColor))
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text("Schedule")
                    .font(.title2)
                    .fontWeight(.semibold)
                Text("Use ⇧⌘N to add a day")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            Button {
                canvasFocused = true
                appState.addDay()
            } label: {
                Label("Add Day", systemImage: "calendar.badge.plus")
            }
            .keyboardShortcut("N", modifiers: [.command, .shift])
        }
    }
}

struct EmptyStateView: View {
    var action: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "calendar.badge.plus")
                .font(.system(size: 48))
                .foregroundColor(.secondary)

            Text("No Days Yet")
                .font(.title2)
                .fontWeight(.semibold)

            VStack(spacing: 4) {
                Text("Add a day to get started")
                    .foregroundColor(.secondary)
                Text("Shortcut: ⇧⌘N")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Button {
                action()
            } label: {
                Label("Add First Day", systemImage: "plus.circle.fill")
            }
            .keyboardShortcut("N", modifiers: [.command, .shift])
            .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity)
        .padding(40)
    }
}

struct DayView: View {
    @ObservedObject var plan: PlanDocument
    let day: DayDisplay
    let isSelected: Bool
    let focusCanvas: () -> Void
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

                Button(action: {
                    focusCanvas()
                    appState.addSegment(to: day.id)
                }) {
                    Label("Add Segment", systemImage: "plus.circle")
                }
                .buttonStyle(.borderless)
                .labelStyle(.titleAndIcon)
            }
            .padding(.vertical, 8)
            .padding(.horizontal, 12)
            .background(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
            .cornerRadius(8)
            .onTapGesture {
                focusCanvas()
                appState.selectDay(day.id)
            }

            // Segments
            let segments = day.segments()
            if segments.isEmpty {
                Button {
                    focusCanvas()
                    appState.addSegment(to: day.id)
                } label: {
                    Label("Add the first segment", systemImage: "plus.circle")
                        .font(.body)
                }
                .buttonStyle(.borderless)
                .padding(.leading, 12)
            } else {
                ForEach(segments) { segment in
                    SegmentRowView(
                        plan: plan,
                        segment: segment,
                        isSelected: appState.selectedSegmentIds.contains(segment.id),
                        isRecent: appState.recentlyAddedSegmentID == segment.id,
                        focusCanvas: focusCanvas
                    )
                    .id(segment.id)
                }
            }
        }
        .padding(.vertical, 8)
    }
}

struct SegmentRowView: View {
    @ObservedObject var plan: PlanDocument
    let segment: SegmentDisplay
    let isSelected: Bool
    let isRecent: Bool
    let focusCanvas: () -> Void
    @EnvironmentObject var appState: AppState

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: segment.icon)
                .font(.title3)
                .foregroundColor(colorForSegment(segment.color))
                .frame(width: 30, height: 30)
                .background(Circle().fill(colorForSegment(segment.color).opacity(0.15)))

            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Text(segment.primaryTitle(with: plan))
                        .font(.headline)
                    Spacer()
                    Text(segment.humanReadableType)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                // Show detailed info based on segment type
                switch segment.kind {
                case .superset, .circuit:
                    // Show list of exercises
                    let exercises = segment.groupExercises(plan: plan)
                    if !exercises.isEmpty {
                        VStack(alignment: .leading, spacing: 4) {
                            ForEach(exercises) { exercise in
                                HStack(spacing: 8) {
                                    Text("•")
                                        .foregroundColor(.secondary)
                                    Text(exercise.name)
                                        .font(.subheadline)
                                    if !exercise.details.isEmpty {
                                        Text("—")
                                            .foregroundColor(.secondary)
                                        Text(exercise.details)
                                            .font(.caption)
                                            .foregroundColor(.secondary)
                                    }
                                }
                            }
                        }
                        .padding(.top, 2)
                    }
                    HStack(alignment: .firstTextBaseline, spacing: 24) {
                        MetricColumn(title: "Rounds", value: segment.setsDescription)
                        MetricColumn(title: "Rest", value: segment.restDescription)
                    }

                case .scheme:
                    // Show set details
                    let sets = segment.schemeSetDetails()
                    if !sets.isEmpty {
                        VStack(alignment: .leading, spacing: 4) {
                            ForEach(sets) { set in
                                HStack(spacing: 8) {
                                    Text("•")
                                        .foregroundColor(.secondary)
                                    Text(set.title)
                                        .font(.subheadline)
                                        .fontWeight(.medium)
                                    Text("—")
                                        .foregroundColor(.secondary)
                                    Text(set.summary)
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                            }
                        }
                        .padding(.top, 2)
                    }

                default:
                    // Standard metric columns for other types
                    HStack(alignment: .firstTextBaseline, spacing: 24) {
                        MetricColumn(title: "Sets × Reps", value: segment.setsDescription)
                        MetricColumn(title: "Rest", value: segment.restDescription)
                        MetricColumn(title: "Notes", value: segment.notesDescription)
                    }
                }
            }

            Spacer()
        }
        .padding(12)
        .background(backgroundColor)
        .cornerRadius(6)
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 2)
        )
        .onTapGesture {
            handleSelection()
        }
        .contextMenu {
            Button("Duplicate (⌘D)") {
                duplicateSegment()
            }
            Button("Move Up (⌘↑)") {
                moveSegmentUp()
            }
            Button("Move Down (⌘↓)") {
                moveSegmentDown()
            }
            Divider()
            Button("Delete (Del)", role: .destructive) {
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
            appState.pushUndo(plan.planJSON, label: "Delete Segment")
            try plan.removeSegment(at: segment.index, fromDayAt: segment.dayIndex)
        } catch {
            ErrorLogger.shared.error("Failed to delete segment: \(error.localizedDescription)")
        }
    }

    private func duplicateSegment() {
        focusCanvas()
        appState.duplicateSelectedSegment(in: plan)
    }

    private func moveSegmentUp() {
        focusCanvas()
        appState.moveSelectedSegment(up: true, in: plan)
    }

    private func moveSegmentDown() {
        focusCanvas()
        appState.moveSelectedSegment(up: false, in: plan)
    }

    private func handleSelection() {
        focusCanvas()
        let modifiers = NSEvent.modifierFlags
        if modifiers.contains(.command) {
            appState.selectSegment(segment, in: plan, multiSelect: true)
        } else if modifiers.contains(.shift) {
            appState.selectSegment(segment, in: plan, rangeSelect: true)
        } else {
            appState.selectSegment(segment, in: plan)
            appState.selectedDayIndex = segment.dayIndex
        }
    }

    private var backgroundColor: Color {
        if isRecent {
            return Color.accentColor.opacity(0.25)
        }
        return isSelected ? Color.accentColor.opacity(0.15) : Color(NSColor.controlBackgroundColor)
    }
}

struct MetricColumn: View {
    let title: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title.uppercased())
                .font(.caption2)
                .foregroundColor(.secondary)
            Text(value)
                .font(.subheadline)
                .foregroundColor(value == "—" ? .secondary : .primary)
        }
    }
}
