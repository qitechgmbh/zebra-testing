#[derive(Debug, Clone)]
pub struct PlateDetectTask {
    pub peak:    Option<f64>,
    pub trigger: Option<f64>,
}

impl PlateDetectTask {

    pub fn new() -> Self {
        Self {
            peak: None,
            trigger: None,
        }
    }

    pub fn check(&mut self, weight: f64) -> Option<f64> {

        let Some(trigger) = self.trigger else {
            return None;
        };

        let Some(current_peak) = self.peak else {
            if weight >= trigger {
                // initialize peak
                self.peak = Some(weight);
            }
            return None;
        };

        // Rising phase → update peak
        if weight > current_peak {
            self.peak = Some(weight);
            return None;
        }

        // Compute drop from peak
        let drop = current_peak - weight;

        // needs to drop to 1/3 of trigger
        if drop < trigger * 0.66 {
            return None;
        }

        return self.peak.take();
    }

    pub fn reset(&mut self) {
        self.peak = None;
    } 

    pub fn set_trigger(&mut self, value: Option<f64>) {
        self.trigger = value;
        self.reset();
    }   
}