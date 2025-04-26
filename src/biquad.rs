use std::f64::consts::PI;

#[derive(Copy, Clone, PartialEq)]
pub enum FilterAlgorithm {
    Lpf1P,
    Lpf1,
    Hpf1,
    Lpf2,
    Hpf2,
    Bpf2,
    Bsf2,
}

#[derive(Copy, Clone, PartialEq)]
pub struct AudioFilterParameters {
    algorithm: FilterAlgorithm,
    fc: f64,
    q: f64,
    boost_cut_db: f64,
}

impl AudioFilterParameters {
    pub fn new() -> AudioFilterParameters {
        AudioFilterParameters {
            algorithm: FilterAlgorithm::Hpf2,
            fc: 35.0,
            q: 0.71,
            boost_cut_db: 0.0,
        }
    }
}

pub struct Biquad {
    coeff_array: Vec<f64>,
    state_array: Vec<f64>,
}

impl Biquad {
    pub fn new() -> Biquad {
        Biquad {
            coeff_array: vec![0.0; 7],
            state_array: vec![0.0; 4],
        }
    }
    pub fn reset() {}
    
    pub fn process_sample(&mut self, xn: f64) -> f64 {
        // canonical form only
        // mix direct/filtered
        let wn = xn - (self.coeff_array[3] * self.state_array[0]) - (self.coeff_array[4] * self.state_array[1]);
        // apply coefficients to feedback/transfer function
        let yn = self.coeff_array[0] * wn + self.coeff_array[1] * self.state_array[0] + self.coeff_array[2] * self.state_array[1];
        // update state array with new input, shift old input over 1
        self.state_array[1] = self.state_array[0];
        self.state_array[0] = wn;
        // return processed sample
        yn
    }
}
