use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Lock-free single-producer single-consumer (SPSC) ring buffer.
///
/// Uses Acquire/Release memory ordering for thread-safe communication
/// between one producer and one consumer without locks.
///
/// This design is suitable for high-frequency telemetry data where
/// the producer (e.g., interrupt handler or DMA callback) writes
/// samples and the consumer (e.g., telemetry service) reads them.
///
/// # Type Parameters
/// - `T`: The element type. Must be `Copy` for safe concurrent access.
/// - `N`: The capacity of the buffer (must be a power of 2 for efficient masking).
///
/// # Memory Ordering
/// - Producer uses `Release` on write commit: ensures all prior writes to
///   the element are visible before the commit.
/// - Consumer uses `Acquire` on read: ensures it sees all writes that
///   happened before the producer's `Release`.
///
/// # Thread Safety
/// Only one thread should call `push()` at a time (single producer).
/// Only one thread should call `pop()` at a time (single consumer).
/// The producer and consumer can be different threads.
pub struct LockFreeSpscRingBuffer<T: Copy, const N: usize> {
    /// The storage buffer with interior mutability for lock-free access.
    /// SAFETY: Access is synchronized by the atomic position counters;
    /// the producer only writes to slots it owns, the consumer only reads.
    buffer: [UnsafeCell<T>; N],
    /// Write position (only producer modifies this).
    write_pos: AtomicUsize,
    /// Read position (only consumer modifies this).
    read_pos: AtomicUsize,
    /// Number of elements dropped due to overflow.
    dropped_count: AtomicUsize,
    /// Total number of elements ever written.
    total_written: AtomicUsize,
    /// Total number of elements ever read.
    total_read: AtomicUsize,
}

impl<T: Copy, const N: usize> LockFreeSpscRingBuffer<T, N> {
    const MASK: usize = N - 1;

    /// Create a new ring buffer with the specified capacity.
    ///
    /// # Panics
    /// Panics if N is not a power of 2.
    pub fn new() -> Self {
        assert!(N.is_power_of_two(), "Ring buffer size must be power of 2");
        Self {
            buffer: std::array::from_fn(|_| {
                UnsafeCell::new(unsafe {
                    // SAFETY: MaybeUninit does not require initialization.
                    std::mem::zeroed()
                })
            }),
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            dropped_count: AtomicUsize::new(0),
            total_written: AtomicUsize::new(0),
            total_read: AtomicUsize::new(0),
        }
    }

    /// Attempt to push an element into the buffer.
    ///
    /// Returns `Ok(())` on success, `Err(T)` if the buffer is full.
    /// When full, the oldest element is overwritten (backpressure handling).
    ///
    /// Only one thread should call push() at a time (single producer).
    pub fn push(&self, item: T) -> Result<(), T> {
        let write = self.write_pos.load(Ordering::Relaxed);
        let read = self.read_pos.load(Ordering::Acquire);
        let next_write = (write + 1) & Self::MASK;

        if next_write == (read & Self::MASK) {
            // Buffer is full: drop the new element and increment counter
            self.dropped_count.fetch_add(1, Ordering::Relaxed);
            return Err(item);
        }

        // SAFETY: The producer exclusively owns this slot (guarded by write_pos).
        unsafe {
            *self.buffer[write & Self::MASK].get() = item;
        }

        // Commit the write with Release ordering
        self.write_pos.store(next_write, Ordering::Release);
        self.total_written.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Attempt to pop an element from the buffer.
    ///
    /// Returns `Some(T)` if an element is available, `None` if empty.
    ///
    /// Only one thread should call pop() at a time (single consumer).
    pub fn pop(&self) -> Option<T> {
        let read = self.read_pos.load(Ordering::Relaxed);
        let write = self.write_pos.load(Ordering::Acquire);

        if read & Self::MASK == write & Self::MASK {
            return None;
        }

        // SAFETY: The consumer exclusively owns this slot (guarded by read_pos and Acquire).
        let item = unsafe { *self.buffer[read & Self::MASK].get() };

        // Advance read position with Release ordering
        self.read_pos
            .store((read + 1) & Self::MASK, Ordering::Release);
        self.total_read.fetch_add(1, Ordering::Relaxed);

        Some(item)
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        let read = self.read_pos.load(Ordering::Acquire);
        let write = self.write_pos.load(Ordering::Acquire);
        (read & Self::MASK) == (write & Self::MASK)
    }

    /// Get the current number of elements in the buffer.
    pub fn len(&self) -> usize {
        let read = self.read_pos.load(Ordering::Acquire);
        let write = self.write_pos.load(Ordering::Acquire);
        (write.wrapping_sub(read)) & Self::MASK
    }

    /// Get the capacity of the buffer.
    pub fn capacity(&self) -> usize {
        N
    }

    /// Get the number of elements dropped due to overflow.
    pub fn dropped_count(&self) -> usize {
        self.dropped_count.load(Ordering::Relaxed)
    }

    /// Get total number of elements written.
    pub fn total_written(&self) -> usize {
        self.total_written.load(Ordering::Relaxed)
    }

    /// Get total number of elements read.
    pub fn total_read(&self) -> usize {
        self.total_read.load(Ordering::Relaxed)
    }

    /// Clear the buffer.
    pub fn clear(&self) {
        self.write_pos.store(0, Ordering::Release);
        self.read_pos.store(0, Ordering::Release);
        self.dropped_count.store(0, Ordering::Relaxed);
        self.total_written.store(0, Ordering::Relaxed);
        self.total_read.store(0, Ordering::Relaxed);
    }
}

impl<T: Copy, const N: usize> Default for LockFreeSpscRingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: LockFreeSpscRingBuffer is Send if T is Send because each element
// is accessed by only one producer and one consumer, synchronized by atomics.
unsafe impl<T: Copy + Send, const N: usize> Send for LockFreeSpscRingBuffer<T, N> {}

// SAFETY: LockFreeSpscRingBuffer is Sync because concurrent access is
// safe through the atomic operations with proper memory ordering.
unsafe impl<T: Copy + Send, const N: usize> Sync for LockFreeSpscRingBuffer<T, N> {}

/// Telemetry sample with timestamp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetrySample {
    /// Timestamp in nanoseconds since some epoch.
    pub timestamp_ns: u64,
    /// Sample value.
    pub value: f64,
    /// Sample type identifier.
    pub sample_type: SampleType,
}

/// Types of telemetry samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SampleType {
    CpuTemperature,
    GpuTemperature,
    CpuPower,
    GpuPower,
    MemoryPower,
    FanSpeed,
    Voltage,
    Current,
    Frequency,
    Custom(u16),
}

impl TelemetrySample {
    pub fn new(sample_type: SampleType, value: f64) -> Self {
        Self {
            timestamp_ns: 0,
            value,
            sample_type,
        }
    }

    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }
}

/// Energy telemetry sample specifically for power measurements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergySample {
    pub timestamp_ns: u64,
    pub voltage: f64,
    pub current: f64,
    pub power: f64,
}

impl EnergySample {
    pub fn new(voltage: f64, current: f64) -> Self {
        Self {
            timestamp_ns: 0,
            voltage,
            current,
            power: voltage * current,
        }
    }

    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }
}

/// Statistics for the ring buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RingBufferStats {
    pub capacity: usize,
    pub current_len: usize,
    pub dropped_count: usize,
    pub total_written: usize,
    pub total_read: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_creation() {
        let buf = LockFreeSpscRingBuffer::<TelemetrySample, 16>::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 16);
    }

    #[test]
    fn test_push_and_pop() {
        let buf = LockFreeSpscRingBuffer::<i32, 16>::new();

        assert!(buf.push(42).is_ok());
        assert!(!buf.is_empty());
        assert_eq!(buf.len(), 1);

        let item = buf.pop();
        assert_eq!(item, Some(42));
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffer_full() {
        let buf = LockFreeSpscRingBuffer::<i32, 4>::new();

        // Fill the buffer
        for i in 0..3 {
            assert!(buf.push(i).is_ok());
        }

        // Next push should fail (buffer full)
        assert!(buf.push(99).is_err());
        assert!(buf.dropped_count() > 0);
    }

    #[test]
    fn test_wrap_around() {
        let buf = LockFreeSpscRingBuffer::<i32, 4>::new();

        // Push and pop to advance positions
        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.pop();
        buf.pop();

        // Push more to cause wrap-around
        buf.push(3).unwrap();
        buf.push(4).unwrap();

        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
    }

    #[test]
    fn test_statistics() {
        let buf = LockFreeSpscRingBuffer::<i32, 8>::new();

        for i in 0..5 {
            buf.push(i).unwrap();
        }

        assert_eq!(buf.total_written(), 5);
        assert_eq!(buf.len(), 5);

        buf.pop();
        buf.pop();

        assert_eq!(buf.total_read(), 2);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_clear() {
        let buf = LockFreeSpscRingBuffer::<i32, 16>::new();

        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.clear();

        assert!(buf.is_empty());
        assert_eq!(buf.dropped_count(), 0);
        assert_eq!(buf.total_written(), 0);
    }

    #[test]
    fn test_telemetry_sample() {
        let sample =
            TelemetrySample::new(SampleType::CpuTemperature, 65.5).with_timestamp(1234567890);

        assert_eq!(sample.value, 65.5);
        assert_eq!(sample.timestamp_ns, 1234567890);
        assert_eq!(sample.sample_type, SampleType::CpuTemperature);
    }

    #[test]
    fn test_energy_sample() {
        let sample = EnergySample::new(12.0, 2.5);
        assert!((sample.power - 30.0).abs() < 1e-10);
        assert_eq!(sample.voltage, 12.0);
        assert_eq!(sample.current, 2.5);
    }

    #[test]
    fn test_sample_type_equality() {
        assert_eq!(SampleType::CpuPower, SampleType::CpuPower);
        assert_ne!(SampleType::CpuPower, SampleType::GpuPower);
        assert_eq!(SampleType::Custom(42), SampleType::Custom(42));
    }
}
