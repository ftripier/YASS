use rand::Rng;

pub enum Gate {
    H,
    X,
    Z,
    Y,
    S,
    Si,
    Sx,
    Sxi,
    Sy,
    Syi,
}

// humble beginnings: a single qubit
// stabilizer simulator with no
// special focus on performance.
pub struct StabilizerSimulator {
    generator_sign_is_negated: bool,
    x_is_a_stabilizer: bool,
    z_is_a_stabilizer: bool,
    rand: rand::rngs::StdRng,
}

impl StabilizerSimulator {
    pub fn new(seed: u64) -> StabilizerSimulator {
        StabilizerSimulator {
            generator_sign_is_negated: false,
            x_is_a_stabilizer: false,
            z_is_a_stabilizer: true,
            rand: rand::SeedableRng::seed_from_u64(seed),
        }
    }

    pub fn seeded() -> StabilizerSimulator {
        StabilizerSimulator::new(0)
    }

    pub fn apply_gate(&mut self, gate: &Gate) {
        match gate {
            Gate::H => {
                // H exchanges the x and z stabilizers, that much is obvious to me.
                std::mem::swap(&mut self.x_is_a_stabilizer, &mut self.z_is_a_stabilizer);
                // but what is the impact on the phase?
                // So, in general H swaps the affect of Pauli X and Pauli Z.
                // That alone shouldn't affect the sign of generators that stabilize
                // the state for eigenstates of either X or Z. You cannot be an
                // eigenstate of both X and Z, since they are conjugates of each other.
                // But, you can be an eigenstate of neither. In that case, you are
                // an eigenstate of Y, and therefore either stabilized by Y or -Y,
                // and thus either iXZ or iZX. H would swap the X and Z parts of these
                // generators, which changes the sign of which Y operator stabilizes
                // the state. So I guess we need to flip the phase of the generator
                // if and only if the state is stabilized by Y.
                if !self.x_is_a_stabilizer && !self.z_is_a_stabilizer {
                    self.generator_sign_is_negated = !self.generator_sign_is_negated;
                }
            }
            Gate::X => {
                // do not affect a state stabilized by X, tautologically.
                // However, states stabilized by Z/-Z will now only be stabilized by their
                // respective negated operator.
                if self.z_is_a_stabilizer {
                    self.generator_sign_is_negated = !self.generator_sign_is_negated;
                }
            }
            Gate::Z => {
                // do not affect a state stabilized by Z, tautologically.
                // However, states stabilized by X/-X will now only be stabilized by their
                // respective negated operator.
                if self.x_is_a_stabilizer {
                    self.generator_sign_is_negated = !self.generator_sign_is_negated;
                }
            }
            Gate::Y => {
                // applying a Y gate to a pauli string would flip both the X and Z parts of the tableau.
                // what does that mean for the generators?
                unimplemented!()
            }
            Gate::S => {
                unimplemented!()
            }
            Gate::Si => {
                unimplemented!()
            }
            Gate::Sx => {
                unimplemented!()
            }
            Gate::Sxi => {
                unimplemented!()
            }
            Gate::Sy => {
                unimplemented!()
            }
            Gate::Syi => {
                unimplemented!()
            }
        }
    }

    pub fn measure(&mut self) -> bool {
        // a state can either be stabilized by pauli X, or pauli Z, but not both, since
        // these two operators are conjugates of each other.
        assert!(self.x_is_a_stabilizer || self.z_is_a_stabilizer);
        if self.z_is_a_stabilizer {
            self.generator_sign_is_negated
        } else {
            // we are stabilized by pauli X, so we are one of the longitudinal
            // eigenstates of the X operator. In the Z basis, these states
            // are superpositions of |0> and |1>, so we should return a
            // uniformly random selection of either true or false
            self.rand.gen()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_i_measured_in_z_basis() {
        let mut stabilizer = StabilizerSimulator::seeded();
        assert!(!stabilizer.measure());
    }

    #[test]
    fn test_x_measured_in_z_basis() {
        let mut stabilizer = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::X);
        assert!(stabilizer.measure());
    }

    #[test]
    fn test_z_measured_in_z_basis() {
        let mut stabilizer = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::Z);
        assert!(!stabilizer.measure());
    }

    #[test]
    fn test_h_z_h_equals_x() {
        let mut stabilizer = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::Z);
        stabilizer.apply_gate(&Gate::H);
        stabilizer.apply_gate(&Gate::Z);
        assert!(stabilizer.measure());
    }
}
