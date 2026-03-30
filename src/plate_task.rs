use crate::logging::Logger;

#[derive(Debug, Clone)]
pub struct PlateDetectTask {
    peak: Option<f64>,
    seen_rising: bool,
    recorded: bool,
}

impl PlateDetectTask {

    // In Kilograms
    const DETECTION_DELTA: f64 = 3.3;

    pub fn new() -> Self {
        Self {
            peak: None,
            seen_rising: false,
            recorded: false,
        }
    }

    pub fn reset(&mut self) {
        self.peak = None;
        self.seen_rising = false;
        self.recorded = false;
    }

    pub fn check(&mut self, weight: f64, logger: &mut Logger) -> bool {

        let Some(current_peak) = self.peak else {
            // First sample initializes peak
            self.peak = Some(weight);
            
            println!("Initialized peak: {}", weight);
            return false;
        };

        // Rising phase → update peak
        if weight > current_peak {
            self.peak = Some(weight);
            self.seen_rising = true;
            self.recorded = false;

            println!("Updated peak: {}", weight);
            return false;
        }

        // Ignore if no rising phase yet or already triggered
        if !self.seen_rising || self.recorded {
            // println!("Recorded or NOT seen rising yet");
            return false;
        }

        // Compute drop from peak
        let drop = current_peak - weight;

        println!("Drop from peak: {}", drop);

        if drop >= Self::DETECTION_DELTA {
            self.recorded = true;
            self.seen_rising = false;

            println!("Detected plate: {:?}", self.peak);
            return true;
        }

        println!("Drop too small");
        false
    }
}