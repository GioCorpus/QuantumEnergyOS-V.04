//! Kernel health snapshot (§77) — feeds future QEOS dashboard, never panics.

#[derive(Debug, Default, Clone, Copy)]
pub struct SubsystemHealth {
    pub ok: bool,
    pub detail: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KernelHealth {
    pub memory: SubsystemHealth,
    pub scheduler: SubsystemHealth,
    pub interrupts: SubsystemHealth,
    pub devices: SubsystemHealth,
    pub ipc: SubsystemHealth,
}

impl KernelHealth {
    pub fn healthy() -> Self {
        Self {
            memory: SubsystemHealth { ok: true, detail: 0 },
            scheduler: SubsystemHealth { ok: true, detail: 0 },
            interrupts: SubsystemHealth { ok: true, detail: 0 },
            devices: SubsystemHealth { ok: true, detail: 0 },
            ipc: SubsystemHealth { ok: true, detail: 0 },
        }
    }
    pub fn is_healthy(&self) -> bool {
        self.memory.ok
            && self.scheduler.ok
            && self.interrupts.ok
            && self.devices.ok
            && self.ipc.ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn healthy_by_default() {
        assert!(KernelHealth::healthy().is_healthy());
    }
}