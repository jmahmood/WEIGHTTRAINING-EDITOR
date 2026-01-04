import SwiftUI

struct GroupRolePicker: View {
    let groupId: String?
    let availableRoles: [String]
    @Binding var selectedRole: String?

    var body: some View {
        if groupId == nil {
            Text("Set an alternative group to use focus.")
                .font(.caption)
                .foregroundColor(.secondary)
        } else if availableRoles.isEmpty {
            HStack {
                Picker("", selection: $selectedRole) {
                    Text("None").tag(nil as String?)
                }
                Text("No focus options defined")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        } else {
            Picker("", selection: $selectedRole) {
                Text("None").tag(nil as String?)
                ForEach(availableRoles, id: \.self) { role in
                    Text(role.capitalized).tag(role as String?)
                }
            }
        }
    }
}
