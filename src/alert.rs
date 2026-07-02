use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct ThresholdAlert {
    /// Janela de tempo em segundos
    window_secs: u64,
    /// Número de eventos para disparar alerta
    threshold: usize,
    /// IP → lista de timestamps recentes
    counters: HashMap<String, Vec<Instant>>,
}

impl ThresholdAlert {
    pub fn new(threshold: usize, window_secs: u64) -> Self {
        Self {
            window_secs,
            threshold,
            counters: HashMap::new(),
        }
    }

    /// Registra um evento de um IP e retorna true se o threshold foi atingido
    pub fn record(&mut self, ip: &str) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);
        let entry = self.counters.entry(ip.to_string()).or_default();

        // Remove eventos fora da janela de tempo
        entry.retain(|t| now.duration_since(*t) < window);
        entry.push(now);

        entry.len() >= self.threshold
    }

    pub fn count(&self, ip: &str) -> usize {
        self.counters.get(ip).map(|v| v.len()).unwrap_or(0)
    }
}
