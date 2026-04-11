// config

use crate::packets::Estimate;

// assume one tick is 5min
const TICKS_TO_HIT: usize = 4;

// In one hour the human body will remove about 1.5 permille from the body.
const BURNRATE_H: f64 = 0.00015;

const BURNRATE: f64 = (BURNRATE_H / 60.0) * TICK_TIME as f64;

/// Time of one tick in minutes
const TICK_TIME: u32 = 5;

/// Maximum amout of alcohol given per tick in ml
const MAX_ALCOHOL: f64 = 20.;

/// Maximum amout of liquid given per tick in ml
const MAX_LIQUID: f64 = 40.;

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

#[derive(Debug, Default, Clone)]
pub struct Alcohol {
    /// Estimate of the current BAC
    pub current: f64,
    /// Current target BAC
    pub target: f64,
    pub status: AlcoholStatus,
    pub tick: usize,
    pub person: Person,
    /// Current drink ABV. For exmaple vodka is 40.0
    pub current_drink_abv: f64,
    /// Amount of ethanol in the Liver
    pub liver: f64,
    pub queue: Vec<f64>,
    pub history: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct Person {
    pub gender: Gender,
    /// Weight in KG
    pub weight: f64,
    /// Height in CM
    pub height: f64,
}

impl Default for Person {
    fn default() -> Self {
        Self {
            gender: Default::default(),
            weight: 80.,
            height: 180.,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum Gender {
    Male,
    Female,
    #[default]
    NonBinary,
}

impl Person {
    /// Returns the amount of alcohol needed for a certain raise in BAC
    pub fn get_amount_needed(&self, delta_bac: &f64) -> f64 {
        let mass: f64 = 10. * delta_bac * self.get_wild_mark_factor() * self.weight;
        return mass;
    }

    pub fn get_promiles(&self, mass_ml: &f64) -> f64 {
        return mass_ml / (10. * (self.get_wild_mark_factor() * self.weight));
    }

    // TODO: Make this gender specific
    pub fn get_wild_mark_factor(&self) -> f64 {
        0.31608 - 0.004_821 * self.weight + 0.4632 * self.height
    }
}

impl Alcohol {
    pub fn update_target(&mut self, target: f64) {
        self.target = target;
    }

    pub fn update_drink(&mut self, new_abv: f64) {
        self.current_drink_abv = new_abv;
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
        self.current_drink_abv = 0.;

        self.liver = 0.0;
    }

    fn update_current(&mut self) {
        if self.tick >= TICKS_TO_HIT {
            match self.queue.get(self.tick - TICKS_TO_HIT) {
                Some(x) => self.current += self.person.get_promiles(x),
                None => (),
            }
        }

        if self.current > 0. {
            self.current -= f64::min(self.current, BURNRATE)
        }
    }

    pub fn estimate(&self)-> Estimate{
        Estimate{ history: self.get_history(), future: self.get_future() }
    }

    fn get_history(&self) -> [f64; 20]{
        let end:usize = self.tick;
        let start:usize = i64::max(self.tick as i64-20,0) as usize;
        let start_index = 20 + start - end;
        let mut res = [0.;20];
        for i in 0..usize::min(20,end-start) {
            res[i+start_index]=self.history[start+i];
        }
        res
    }

    fn get_future(&self) -> [f64;20]{
        self.simulate()
    }

    fn simulate(&self) -> [f64;20]{
        let mut res = [0.;20];
        let mut new = self.clone();
        for i in 0..20{
            new.tick();
            res[i] = new.current;
        }
        res
    }

    ///Runs one tick and returns the amount of alcohol to be given
    pub fn tick(&mut self) -> Option<f64> {
        self.update_current();
        if self.tick > 24 {
            self.status = AlcoholStatus::Uninitialized;
        }
        self.history.push(self.current);

        // "köyhän miehen PID
        if self.estimate_forward(TICKS_TO_HIT) < self.target {
            let diff: f64 = self.target - self.estimate_forward(TICKS_TO_HIT);
            self.tick += 1;
            let amount = self.calculate_amount(diff);
            self.queue.push(amount / 100. * self.current_drink_abv);
            return Option::Some(amount);
        }
        self.queue.push(0.);
        self.tick += 1;
        None
    }

    /// Estimates BAC at in the future at n tics, without any interaction
    fn estimate_forward(&self, ticks: usize) -> f64 {
        let mut current = self.current;
        for t in self.tick..=self.tick + ticks {
            if t >= TICKS_TO_HIT {
                match self.queue.get(t - TICKS_TO_HIT) {
                    Some(x) => current += self.person.get_promiles(x),
                    None => (),
                }
            }
            if current > 0. {
                current -= f64::min(current, BURNRATE)
            }
        }
        current
    }

    fn calculate_amount(&self, diff: f64) -> f64 {
        let amount_needed = self.person.get_amount_needed(&diff);
        let liquid: f64 = amount_needed * 100. / self.current_drink_abv;
        let liquid_claped = f64::min(liquid, MAX_LIQUID);
        let alhocol = liquid_claped * self.current_drink_abv / 100.;
        let alcohol_clamped = f64::min(alhocol, MAX_ALCOHOL);
        alcohol_clamped / self.current_drink_abv * 100.
    }
}

#[cfg(test)]
pub mod tests {
    use super::Alcohol;

    #[test]
    fn test_calculate_amount() {
        let mut alc: Alcohol = Alcohol::default();
        alc.update_target(0.001);
        alc.update_drink(38.5);
        for _ in 0..25 {
            alc.tick();
        }
        assert!(f64::abs(alc.current-0.001)<=0.0002)
    }

    #[test]
    fn test_estimate() {
        let mut alc: Alcohol = Alcohol::default();
        alc.update_target(0.001);
        alc.update_drink(38.5);
        for _ in 0..25 {
            alc.tick();
            println!("{:?}",alc.estimate());
        }
    }
}
