use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTier {
    Light,
    Balanced,
    Performance,
}

impl PerformanceTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerformanceTier::Light => "light",
            PerformanceTier::Balanced => "balanced",
            PerformanceTier::Performance => "performance",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "light" => PerformanceTier::Light,
            "performance" => PerformanceTier::Performance,
            _ => PerformanceTier::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InferenceProfile {
    pub thread_count: i32,
    pub context_size: u32,
    pub max_tokens: u32,
    pub max_active_sessions: usize,
}

pub struct HardwareInfo {
    pub physical_core_count: usize,
    pub logical_core_count: usize,
    pub total_memory_mb: u64,
}

pub struct HardwareDetector;

impl HardwareDetector {
    pub fn detect() -> HardwareInfo {
        let mut system = System::new();
        system.refresh_memory();

        HardwareInfo {
            physical_core_count: num_cpus::get_physical(),
            logical_core_count: num_cpus::get(),
            total_memory_mb: system.total_memory() / (1024 * 1024),
        }
    }

    pub fn recommend_tier(info: &HardwareInfo) -> PerformanceTier {
        if info.physical_core_count <= 4 || info.total_memory_mb < 8192 {
            return PerformanceTier::Light;
        }

        if info.physical_core_count >= 8 && info.total_memory_mb >= 16384 {
            return PerformanceTier::Performance;
        }

        PerformanceTier::Balanced
    }

    pub fn inference_profile_for(
        tier: PerformanceTier,
        physical_core_count: usize,
    ) -> InferenceProfile {
        let physical_core_count = physical_core_count.max(1) as i32;

        match tier {
            PerformanceTier::Light => InferenceProfile {
                thread_count: (physical_core_count / 2).max(1),
                context_size: 4096,
                max_tokens: 256,
                max_active_sessions: 2,
            },
            PerformanceTier::Balanced => InferenceProfile {
                thread_count: (physical_core_count - 1).max(1),
                context_size: 8192,
                max_tokens: 512,
                max_active_sessions: 5,
            },
            PerformanceTier::Performance => InferenceProfile {
                thread_count: physical_core_count,
                context_size: 16384,
                max_tokens: 768,
                max_active_sessions: 8,
            },
        }
    }
}
