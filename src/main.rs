use std::time::Duration;

use sysinfo::System;

use self::datastore::Datastore;

mod datastore;

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    std::thread::sleep(Duration::from_millis(500));

    let ncpu = sys.cpus().len();
    let memlim = sys.total_memory() / 10;

    let mut data = Datastore::new(ncpu);
    loop {
        sys.refresh_all();

        for (pid, proc) in sys.processes() {
            if proc.cpu_usage() > 0.4 || proc.memory() > memlim {
                data.observe(
                    *pid,
                    proc.cpu_usage() as f64,
                    proc.memory() as f64 / sys.total_memory() as f64,
                );
            }
        }

        if sys.global_cpu_usage() > (0.9 * (ncpu as f32) * 100.0) {
            println!("CPU Threshold Exceeded: {:.2}%", sys.global_cpu_usage());
            let mut procs = data.get(0.5 * (ncpu as f64) * 100.0, 0.2);
            procs.sort_by(|a, b| a.cpu().partial_cmp(&b.cpu()).unwrap().reverse());
            for proc in procs {
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
