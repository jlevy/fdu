import Foundation
for p in CommandLine.arguments.dropFirst() {
  let u = URL(fileURLWithPath: p)
  do {
    let v = try u.resourceValues(forKeys: [.volumeAvailableCapacityKey, .volumeAvailableCapacityForImportantUsageKey, .volumeAvailableCapacityForOpportunisticUsageKey, .volumeTotalCapacityKey, .volumeUUIDStringKey, .volumeNameKey])
    let avail = v.volumeAvailableCapacity ?? 0
    let imp = Int(v.volumeAvailableCapacityForImportantUsage ?? 0)
    let opp = Int(v.volumeAvailableCapacityForOpportunisticUsage ?? 0)
    print("\(p) name=\(v.volumeName ?? "?") uuid=\(v.volumeUUIDString ?? "?") total=\(v.volumeTotalCapacity ?? 0) available=\(avail) forImportantUsage=\(imp) forOpportunisticUsage=\(opp) purgeable_estimate(important-available)=\(imp - avail)")
  } catch { print("\(p) error \(error)") }
}
