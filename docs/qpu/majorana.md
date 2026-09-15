# Majorana Model (Phase 4.3, MODEL)

Computational model only. Satisfies `{gamma_i, gamma_j} = 2 delta_ij` in the
two-mode Pauli sector used for verification tests.

- `MajoranaMode`: stable identity (`PhysicalModeId` + label).
- `ParityOperator P_ij = i gamma_i gamma_j`: symbolic handle, distinct modes.
- `FermionParity`: Even (+1 / bit 0), Odd (-1 / bit 1).
- `Tetron`: four modes gamma1..gamma4, total parity even.
- `MajoranaLogicalQubit`: logical Z from parity sector.
- `MajoranaExchange`: symbolic record, never hardware braiding.

What it is NOT: device physics, microscopic noise, or hardware control.
