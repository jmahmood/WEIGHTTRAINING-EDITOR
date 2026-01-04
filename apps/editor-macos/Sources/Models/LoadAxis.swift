import Foundation

struct LoadAxis: Equatable, Hashable {
    var kind: LoadAxisKind
    var values: [String]
}

enum LoadAxisKind: String {
    case categorical
    case ordinal
}

struct LoadAxisTarget: Equatable, Hashable {
    let axis: String
    let target: String
}

struct RoleRepsRange: Equatable, Hashable {
    var min: Int
    var max: Int
}

struct ExerciseMeta: Equatable, Hashable {
    var loadAxes: [String: LoadAxis]
    var roleReps: [String: RoleRepsRange]
}

extension LoadAxis {
    func toDictionary() -> [String: Any] {
        [
            "kind": kind.rawValue,
            "values": values
        ]
    }
}
