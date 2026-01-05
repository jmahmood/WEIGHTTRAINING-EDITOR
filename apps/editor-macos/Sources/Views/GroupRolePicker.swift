import SwiftUI

struct GroupRolePicker: View {
    let groupId: String?
    let availableRoles: [String]
    @Binding var selectedRole: String?

    var body: some View {
        let selection = Binding<String>(
            get: { selectedRole ?? "" },
            set: { value in selectedRole = value.isEmpty ? nil : value }
        )

        if groupId == nil {
            Text("Set an alternative group to use focus.")
                .font(.caption)
                .foregroundColor(.secondary)
        } else if availableRoles.isEmpty {
            HStack {
                Picker("", selection: selection) {
                    Text("None").tag("")
                }
                .pickerStyle(.menu)
                Text("No focus options defined")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        } else {
            Picker("", selection: selection) {
                Text("None").tag("")
                ForEach(availableRoles, id: \.self) { role in
                    Text(role.capitalized).tag(role)
                }
            }
            .pickerStyle(.menu)
        }
    }
}
