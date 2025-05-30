use crate::gates::Gate;
use rand::Rng;
use std::mem;

// TODO: const N is a choice. It makes things
// easy, but it means
// you can't determine the simulator size
// dynamically. This is something to fix
// later -- we should probably back storage
// by vectors.
#[derive(Debug, Clone)]
struct TableauGeneratorRow<const N: usize> {
    phase_is_negated: bool,
    x_bits: [bool; N],
    z_bits: [bool; N],
}

// humble beginnings: slow stabilizer
// simulator that tracks stabilizers and
// destabilizers for n qubits, and supports
// h, s, and cnot.
pub struct StabilizerSimulator<const N: usize> {
    stabilizers: [TableauGeneratorRow<N>; N],
    destabilizers: [TableauGeneratorRow<N>; N],
    rand: rand::rngs::StdRng,
}

impl<const N: usize> StabilizerSimulator<N> {
    pub fn new(seed: u64) -> StabilizerSimulator<N> {
        let mut initial_stabilizers: [TableauGeneratorRow<N>; N] = unsafe { mem::zeroed() };
        let mut initial_destabilizers: [TableauGeneratorRow<N>; N] = unsafe { mem::zeroed() };
        for i in 0..N {
            initial_stabilizers[i] = TableauGeneratorRow {
                phase_is_negated: false,
                x_bits: [false; N],
                z_bits: [false; N],
            };
            initial_destabilizers[i] = TableauGeneratorRow {
                phase_is_negated: false,
                x_bits: [false; N],
                z_bits: [false; N],
            };
        }

        // initialize the stabilizers and destabilziers of the
        // |0...0> state. -- Z stabilizes 0, and X destabilizes 0.
        // The generators for the stabilizers and
        // destabilizers are the product of terms like ZI*...*I and
        // XI*...*I, respectively. We just need N of each generator with
        // a single Z or X acting on each qubit. From there, all stabilizer
        // pauli strings can be generated by the product of these generators.
        for i in 0..N {
            initial_stabilizers[i].z_bits[i] = true;
            initial_destabilizers[i].x_bits[i] = true;
        }

        StabilizerSimulator {
            stabilizers: initial_stabilizers,
            destabilizers: initial_destabilizers,
            rand: rand::SeedableRng::seed_from_u64(seed),
        }
    }

    pub fn seeded() -> StabilizerSimulator<N> {
        StabilizerSimulator::new(0)
    }

    pub fn apply_gate(&mut self, gate: &Gate) {
        match gate {
            // TODO: I wonder if I should move the dispatch to a trait
            // on the gates enum. This is probably only important in a world
            // where I have multiple clients for the gate type, which seems
            // out of scope for this project.
            //
            // All gates act on stabilizer and destabilizer generators in the same way,
            // given that they maintain their initial relationships to each other as an invariant.
            //
            // In particular, you need all destabilizers to commute with each other, and for
            // each i in 1..n, the ith destabilizer must anticommute with the ith stabilizer,
            // but commute with all other stabilizers. This is the tableau convention.
            Gate::H(qubit) => {
                for i in 0..N {
                    for generator in
                        [&mut self.stabilizers[i], &mut self.destabilizers[i]].iter_mut()
                    {
                        let generator_x_component = generator.x_bits[*qubit as usize];
                        let generator_z_component = generator.z_bits[*qubit as usize];
                        //H swaps X and Z components of the stabilizer. Y == -iZX, which we turn into
                        // -iXZ == -Y. So we just need to flip the sign of the stabilizer if it has both
                        // X and Z components.
                        // Otherwise, if you are stabilized by only X, you are one of |+> or |->. Hadamard
                        // Will simply map you to |0> |1> with the same generator phase. If you are stabilized
                        // by only Z, you are one of |0> or |1>. Hadamard will map you to |+> |-> with the same
                        // generator phase.
                        // In general, H maps X and Z stabilizer states to the Z and X stabilizer states, respectively,
                        // and with the same phase.
                        generator.phase_is_negated ^=
                            generator_x_component && generator_z_component;
                        mem::swap(
                            &mut generator.x_bits[*qubit as usize],
                            &mut generator.z_bits[*qubit as usize],
                        )
                    }
                }
            }
            Gate::S(qubit) => {
                for i in 0..N {
                    for generator in
                        [&mut self.stabilizers[i], &mut self.destabilizers[i]].iter_mut()
                    {
                        // the S gate cycles through the Y and X stabilizers longitudinally, in a
                        // X, Y, -X, -Y pattern, assuming you start in |+>.
                        // That means, if you are a Y stabilizer (you have both X and Z components),
                        // you will be mapped to an X stabilizer with an opposing phase. If you are an X
                        // stabilizer, you will be mapped to a Y stabilizer with the same phase.
                        let generator_x_component = generator.x_bits[*qubit as usize];
                        let generator_z_component = generator.z_bits[*qubit as usize];
                        // flip phase of Y stabilizers.
                        generator.phase_is_negated ^=
                            generator_x_component && generator_z_component;

                        // cycle through X and Y stabilizers.
                        generator.z_bits[*qubit as usize] ^= generator_x_component;
                    }
                }
            }
            Gate::Cx(control, target) => {
                for i in 0..N {
                    for generator in
                        [&mut self.stabilizers[i], &mut self.destabilizers[i]].iter_mut()
                    {
                        // the rules for a CNOT acting on a generator are less intuitive for me. In the heisenberg picture,
                        // CNOT acts on future stabilizers by conjugating them with the CNOT gate. So something like
                        // CNOT * generator * CNOT. This ends up working on the pauli basis like so:
                        // CNOT * Z ⊗ I * CNOT = Z ⊗ I
                        // CNOT * I ⊗ Z * CNOT = Z ⊗ Z
                        // CNOT * Z ⊗ Z * CNOT = I ⊗ Z
                        // CNOT * X ⊗ I * CNOT = X ⊗ X
                        // CNOT * I ⊗ X * CNOT = I ⊗ X
                        // CNOT * X ⊗ X * CNOT = X ⊗ I
                        // and for action on Y operators you can take the product of X and Z cases.
                        generator.x_bits[*target as usize] ^= generator.x_bits[*control as usize];
                        generator.z_bits[*control as usize] ^= generator.z_bits[*target as usize];
                        // we invert the phase if CNOT would negate a pauli operator in the heisenberg picture.
                        // that is to say, something like CNOT * (P1 ⊗ P2) * CNOT = -P1 ⊗ P2.
                        // This happens when the control qubit is stabilized by X, and the target qubit is stabilized by Z.
                        // Because CNOT * (X ⊗ I * I ⊗ Z) * CNOT =
                        // (CNOT * (X ⊗ I) * CNOT)(CNOT * (I ⊗ Z) * CNOT) =
                        // (X ⊗ X)(Z ⊗ Z) or (Z ⊗ Z)(X ⊗ X)
                        // so either
                        // iY ⊗ iY = -(Y ⊗ Y).
                        // or -iY ⊗ -iY = -(Y ⊗ Y).
                        let add_phase_flip = generator.x_bits[*control as usize]
                            && generator.z_bits[*target as usize];
                        // However, if you have an odd balance of X and Y components, the anticommutation rules described
                        // above cancel out. E.g. CNOT(Y ⊗  X)CNOT = Y ⊗ I
                        let anticommutation_parity = generator.z_bits[*control as usize]
                            ^ generator.x_bits[*target as usize]
                            ^ true;
                        generator.phase_is_negated ^= add_phase_flip && anticommutation_parity;
                    }
                }
            }
        }
    }

    fn is_deterministic(&self, qubit: u32) -> bool {
        // are there no stabilizer rows with an X component at the qubit?
        // if so, we're chillin -- we are already in the Z measurement basis because
        // we are either stabilized by Z or -Z, and so either |0> or |1>.
        self.find_x_stabilizer_index(qubit).is_none()
    }

    fn pauli_imaginary_phase_exponent(x1: bool, z1: bool, x2: bool, z2: bool) -> i32 {
        // return the sign to which i is raised when the pauli matrices represented by x1*z1 and x2*z2 are multiplied.
        // e.g. X*X = I. X*Z = iY. Z*Z = I. Z*X = -iY. etc.
        // I've used scott aaronson's math here, and it checks out.
        match (x1, z1) {
            (false, false) => 0,
            (true, true) => z2 as i32 - x2 as i32,
            (true, false) => (z2 as i32) * (2 * x2 as i32 - 1),
            (false, true) => (1 - 2 * z2 as i32) * x2 as i32,
        }
    }

    fn rowsum(
        row_h: &mut TableauGeneratorRow<N>,
        row_i: &TableauGeneratorRow<N>,
    ) -> Result<(), &'static str> {
        let mut exponent_sum: i32 = 0;
        for j in 0..N {
            exponent_sum += Self::pauli_imaginary_phase_exponent(
                row_i.x_bits[j],
                row_i.z_bits[j],
                row_h.x_bits[j],
                row_h.z_bits[j],
            );
        }
        let pauli_operator_phase =
            2 * (row_h.phase_is_negated as i32) + 2 * (row_i.phase_is_negated as i32);
        let pauli_operator_phase = (pauli_operator_phase + exponent_sum) % 4;
        if pauli_operator_phase == 0 {
            row_h.phase_is_negated = false;
        } else if pauli_operator_phase == 2 {
            row_h.phase_is_negated = true;
        } else {
            // TODO -- maybe use anyhow results and dynamic strings.
            return Err("Non-stabilizer rowsum");
        }
        for j in 0..N {
            row_h.x_bits[j] ^= row_i.x_bits[j];
            row_h.z_bits[j] ^= row_i.z_bits[j];
        }
        Ok(())
    }

    fn find_x_stabilizer_index(&self, qubit: u32) -> Option<usize> {
        self.stabilizers
            .iter()
            .position(|row| row.x_bits[qubit as usize])
    }

    fn extract_stabilizer_p_after_flipping_preparing_other_stabilizers_to_expect_collapsed_state(
        &mut self,
        qubit: u32,
        p: usize,
    ) -> Result<(), &'static str> {
        // helper method for nondeterministic_measurement
        let p_stabilizer = self.stabilizers[p].clone();
        for i in 0..N {
            if i == p {
                continue;
            }
            if self.stabilizers[i].x_bits[qubit as usize] {
                Self::rowsum(&mut self.stabilizers[i], &p_stabilizer)?;
            }
            if self.destabilizers[i].x_bits[qubit as usize] {
                Self::rowsum(&mut self.destabilizers[p], &p_stabilizer)?;
            }
        }
        Ok(())
    }

    fn collapse_p_stabilizer_and_return_measurement_outcome(
        &mut self,
        p: usize,
        qubit: u32,
    ) -> Result<bool, &'static str> {
        // helper method for nondeterministic_measurement
        let old_p_stabilizer = mem::replace(
            &mut self.stabilizers[p],
            TableauGeneratorRow {
                phase_is_negated: self.rand.gen_bool(0.5),
                x_bits: [false; N],
                z_bits: [false; N],
            },
        );
        self.stabilizers[p].z_bits[qubit as usize] = true;
        self.destabilizers[p] = old_p_stabilizer;
        Ok(self.stabilizers[p].phase_is_negated)
    }

    fn nondeterministic_measurement(&mut self, qubit: u32) -> Result<bool, &'static str> {
        // 1. find index p amoung stabilizers such that stabilizers[p][x_bits][qubit] = 1
        //
        // 1. add all rows (i, p)  for all i over stabilizers[i] and destabilizers[i] such
        // that i != p and x_ia = 1
        //
        // 2. set the corresponding destabilizer[p] to be equal to the former destabilizer row.
        //
        // 3. set the pth row to be identically 0 except for z[qubit] = 1, and the phase is either
        // negated or not with equal probability. Return if the phase was negated or not as the measurement outcome.
        //
        //
        // This mimes the process of collapsing the state at the qubit to either |0> or |1>.
        // First you prepare the tableau to accept the new reality of a collapsed measurement
        // on the P stabilizer by mutating stabilizers with an x component on the qubit
        // to now stabilize the state in the Z basis.
        // Then you make sure the pth stabilizer's destabilizer is prepared to anticommute with the
        // stabilizer.
        // Then you collapse the pth stabilizer to either |0> or |1> by setting the z component
        // to 1, and the phase to either -1 or 1 with equal probability.
        let p = self.find_x_stabilizer_index(qubit);
        if p.is_none() {
            return Err("No stabilizer row with X component at qubit -- we should've checked for this already when we were determining if the measurement was deterministic or not.");
        }
        let p = p.unwrap();
        self.extract_stabilizer_p_after_flipping_preparing_other_stabilizers_to_expect_collapsed_state(qubit, p)?;
        self.collapse_p_stabilizer_and_return_measurement_outcome(p, qubit)
    }

    fn determine_deterministic_measurement(&mut self, qubit: u32) -> Result<bool, &'static str> {
        let mut scratch_row = TableauGeneratorRow {
            phase_is_negated: false,
            x_bits: [false; N],
            z_bits: [false; N],
        };
        // try and determine if Z or -Z on the qubit is a stabilizer of the state.
        // You need to sum up a subset of stabilizer generators that produces +-Z[qubit] with
        // identity on all other qubits. The choice of which stabilizers to include in this
        // group product is predicated on their corresponding destabilizer anticommuting with
        // +-Z[qubit]; the destabilizers in the tableau convention are constructed so that
        // only corresponding stabilizers and destabilizers anticommute with each other. This implies
        // that all stabilizers that _are_ involved in the group product to construct Z[qubit]
        // must have a corresponding destabilizer that anticommutes with Z[qubit].
        // In other words, destabilizers are intentionally constructed to maintain an invariant that they anticommute
        // one-to-one with the stabilizer on the corresponding index. This means if a stabilizer generator would
        // be part of a group product to produce a given stabilizer element, the corresponding destabilizer generator
        // would anticommute with the stabilizer generator.
        for (destabilizer_row, stabilizer_row) in self
            .destabilizers
            .iter_mut()
            .zip(self.stabilizers.iter_mut())
        {
            if destabilizer_row.x_bits[qubit as usize] {
                Self::rowsum(&mut scratch_row, stabilizer_row)?;
            }
        }
        Ok(scratch_row.phase_is_negated)
    }

    pub fn measure(&mut self, qubit: u32) -> Result<bool, &'static str> {
        if self.is_deterministic(qubit) {
            self.determine_deterministic_measurement(qubit)
        } else {
            self.nondeterministic_measurement(qubit)
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_i_measured_in_z_basis() {
        let mut stabilizer: StabilizerSimulator<1> = StabilizerSimulator::seeded();
        assert!(!stabilizer.measure(0).unwrap());
    }

    #[test]
    fn test_h_s_s_h_equals_x() {
        let mut stabilizer: StabilizerSimulator<1> = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::H(0));
        stabilizer.apply_gate(&Gate::S(0));
        stabilizer.apply_gate(&Gate::S(0));
        stabilizer.apply_gate(&Gate::H(0));
        assert!(stabilizer.measure(0).unwrap());
    }

    #[test]
    fn test_cnot_when_control_is_zero() {
        let mut stabilizer: StabilizerSimulator<2> = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::Cx(0, 1));
        assert!(!stabilizer.measure(0).unwrap());
        assert!(!stabilizer.measure(1).unwrap());
    }

    #[test]
    fn test_cnot_when_control_is_one() {
        let mut stabilizer: StabilizerSimulator<2> = StabilizerSimulator::seeded();
        stabilizer.apply_gate(&Gate::H(0));
        stabilizer.apply_gate(&Gate::S(0));
        stabilizer.apply_gate(&Gate::S(0));
        stabilizer.apply_gate(&Gate::H(0));
        stabilizer.apply_gate(&Gate::Cx(0, 1));
        assert!(stabilizer.measure(0).unwrap());
        assert!(stabilizer.measure(1).unwrap());
    }

    #[test]
    fn test_nondeterministic_measurement() {
        // tests that we can expect either |0> or |1> when preparing and measuring multiple copies of
        // |+> |-> or the Y eigenstates. Our stabilizer simulator is seeded, so, once we have passed
        // with a given configuration, we should expect this test to pass deterministically.

        let mut stabilizer: StabilizerSimulator<2> = StabilizerSimulator::seeded();
        let mut results = HashSet::new();
        // s_reps = 0, 1, 2, 3.
        // The amount of s gates to apply after hadamard.
        // Lets us check all possible single
        // qubit superpositon states.
        for s_reps in 0..4 {
            // try multiple times so we can be relatively sure
            // we will produce either |0> or |1> at least once.
            // We only have 1 - 0.5^10 chance of not getting either,
            // e.g. 99.9%+ chance of getting getting both.
            for _ in 0..10 {
                stabilizer.apply_gate(&Gate::H(0));
                for _ in 0..s_reps {
                    // the amount of additional S gates determines
                    // which X/Y eigenstate we are in.
                    // 0 -- |+>
                    // 1 -- Y stabilizer state
                    // 2 -- |->
                    // 3 -- -Y stabilizer state
                    stabilizer.apply_gate(&Gate::S(0));
                }
                let result = stabilizer.measure(0).unwrap();
                results.insert(result);
            }
            assert!(results.len() == 2);
            assert!(results.contains(&true));
            assert!(results.contains(&false));
            results.clear();
        }
    }
}
