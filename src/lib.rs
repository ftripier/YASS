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
    stabilizer_has_x_component: bool,
    stabilizer_has_z_component: bool,
    rand: rand::rngs::StdRng,
}

impl StabilizerSimulator {
    pub fn new(seed: u64) -> StabilizerSimulator {
        StabilizerSimulator {
            generator_sign_is_negated: false,
            stabilizer_has_x_component: false,
            stabilizer_has_z_component: true,
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
                std::mem::swap(
                    &mut self.stabilizer_has_x_component,
                    &mut self.stabilizer_has_z_component,
                );
                // but what is the impact on the phase?
                // So, in general H swaps the affect of Pauli X and Pauli Z.
                // That alone shouldn't affect the sign of generators that stabilize
                // the state for eigenstates of either X or Z individually. If the stabilizer has
                // both an X and a Y component on the qubit, you are
                // an eigenstate of Y, and therefore either stabilized by Y or -Y,
                // and thus either iXZ or iZX. H would swap the X and Z parts of these
                // generators, which changes the sign of which Y operator stabilizes
                // the state. So I guess we need to flip the phase of the generator
                // if and only if the state is stabilized by Y.
                if self.stabilizer_has_x_component && self.stabilizer_has_z_component {
                    self.generator_sign_is_negated = !self.generator_sign_is_negated;
                }
            }
            Gate::X => {
                // do not affect a state stabilized by X, tautologically.
                // However, states stabilized by Z/-Z will now only be stabilized by their
                // respective negated operator.
                if self.stabilizer_has_z_component {
                    self.generator_sign_is_negated = !self.generator_sign_is_negated;
                }
            }
            Gate::Z => {
                // do not affect a state stabilized by Z, tautologically.
                // However, states stabilized by X/-X will now only be stabilized by their
                // respective negated operator.
                if self.stabilizer_has_x_component {
                    self.generator_sign_is_negated = !self.generator_sign_is_negated;
                }
            }
            Gate::Y => {
                // Y = iXZ, so applying Y is equivalent to applying X and Z in sequence.
                // If you were stabilized by X, the stabilizer operator sign ins flipped by applying Z.
                // If you were stabilized by Z, the stabilizer operator sign is flipped by applying X.
                // If you were stabilized by Y, then nothing happens.
                // That means we need X XOR Z in the stabilizer to determine if we need to flip the sign.
                self.generator_sign_is_negated =
                    self.stabilizer_has_x_component ^ self.stabilizer_has_z_component;
            }
            Gate::S => {
                // swaps if a state is stabilized by X or Y. Flips the sign of the operator if the
                // state was previously stabilized by Y. We're just going around the clock
                // of the X Y longitudinal eigenstates, basically.
                // Stabilized by X -> Stabilized by Y -> Stabilized by -X -> Stabilized by -Y -> Stabilized by X, etc.
                self.generator_sign_is_negated ^=
                    self.stabilizer_has_x_component && self.stabilizer_has_z_component;
                self.stabilizer_has_z_component ^= self.stabilizer_has_x_component;
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
        assert!(self.stabilizer_has_x_component || self.stabilizer_has_z_component);
        if self.stabilizer_has_z_component {
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

    #[test]
    fn test_h_y_h_equals_y() {
        let mut stabilizer = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::Y);
        stabilizer.apply_gate(&Gate::H);
        stabilizer.apply_gate(&Gate::Y);
        assert!(stabilizer.measure());
    }

    #[test]
    fn test_h_x_h_equals_z() {
        let mut stabilizer = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::H);
        stabilizer.apply_gate(&Gate::X);
        stabilizer.apply_gate(&Gate::H);
        assert!(!stabilizer.measure());
    }

    #[test]
    fn test_h_s_s_h_equals_x() {
        let mut stabilizer = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::H);
        stabilizer.apply_gate(&Gate::S);
        stabilizer.apply_gate(&Gate::S);
        stabilizer.apply_gate(&Gate::H);
        assert!(stabilizer.measure());
    }
}
