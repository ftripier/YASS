pub enum Gate {
    // keep qubit indiciis as u32 for some
    // semblance of an upper bound on the number of qubits.
    // has the benefit of preparing the code to handle
    // type indirection between qubit register vector indexing
    // (in usize) and qubit index (in u32 for now).
    H(u32),
    S(u32),
    Cx(u32, u32),
}