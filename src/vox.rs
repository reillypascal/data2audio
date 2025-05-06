pub struct VoxState {
    predictor: i16,
    step_index: i16,
}

impl VoxState {
    pub fn new() -> VoxState {
        VoxState { 
            predictor: 0,
            step_index: 0,
        }
    }
    pub fn vox_decode(&mut self, in_nibble: &u8) -> i16 {
        // get step size from last time's index before updating
        let step_size = VOX_STEP_TABLE[self.step_index as usize];
        // use in_nibble to index into adpcm step table; add to step
        let mut step_index = self.step_index + ADPCM_INDEX_TABLE[*in_nibble as usize];
        // clamp index to size of step table — for next time
        step_index = i16::clamp(step_index, 0, (VOX_STEP_TABLE.len() as i16) - 1);
        
        // sign is 4th bit; magnitude is 3 LSBs
        let sign = in_nibble & 8;
        let delta = in_nibble & 7;
        // delta; after * 2 and >> 3, equivalent to scale of 3 bits in (ss(n)*B2)+(ss(n)/2*B1)+(ss(n)/4*BO) from pseudocode
        // + 1; after >> 3, corresponds to ss(n)/8 from pseudocode — bit always multiplies step, regardless of 3 delta bits on/off
        let diff = ((2 * (delta as i16) + 1) * step_size) >> 3;
        // last time's value
        let mut predictor = self.predictor;
        // if sign bit (4th one) is set, value is negative
        if sign != 0 { predictor -= diff; } 
        else { predictor += diff; }
        
        // clamp output between 12-bit signed min/max value
        self.predictor = i16::clamp(predictor, -i16::pow(2, 11), i16::pow(2, 11) - 1);
        // update for next time through; ss(n+1) into z-1 from block diagram
        self.step_index = step_index;
        // return updated predictor, which is also saved for next time; X(n) into z-1
        // scale from 12-bit to 16-bit; 16 = 2^4, or 4 extra bits
        self.predictor * 16
    }
}
// duplicate values from spec; can index w/ whole nibble, incl sign bit (4th)
// increment up/down thru this table...
const ADPCM_INDEX_TABLE: [i16; 16] = [
    -1, -1, -1, -1, 2, 4, 6, 8,
    -1, -1, -1, -1, 2, 4, 6, 8,
];
// ...use (clamped) index table to index this array for step size
const VOX_STEP_TABLE: [i16; 49] = [
    16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45,
    50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130, 143, 
    157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449, 
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552,
];
