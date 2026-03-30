#[derive(Debug, Clone)]
pub struct PlateDetectTask {
    peak: Option<f64>,
    seen_rising: bool,
}

impl PlateDetectTask {

    // In Kilograms
    const DETECTION_DELTA: f64 = 3.3;

    pub fn new() -> Self {
        Self {
            peak: None,
            seen_rising: false,
        }
    }

    pub fn check(&mut self, weight: f64) -> Option<f64> {

        let Some(current_peak) = self.peak else {
            // First sample initializes peak
            self.peak = Some(weight);
            return None;
        };

        // Rising phase → update peak
        if weight > current_peak {
            self.peak = Some(weight);
            self.seen_rising = true;
            return None;
        }

        // Ignore if no rising phase yet or already triggered
        if !self.seen_rising {
            // println!("Recorded or NOT seen rising yet");
            return None;
        }

        // Compute drop from peak
        let drop = current_peak - weight;

        if drop >= Self::DETECTION_DELTA {
            self.seen_rising = false;
            return self.peak.take();
        }

        None
    }

    pub fn reset(&mut self) {
        self.peak = None;
        self.seen_rising = false;
    } 
}