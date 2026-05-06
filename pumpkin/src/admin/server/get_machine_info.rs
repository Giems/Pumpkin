use crate::admin::AppState;
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

#[derive(Serialize)]
pub struct MachineInfo {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub swap_used_mb: u64,
    pub swap_total_mb: u64,
}

pub async fn get_machine_info(_: State<Arc<AppState>>) -> impl IntoResponse {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything()),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();

    let cpus = sys.cpus();
    let cpu_brand = cpus
        .first()
        .map(|c| c.brand().to_owned())
        .unwrap_or_default();
    let cpu_usage = if cpus.is_empty() {
        0.0
    } else {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    };

    const MB: u64 = 1024 * 1024;
    Json(MachineInfo {
        hostname: System::host_name().unwrap_or_default(),
        os: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        cpu_count: cpus.len(),
        cpu_brand,
        cpu_usage_percent: (cpu_usage * 100.0).round() / 100.0,
        memory_used_mb: sys.used_memory() / MB,
        memory_total_mb: sys.total_memory() / MB,
        swap_used_mb: sys.used_swap() / MB,
        swap_total_mb: sys.total_swap() / MB,
    })
}
