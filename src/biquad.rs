use std::f64::consts::PI;

use crate::cli::Args;

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
    pub fn new(
        algorithm: FilterAlgorithm,
        fc: f64,
        q: f64,
        boost_cut_db: f64,
    ) -> AudioFilterParameters {
        AudioFilterParameters {
            algorithm,
            fc,
            q,
            boost_cut_db,
        }
    }
}

pub struct Biquad {
    coeff_array: Vec<f64>,
    state_array: Vec<f64>,
}

// see https://rust-lang.github.io/rust-clippy/master/index.html#new_without_default
// The user might expect to be able to use Default as the type can be constructed without arguments.
impl Default for Biquad {
    fn default() -> Self {
        Biquad::new()
    }
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
        let wn = xn
            - (self.coeff_array[3] * self.state_array[0])
            - (self.coeff_array[4] * self.state_array[1]);
        // apply coefficients to feedback/transfer function
        let yn = self.coeff_array[0] * wn
            + self.coeff_array[1] * self.state_array[0]
            + self.coeff_array[2] * self.state_array[1];
        // update state array with new input, shift old input over 1
        self.state_array[1] = self.state_array[0];
        self.state_array[0] = wn;
        // return processed sample
        yn
    }
}

pub struct AudioFilter {
    parameters: AudioFilterParameters,
    biquad: Biquad,
    sample_rate: u32,
}

impl AudioFilter {
    pub fn new(params: &AudioFilterParameters, sample_rate: u32) -> AudioFilter {
        AudioFilter {
            parameters: *params,
            biquad: Biquad::new(),
            sample_rate,
        }
    }

    pub fn get_params(&self) -> AudioFilterParameters {
        self.parameters // does this move?
    }

    pub fn set_params(&mut self, params: AudioFilterParameters) {
        self.parameters = params;

        if self.parameters.q <= 0.0 {
            self.parameters.q = 0.707;
        }

        self.calculate_filter_coeffs();
    }

    pub fn reset(&self) {}

    pub fn process_sample(&mut self, xn: f64) -> f64 {
        self.biquad.coeff_array[6] * xn
            + self.biquad.coeff_array[5] * self.biquad.process_sample(xn)
    }

    // pub fn process_vec<T>(&mut self, data: &mut Vec<T>, args: &Args)
    // // -> Vec<T>
    // where
    //     T: Copy + From<f64> + Into<f64>,
    //     // f64: std::convert::From<T>,
    //     // T: FromPrimitive,
    // {
    //     // make filter
    //     let filter_params = AudioFilterParameters::new(FilterAlgorithm::Hpf2, 20.0, 0.707, 0.0);
    //     let mut filter = AudioFilter::new(&filter_params, args.samplerate);
    //     filter.calculate_filter_coeffs();
    //     let gain_lin = f64::powf(10.0, args.gain / 20.0);
    //
    //     // filter audio
    //     for sample in data {
    //         *sample = (filter.process_sample((*sample as f64) * gain_lin)) as T;
    //     }
    // }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn calculate_filter_coeffs(&mut self) {
        self.biquad.coeff_array.fill(0.0);

        self.biquad.coeff_array[0] = 1.0; // a0
        self.biquad.coeff_array[5] = 1.0; // c0
        self.biquad.coeff_array[6] = 0.0; // d0

        let filter_algorithm = self.parameters.algorithm;
        let fc = self.parameters.fc;
        let q = self.parameters.q;

        if filter_algorithm == FilterAlgorithm::Hpf2 {
            let theta_c = (2.0 * PI * fc) / self.sample_rate as f64;
            let d = 1.0 / q;

            let beta_numerator = 1.0 - (d / 2.0) * f64::sin(theta_c);
            let beta_denominator = 1.0 + (d / 2.0) * f64::sin(theta_c);
            let beta = 0.5 * (beta_numerator / beta_denominator);

            let gamma = (0.5 + beta) * f64::cos(theta_c);
            let alpha = (0.5 + beta + gamma) / 2.0;

            self.biquad.coeff_array[0] = alpha; // a0
            self.biquad.coeff_array[1] = -alpha * 2.0; // a1
            self.biquad.coeff_array[2] = alpha; // a2
            self.biquad.coeff_array[3] = -2.0 * gamma; // b1
            self.biquad.coeff_array[4] = 2.0 * beta; // b2
        }
    }
}
