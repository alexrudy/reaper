use std::time::Duration;

use clap::Parser as _;
use sysinfo::System;

use self::datastore::Datastore;

mod datastore;

#[derive(Debug, clap::Parser)]
struct Args {
    /// CPU usage threshold
    #[clap(long, default_value_t = 90)]
    cpu: u32,

    /// Memory usage threshold
    #[clap(long, default_value_t = 90)]
    mem: u32,

    /// Polling interval
    #[clap(long, default_value_t = 50)]
    interval: u32,
}

fn main() {
    let args = Args::parse();

    let mut sys = System::new_all();
    sys.refresh_all();

    std::thread::sleep(Duration::from_millis(500));

    let ncpu = sys.cpus().len();
    let memlim = sys.total_memory() / 10;

    let cpu_recording = (ncpu as f32) * 5.0;
    let cpu_threshold = (ncpu as f32) * (args.cpu as f32);

    let mut data = Datastore::new(ncpu);
    loop {
        sys.refresh_all();

        for (pid, proc) in sys.processes() {
            if proc.cpu_usage() > cpu_recording || proc.memory() > memlim {
                data.observe(
                    *pid,
                    proc.cpu_usage() as f64,
                    proc.memory() as f64 / sys.total_memory() as f64,
                );
            }
        }

        if sys.global_cpu_usage() > cpu_threshold {
            println!("CPU Threshold Exceeded: {:.2}%", sys.global_cpu_usage());
            let mut procs = data.get(0.5 * (ncpu as f64) * 100.0, 0.2);
            if procs.is_empty() {
                procs = data.all();
            }

            procs.sort_by(|a, b| a.cpu().partial_cmp(&b.cpu()).unwrap().reverse());
            for proc in procs.iter().take(20) {
                println!(
                    "PID: {}, CPU: {:.2}%, MEM: {:.2}%",
                    proc.id(),
                    proc.cpu() * 100.0,
                    proc.mem() * 100.0
                );
            }
        }

        std::thread::sleep(Duration::from_millis(50));
    }
}
