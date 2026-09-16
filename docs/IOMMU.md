# IOMMU (Phase 4.4)

`kernel::driver::iommu::{Iommu, DomainId, IommuPerm, MockIommu}`:
domain creation, device attach, map/unmap with R/W permissions, isolation
model. `MockIommu` is a host-testable software model, NOT hardware support.
DMA grants additionally require device-manager capability + bus-master facts.
Real IOMMU (Intel VT-d / AMD-Vi / ARM SMMU) is FUTURE.
