// config

use crate::state::{SharedState, State};

const DEFAULT_AMOUNT: f64 = 40.;
const STRENGHT: f64 = 0.385;

// mock
const PERMILLES_FROM_DRINK: f64 = 0.5;

// assume one tick is 5min

const TICKS_TO_HIT: usize = 4;

// one drink takes about 2h to burn
const BURNRATE: f64 = 1. / 24.;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlcoholRequest {
    None,
    Shot  { amount: f64 }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlcoholStatus {
    Uninitialized,
    Calibrated,
}

impl Default for AlcoholStatus {
    fn default() -> Self {
        Self::Uninitialized
    }
}

impl Into<bool> for AlcoholStatus {
    fn into(self) -> bool {
        match self {
            AlcoholStatus::Uninitialized => true,
            AlcoholStatus::Calibrated => false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Alcohol {
    pub current: f64,
    pub target: f64,
    pub status: AlcoholStatus,
    pub tick: usize,

    // this should and will be refactored
    queue: Vec<Vec<f64>>,
    estimate: f64,
}

impl Alcohol {
    pub fn update_target(&mut self, target: f64) {
        self.target = target;
    }

    pub fn calibrate(&mut self, current: f64) {
        self.current = current;
        self.status = AlcoholStatus::Calibrated;
        self.tick = 0;
    }

    pub fn reset(&mut self) {
        self.status = AlcoholStatus::Uninitialized;
        self.target = 0.;
        self.current = 0.;
        self.tick = 0;

        self.queue.clear();
        self.estimate = 0.0;
    }

    pub fn drink(&mut self) {
        self.queue.push(
            std::iter::repeat(PERMILLES_FROM_DRINK / (TICKS_TO_HIT as f64))
                .take(TICKS_TO_HIT)
                .collect(),
        );
        
    }

    pub fn tick(&mut self) -> AlcoholRequest {
        let mut request = AlcoholRequest::None;
        // apply all the drinks
        let mut up = 0.;
        for drink in self.queue.iter_mut() {
            if let Some(amount) = drink.pop() {
                up += amount;
            }
        }

        self.queue = self.queue.drain(..).filter(|v| !v.is_empty()).collect();

        let delta = up - BURNRATE;
        self.current = f64::max(self.current + delta, 0.);

        self.estimate = self.estimate();

        // require calibration

        if self.tick > 24 {
            self.status = AlcoholStatus::Uninitialized;
        }

        // "köyhän miehen PID

        if self.status == AlcoholStatus::Calibrated {

            if self.estimate < self.target {
                self.drink();
                request = AlcoholRequest::Shot { amount: DEFAULT_AMOUNT };
            }
        }

        self.tick += 1;

        request
    }


    fn estimate(&self) -> f64 {
        self.queue.iter().flatten().sum::<f64>() + self.current
    } 
}
