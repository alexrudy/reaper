use std::{collections::HashMap, time::Instant};
use sysinfo::Pid;

#[derive(Debug)]
pub struct Datastore {
    cpus: usize,
    processes: HashMap<Pid, Process>,
}

impl Datastore {
    pub fn new(cpus: usize) -> Self {
        Self {
            cpus,
            processes: HashMap::new(),
        }
    }

    pub fn observe(&mut self, pid: Pid, cpu: f64, mem: f64) {
        let process = self
            .processes
            .entry(pid)
            .or_insert(Process::new(pid, self.cpus));
        process.cpu.update(cpu);
        process.mem.update(mem);
    }

    pub fn get(&self, cpu: f64, mem: f64) -> Vec<&Process> {
        self.processes
            .values()
            .filter(|p| p.cpu() >= cpu && p.mem() >= mem)
            .collect()
    }

    pub fn all(&self) -> Vec<&Process> {
        self.processes.values().collect()
    }
}

#[derive(Debug)]
pub struct Process {
    pid: Pid,
    cpu: ExponentialWeightedAverage,
    mem: ExponentialWeightedAverage,
}

impl Process {
    pub fn new(pid: Pid, cpus: usize) -> Self {
        Self {
            pid,
            cpu: ExponentialWeightedAverage::new(0.1, 0.1, 1.0 * (cpus as f64)),
            mem: ExponentialWeightedAverage::new(0.1, 0.1, 1.0),
        }
    }

    pub fn id(&self) -> Pid {
        self.pid
    }

    pub fn cpu(&self) -> f64 {
        self.cpu.get_estimate()
    }

    pub fn mem(&self) -> f64 {
        self.mem.get_estimate()
    }
}

#[derive(Debug)]
pub struct ExponentialWeightedAverage {
    estimate: f64,
    trend: f64,
    alpha: f64,
    beta: f64,
    decay: f64,
    last_seen: Instant,
    maximum: f64,
}

impl ExponentialWeightedAverage {
    pub fn new(alpha: f64, beta: f64, maximum: f64) -> Self {
        ExponentialWeightedAverage {
            estimate: 0.0,
            trend: 0.0,
            alpha,
            beta,
            decay: 3.0,
            last_seen: Instant::now(),
            maximum,
        }
    }

    pub fn update(&mut self, value: f64) {
        let previous_estimate = self.estimate;

        self.estimate = ((self.alpha * value)
            + (1.0 - self.alpha) * (previous_estimate + self.trend))
            .min(self.maximum);
        self.trend =
            self.beta * (self.estimate - previous_estimate) + (1.0 - self.beta) * self.trend;
    }

    pub fn get_estimate(&self) -> f64 {
        self.estimate * compute_decay(self.last_seen, self.decay)
    }
}

fn compute_decay(last_update: Instant, weight: f64) -> f64 {
    let exponent = (-last_update.elapsed().as_secs_f64().max(1.0)) / weight;
    exponent.exp()
}
