import SwiftUI

struct LoadAxisTargetPicker: View {
    let availableAxes: [String: LoadAxis]
    @Binding var target: LoadAxisTarget?

    @State private var selectedAxis: String?
    @State private var selectedValue: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Picker("Resistance Type", selection: $selectedAxis) {
                Text("None").tag(nil as String?)
                ForEach(availableAxes.keys.sorted(), id: \.self) { axisName in
                    Text(axisName).tag(axisName as String?)
                }
            }

            if let axis = selectedAxis,
               let axisConfig = availableAxes[axis] {
                Picker("Resistance Value", selection: $selectedValue) {
                    ForEach(axisConfig.values, id: \.self) { value in
                        Text(value).tag(value as String?)
                    }
                }
            }
        }
        .onAppear(perform: syncFromTarget)
        .onChange(of: target) { _ in syncFromTarget() }
        .onChange(of: selectedAxis) { _ in
            if let axis = selectedAxis,
               let axisConfig = availableAxes[axis],
               !axisConfig.values.contains(selectedValue ?? "") {
                selectedValue = axisConfig.values.first
            }
            updateTarget()
        }
        .onChange(of: selectedValue) { _ in updateTarget() }
    }

    private func syncFromTarget() {
        guard let target = target else {
            selectedAxis = nil
            selectedValue = nil
            return
        }
        selectedAxis = target.axis
        selectedValue = target.target
    }

    private func updateTarget() {
        if let axis = selectedAxis, let value = selectedValue {
            target = LoadAxisTarget(axis: axis, target: value)
        } else {
            target = nil
        }
    }
}
