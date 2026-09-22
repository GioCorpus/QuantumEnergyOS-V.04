//! P7.3-05 — GPU memory safety.
//!
//! Device memory is owned by a [`DeviceTable`] and referenced through
//! [`GpuMemoryHandle`]s that carry a **device generation**. When a device is
//! reset or removed, the generation is bumped, which invalidates every
//! outstanding handle. This makes stale-handle use detectable at the boundary
//! and prevents use-after-remove.
//!
//! Memory is allocated and freed explicitly; double-free is rejected because a
//! freed id is removed from the table. Allocation is bounded by `capacity`,
//! providing deterministic allocation-failure (OOM-like) behavior.

use std::collections::HashMap;

use crate::caps::GpuCaps;
use crate::error::{GpuError, Result};

/// Stable identifier of an allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMemoryId(pub u64);

/// A validated, generation-bound handle to device memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuMemoryHandle {
    pub id: GpuMemoryId,
    /// Generation of the device that issued the handle. Handles are only valid
    /// while it matches the device's current generation.
    generation: u64,
}

impl GpuMemoryHandle {
    pub fn is_compatible(self, device_generation: u64) -> bool {
        self.generation == device_generation
    }
}

/// Raw backing store for one allocation.
#[derive(Debug)]
pub struct DeviceMemory {
    pub id: GpuMemoryId,
    pub len: usize,
    /// Host shadow storage. For the CPU/mock backends this *is* the storage;
    /// a real vendor backend would keep this in device memory.
    pub data: Vec<f32>,
    pub mapped: bool,
}

/// Owns device memory allocations for a single device instance.
#[derive(Debug)]
pub struct DeviceTable {
    next_id: u64,
    /// Bound the total amount of memory a device may allocate (in bytes).
    capacity_bytes: u64,
    pub allocated_bytes: u64,
    heap: HashMap<GpuMemoryId, DeviceMemory>,
}

impl DeviceTable {
    pub fn new(capacity_bytes: u64) -> Self {
        Self {
            next_id: 0,
            capacity_bytes,
            allocated_bytes: 0,
            heap: HashMap::new(),
        }
    }

    pub fn live_allocations(&self) -> usize {
        self.heap.len()
    }

    pub fn remaining_bytes(&self) -> u64 {
        self.capacity_bytes.saturating_sub(self.allocated_bytes)
    }

    /// Allocate device memory of `len` f32 elements.
    pub fn allocate(&mut self, len: usize, device_generation: u64) -> Result<GpuMemoryHandle> {
        if len == 0 {
            return Err(GpuError::BadBuffer("zero-length allocation".into()));
        }
        let bytes = len
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| GpuError::AllocationFailed("size overflow".into()))?;
        if bytes as u64 > self.remaining_bytes() {
            return Err(GpuError::AllocationFailed("device memory exhausted".into()));
        }

        let id = GpuMemoryId(self.next_id);
        self.next_id += 1;
        self.heap.insert(
            id,
            DeviceMemory {
                id,
                len,
                data: vec![0.0; len],
                mapped: false,
            },
        );
        self.allocated_bytes += bytes as u64;

        Ok(GpuMemoryHandle {
            id,
            generation: device_generation,
        })
    }

    /// Free device memory. Rejects unknown/double-free and stale-generation
    /// handles.
    pub fn free(&mut self, handle: GpuMemoryHandle, device_generation: u64) -> Result<()> {
        if !handle.is_compatible(device_generation) {
            return Err(GpuError::StaleHandle);
        }
        let mem = self.heap.remove(&handle.id).ok_or(GpuError::StaleHandle)?;
        self.allocated_bytes = self
            .allocated_bytes
            .saturating_sub((mem.len * std::mem::size_of::<f32>()) as u64);
        Ok(())
    }

    /// Read a copy of the buffer contents (validates handle + lifetime).
    pub fn read(&self, handle: GpuMemoryHandle, device_generation: u64) -> Result<Vec<f32>> {
        let mem = self.get(handle, device_generation)?;
        Ok(mem.data.clone())
    }

    /// Write data into the buffer (validates handle + length).
    pub fn write(
        &mut self,
        handle: GpuMemoryHandle,
        device_generation: u64,
        data: &[f32],
    ) -> Result<()> {
        let mem = self.get_mut(handle, device_generation)?;
        if data.len() != mem.len {
            return Err(GpuError::BadBuffer(
                "write length does not match allocation".into(),
            ));
        }
        mem.data.copy_from_slice(data);
        Ok(())
    }

    /// Obtain a mutable borrow of the backing data (used by compute kernels).
    pub fn data_mut(
        &mut self,
        handle: GpuMemoryHandle,
        device_generation: u64,
    ) -> Result<&mut [f32]> {
        let mem = self.get_mut(handle, device_generation)?;
        Ok(&mut mem.data)
    }

    /// Mark a buffer as mapped. Only one mapping is allowed at a time.
    pub fn map(&mut self, handle: GpuMemoryHandle, device_generation: u64) -> Result<()> {
        let mem = self.get_mut(handle, device_generation)?;
        if mem.mapped {
            return Err(GpuError::MappingConflict("already mapped".into()));
        }
        mem.mapped = true;
        Ok(())
    }

    pub fn unmap(&mut self, handle: GpuMemoryHandle, device_generation: u64) -> Result<()> {
        let mem = self.get_mut(handle, device_generation)?;
        if !mem.mapped {
            return Err(GpuError::MappingConflict("not mapped".into()));
        }
        mem.mapped = false;
        Ok(())
    }

    /// Drop all allocations. Used on device reset/removal. Returns bytes freed.
    pub fn clear(&mut self) -> u64 {
        let freed = self.allocated_bytes;
        self.heap.clear();
        self.allocated_bytes = 0;
        freed
    }

    fn get(&self, handle: GpuMemoryHandle, device_generation: u64) -> Result<&DeviceMemory> {
        if !handle.is_compatible(device_generation) {
            return Err(GpuError::StaleHandle);
        }
        self.heap.get(&handle.id).ok_or(GpuError::StaleHandle)
    }

    fn get_mut(
        &mut self,
        handle: GpuMemoryHandle,
        device_generation: u64,
    ) -> Result<&mut DeviceMemory> {
        if !handle.is_compatible(device_generation) {
            return Err(GpuError::StaleHandle);
        }
        self.heap.get_mut(&handle.id).ok_or(GpuError::StaleHandle)
    }
}

/// Validate that a `len * size_of::<f32>()` fits the runtime's buffer limit.
pub fn check_size(len: usize, caps: &GpuCaps) -> Result<()> {
    if len == 0 {
        return Err(GpuError::BadBuffer("zero-length".into()));
    }
    let bytes = len
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or_else(|| GpuError::BadBuffer("size overflow".into()))?;
    if bytes > caps.max_buffer_bytes {
        return Err(GpuError::BadBuffer("exceeds max buffer size".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> DeviceTable {
        DeviceTable::new(1024)
    }

    #[test]
    fn allocate_read_write() {
        let mut t = table();
        let h = t.allocate(4, 1).unwrap();
        t.write(h, 1, &[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(t.read(h, 1).unwrap(), vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(t.live_allocations(), 1);
    }

    #[test]
    fn double_free_rejected() {
        let mut t = table();
        let h = t.allocate(4, 1).unwrap();
        t.free(h, 1).unwrap();
        assert_eq!(t.free(h, 1).unwrap_err(), GpuError::StaleHandle);
        assert_eq!(t.live_allocations(), 0);
    }

    #[test]
    fn stale_handle_after_generation_bump() {
        let mut t = table();
        let h = t.allocate(4, 1).unwrap();
        // Device removed/reset -> generation becomes 2.
        t.clear();
        assert_eq!(t.read(h, 2).unwrap_err(), GpuError::StaleHandle);
        assert_eq!(t.free(h, 2).unwrap_err(), GpuError::StaleHandle);
    }

    #[test]
    fn allocation_exhaustion() {
        let mut t = DeviceTable::new(0); // no capacity
        assert_eq!(
            t.allocate(4, 1).unwrap_err(),
            GpuError::AllocationFailed("device memory exhausted".into())
        );
    }

    #[test]
    fn zero_length_rejected() {
        let mut t = table();
        assert!(matches!(t.allocate(0, 1), Err(GpuError::BadBuffer(_))));
    }

    #[test]
    fn mapping_invalid_conflicts() {
        let mut t = table();
        let h = t.allocate(4, 1).unwrap();
        t.map(h, 1).unwrap();
        assert!(matches!(t.map(h, 1), Err(GpuError::MappingConflict(_))));
        t.unmap(h, 1).unwrap();
        assert!(matches!(t.unmap(h, 1), Err(GpuError::MappingConflict(_))));
    }

    #[test]
    fn clear_frees_all() {
        let mut t = table();
        let _ = t.allocate(4, 1).unwrap();
        let _ = t.allocate(8, 1).unwrap();
        let freed = t.clear();
        assert_eq!(freed, (4 + 8) * std::mem::size_of::<f32>() as u64);
        assert_eq!(t.live_allocations(), 0);
        assert_eq!(t.remaining_bytes(), 1024);
    }

    #[test]
    fn check_size_bounds() {
        let caps = GpuCaps::default();
        assert!(check_size(1024, &caps).is_ok());
        assert!(check_size(0, &caps).is_err());
    }
}
