//! Capability-based device access (§20) and the DMA/IOMMU policy that gates it.
//!
//! Every operation that touches a device goes through the manager, and the
//! manager requires the matching [`DeviceCapability`] to be *declared* by the
//! hardware (derived from configuration space) and *granted* by policy. A device
//! therefore never becomes reachable just because it was enumerated.
//!
//! The defaults are deny-by-default:
//!
//! - [`IommuPolicy::DenyUnmanagedDma`]: DMA capability cannot be granted while no
//!   IOMMU enforcement exists, because untranslated DMA is not isolatable.
//! - [`HotplugPolicy::DenyRemoval`]: removal is refused unless policy allows it,
//!   and (with [`HotplugPolicy::AllowReported`]) the device reports PCIe hotplug.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A device-touching capability that can be required before an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DeviceCapability {
    /// Walk the bus and list devices.
    Enumerate,
    /// Read configuration space and health information.
    Inspect,
    /// Drive the device lifecycle (initialize / start / stop).
    Operate,
    /// Map a memory BAR and read/write through an [`crate::pci::bar::MmioRegion`].
    MmioAccess,
    /// Hand DMA-capable buffers to the device.
    DmaAccess,
    /// Program MSI/MSI-X vectors or bind a legacy interrupt line.
    InterruptControl,
    /// Reset the device.
    Reset,
    /// Change power state (D0..D3).
    PowerManagement,
    /// Allow hot removal of the device.
    Hotplug,
}

/// Every capability, in a stable order. Used by iteration and mask helpers.
pub const ALL_CAPABILITIES: [DeviceCapability; 9] = [
    DeviceCapability::Enumerate,
    DeviceCapability::Inspect,
    DeviceCapability::Operate,
    DeviceCapability::MmioAccess,
    DeviceCapability::DmaAccess,
    DeviceCapability::InterruptControl,
    DeviceCapability::Reset,
    DeviceCapability::PowerManagement,
    DeviceCapability::Hotplug,
];

impl DeviceCapability {
    /// Single-bit mask for this capability inside [`DeviceCapabilitySet`].
    pub const fn mask(self) -> u16 {
        1u16 << (self as u16)
    }

    /// Stable snake_case label used in snapshots, telemetry and CLI output.
    pub const fn label(self) -> &'static str {
        match self {
            DeviceCapability::Enumerate => "enumerate",
            DeviceCapability::Inspect => "inspect",
            DeviceCapability::Operate => "operate",
            DeviceCapability::MmioAccess => "mmio_access",
            DeviceCapability::DmaAccess => "dma_access",
            DeviceCapability::InterruptControl => "interrupt_control",
            DeviceCapability::Reset => "reset",
            DeviceCapability::PowerManagement => "power_management",
            DeviceCapability::Hotplug => "hotplug",
        }
    }

    /// Parses a label produced by [`DeviceCapability::label`].
    pub fn from_label(label: &str) -> Option<Self> {
        ALL_CAPABILITIES
            .into_iter()
            .find(|capability| capability.label() == label)
    }
}

impl fmt::Display for DeviceCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A bounded set of [`DeviceCapability`] values, backed by a `u16` bitmask.
///
/// The set is small, `Copy` and lock-free: it is read on hot paths (every gated
/// operation) and serialized into snapshots.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCapabilitySet {
    bits: u16,
}

impl DeviceCapabilitySet {
    /// The empty set: nothing is permitted.
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Every capability. Never a default: it must be requested explicitly.
    pub const fn all() -> Self {
        Self {
            bits: (1u16 << ALL_CAPABILITIES.len() as u32) - 1,
        }
    }

    /// A single capability.
    pub const fn of(capability: DeviceCapability) -> Self {
        Self {
            bits: capability.mask(),
        }
    }

    /// Baseline granted to auto-bound devices: enumerate + inspect + operate.
    ///
    /// Deliberately excludes MMIO, DMA, interrupts, reset and hotplug.
    pub const fn baseline() -> Self {
        Self {
            bits: DeviceCapability::Enumerate.mask()
                | DeviceCapability::Inspect.mask()
                | DeviceCapability::Operate.mask(),
        }
    }

    /// Adds a capability to the set.
    pub fn grant(&mut self, capability: DeviceCapability) {
        self.bits |= capability.mask();
    }

    /// Removes a capability from the set.
    pub fn revoke(&mut self, capability: DeviceCapability) {
        self.bits &= !capability.mask();
    }

    /// True when the capability is present.
    pub const fn contains(&self, capability: DeviceCapability) -> bool {
        self.bits & capability.mask() != 0
    }

    /// Raw bitmask, for snapshots and tests.
    pub const fn bits(&self) -> u16 {
        self.bits
    }

    /// True when no capability is present.
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Union of two sets.
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// Set difference (`self` without `other`).
    pub const fn difference(self, other: Self) -> Self {
        Self {
            bits: self.bits & !other.bits,
        }
    }

    /// True when every capability of `self` is also in `other`.
    pub const fn is_subset_of(self, other: Self) -> bool {
        self.bits & !other.bits == 0
    }

    /// Iterates the contained capabilities in [`ALL_CAPABILITIES`] order.
    pub fn iter(&self) -> impl Iterator<Item = DeviceCapability> + '_ {
        ALL_CAPABILITIES
            .into_iter()
            .filter(move |capability| self.contains(*capability))
    }

    /// Stable labels of the contained capabilities.
    pub fn labels(&self) -> Vec<&'static str> {
        self.iter().map(|capability| capability.label()).collect()
    }
}

impl fmt::Display for DeviceCapabilitySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("none");
        }
        f.write_str(&self.labels().join(","))
    }
}

/// Policy applied when a device asks for DMA access.
///
/// QEOS has no IOMMU enforcement in the host model, so the default is to refuse
/// unmanaged DMA instead of silently handing out bus addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IommuPolicy {
    /// Default: unmanaged DMA cannot be authorised (no translation domain).
    #[default]
    DenyUnmanagedDma,
    /// Explicit opt-in for research/host models. Documented weaker guarantee:
    /// buffers handed to the device are mapped 1:1 and are not isolated from it.
    AllowUnmanagedDma,
    /// A kernel backend enforces IOMMU translation domains. No bundled backend
    /// implements this yet; it is the target for Phase 4.2.
    Enforced,
}

impl IommuPolicy {
    /// True when the policy permits granting [`DeviceCapability::DmaAccess`].
    pub const fn permits_dma_grant(self) -> bool {
        match self {
            IommuPolicy::DenyUnmanagedDma => false,
            IommuPolicy::AllowUnmanagedDma | IommuPolicy::Enforced => true,
        }
    }

    /// Stable label used in diagnostics.
    pub const fn label(self) -> &'static str {
        match self {
            IommuPolicy::DenyUnmanagedDma => "deny-unmanaged-dma",
            IommuPolicy::AllowUnmanagedDma => "allow-unmanaged-dma",
            IommuPolicy::Enforced => "enforced",
        }
    }
}

impl fmt::Display for IommuPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Policy applied to hot-removal requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HotplugPolicy {
    /// Default: removal is refused. Not every device supports hotplug.
    #[default]
    DenyRemoval,
    /// Removal is allowed only for devices that report PCIe hotplug support.
    AllowReported,
    /// Removal is allowed for any device the manager owns. Development/test only.
    AllowAny,
}

impl HotplugPolicy {
    /// Stable label used in diagnostics.
    pub const fn label(self) -> &'static str {
        match self {
            HotplugPolicy::DenyRemoval => "deny-removal",
            HotplugPolicy::AllowReported => "allow-reported",
            HotplugPolicy::AllowAny => "allow-any",
        }
    }
}

impl fmt::Display for HotplugPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_are_distinct_and_within_range() {
        let mut seen = 0u16;
        for capability in ALL_CAPABILITIES {
            let mask = capability.mask();
            assert_ne!(mask, 0);
            assert_eq!(seen & mask, 0, "duplicate mask for {capability}");
            seen |= mask;
        }
        assert_eq!(seen, DeviceCapabilitySet::all().bits());
    }

    #[test]
    fn grant_and_revoke_are_inverse() {
        let mut set = DeviceCapabilitySet::empty();
        assert!(set.is_empty());
        set.grant(DeviceCapability::Operate);
        assert!(set.contains(DeviceCapability::Operate));
        assert!(!set.contains(DeviceCapability::DmaAccess));
        set.revoke(DeviceCapability::Operate);
        assert!(set.is_empty());
    }

    #[test]
    fn baseline_excludes_dangerous_capabilities() {
        let baseline = DeviceCapabilitySet::baseline();
        assert!(baseline.contains(DeviceCapability::Inspect));
        assert!(baseline.contains(DeviceCapability::Operate));
        for dangerous in [
            DeviceCapability::MmioAccess,
            DeviceCapability::DmaAccess,
            DeviceCapability::InterruptControl,
            DeviceCapability::Reset,
            DeviceCapability::Hotplug,
        ] {
            assert!(
                !baseline.contains(dangerous),
                "{dangerous} must not be part of the baseline"
            );
        }
    }

    #[test]
    fn set_algebra_helpers_agree() {
        let a = DeviceCapabilitySet::of(DeviceCapability::Operate);
        let b = DeviceCapabilitySet::of(DeviceCapability::Reset);
        let union = a.union(b);
        assert!(union.contains(DeviceCapability::Operate));
        assert!(union.contains(DeviceCapability::Reset));
        assert!(a.is_subset_of(union));
        assert_eq!(union.difference(b), a);
    }

    #[test]
    fn iteration_and_labels_are_stable() {
        let mut set = DeviceCapabilitySet::empty();
        set.grant(DeviceCapability::Hotplug);
        set.grant(DeviceCapability::Enumerate);
        assert_eq!(set.labels(), vec!["enumerate", "hotplug"]);
        assert_eq!(set.to_string(), "enumerate,hotplug");
        assert_eq!(DeviceCapabilitySet::empty().to_string(), "none");
        assert_eq!(
            DeviceCapability::from_label("dma_access"),
            Some(DeviceCapability::DmaAccess)
        );
        assert_eq!(DeviceCapability::from_label("nope"), None);
    }

    #[test]
    fn iommu_policy_defaults_to_denying_unmanaged_dma() {
        let policy = IommuPolicy::default();
        assert_eq!(policy, IommuPolicy::DenyUnmanagedDma);
        assert!(!policy.permits_dma_grant());
        assert!(IommuPolicy::AllowUnmanagedDma.permits_dma_grant());
        assert_eq!(IommuPolicy::Enforced.to_string(), "enforced");
    }

    #[test]
    fn hotplug_policy_defaults_to_denying_removal() {
        assert_eq!(HotplugPolicy::default(), HotplugPolicy::DenyRemoval);
        assert_eq!(HotplugPolicy::AllowReported.to_string(), "allow-reported");
        assert_eq!(HotplugPolicy::AllowAny.to_string(), "allow-any");
    }
}