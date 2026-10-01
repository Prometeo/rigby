use crate::container::models::{DockerContainer, DockerContainerDetail};
use bollard::query_parameters::StatsOptions;
use bollard::service::ContainerStatsResponse;
use bollard::{
    Docker,
    container::LogOutput,
    query_parameters::{ListContainersOptionsBuilder, LogsOptions},
};
use color_eyre::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, UnboundedReceiver, unbounded_channel};
use tokio::task::AbortHandle;

pub async fn list_containers(client: &Docker) -> Result<Vec<DockerContainer>> {
    let options = ListContainersOptionsBuilder::default().all(true).build();
    let containers = client.list_containers(Some(options)).await?;

    let container_list = containers
        .into_iter()
        .map(DockerContainer::try_from)
        .collect::<Result<Vec<DockerContainer>, _>>()?;
    Ok(container_list)
}

pub async fn inspect_container(
    client: &Docker,
    container: &DockerContainer,
) -> Result<DockerContainerDetail> {
    let container_info = client.inspect_container(&container.id, None).await?;
    Ok(DockerContainerDetail::new(&container_info, container))
}

pub fn get_container_logs(
    client: Docker,
    container_id: String,
    tail: usize,
) -> (UnboundedReceiver<String>, AbortHandle) {
    let (tx, rx) = mpsc::unbounded_channel();

    let task = tokio::spawn(async move {
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            follow: true,
            tail: tail.to_string(),
            timestamps: false,
            ..Default::default()
        };

        let mut stream = client.logs(&container_id, Some(options));

        while let Some(chunk) = stream.next().await {
            let Ok(output) = chunk else { break };

            let text = match output {
                LogOutput::StdOut { message } | LogOutput::Console { message } => {
                    String::from_utf8_lossy(&message).to_string()
                }
                LogOutput::StdErr { message } => {
                    format!("[ERR] {}", String::from_utf8_lossy(&message))
                }
                _ => continue,
            };

            for line in text.lines() {
                if tx.send(line.to_string()).is_err() {
                    return;
                }
            }
        }
    });

    (rx, task.abort_handle())
}
pub fn get_container_cpu_stream(
    client: Docker,
    container_id: String,
) -> (UnboundedReceiver<f64>, AbortHandle) {
    let (tx, rx) = unbounded_channel();

    let task = tokio::spawn(async move {
        let options = StatsOptions {
            stream: true,
            one_shot: false,
        };

        let mut stream = client.stats(&container_id, Some(options));

        let mut last_cpu_total: u64 = 0;
        let mut last_system_total: u64 = 0;

        while let Some(item) = stream.next().await {
            let stats: ContainerStatsResponse = match item {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Bollard stats error for {container_id}: {e:?}");
                    break;
                }
            };

            let cpu = match stats.cpu_stats {
                Some(ref c) => c,
                None => continue,
            };

            let cur_cpu = cpu
                .cpu_usage
                .as_ref()
                .and_then(|u| u.total_usage)
                .unwrap_or(0);

            let cur_system = cpu.system_cpu_usage.unwrap_or(0);

            let pre_cpu = stats
                .precpu_stats
                .as_ref()
                .and_then(|p| p.cpu_usage.as_ref())
                .and_then(|u| u.total_usage)
                .unwrap_or(last_cpu_total);

            let pre_system = stats
                .precpu_stats
                .as_ref()
                .and_then(|p| p.system_cpu_usage)
                .unwrap_or(last_system_total);

            let cpu_delta = cur_cpu.saturating_sub(pre_cpu) as f64;
            let system_delta = cur_system.saturating_sub(pre_system) as f64;

            last_cpu_total = cur_cpu;
            last_system_total = cur_system;

            let online_cpus = cpu
                .online_cpus
                .map(|c| c as f64)
                .or_else(|| {
                    cpu.cpu_usage
                        .as_ref()
                        .and_then(|u| u.percpu_usage.as_ref())
                        .map(|p| p.len() as f64)
                })
                .unwrap_or(1.0);

            let cpu_percent = if system_delta > 0.0 {
                (cpu_delta / system_delta) * online_cpus * 100.0
            } else {
                0.0
            };

            if tx.send(cpu_percent).is_err() {
                break;
            }
        }
    });

    (rx, task.abort_handle())
}
