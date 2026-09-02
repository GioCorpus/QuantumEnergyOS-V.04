use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Quantum gates supported by the simulator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuantumGate {
    /// Hadamard gate: creates superposition
    Hadamard,

    /// Pauli-X (NOT) gate: flips qubit
    PauliX,

    /// Pauli-Y gate: rotation around Y-axis
    PauliY,

    /// Pauli-Z gate: phase flip
    PauliZ,

    /// Phase gate: applies phase
    Phase { theta: f64 },

    /// Rotation around X-axis
    RotationX { theta: f64 },

    /// Rotation around Y-axis
    RotationY { theta: f64 },

    /// Rotation around Z-axis
    RotationZ { theta: f64 },

    /// Controlled-NOT (CNOT) gate: two-qubit gate
    CNOT { control: usize, target: usize },

    /// Controlled-Z gate: two-qubit gate
    ControlledZ { control: usize, target: usize },

    /// Swap gate: exchanges two qubits
    Swap { qubit1: usize, qubit2: usize },

    /// Toffoli gate (CCX): three-qubit gate
    Toffoli { control1: usize, control2: usize, target: usize },

    /// Measurement: collapses qubit to 0 or 1
    Measurement { qubit: usize },
}

impl QuantumGate {
    /// Get a 2x2 single-qubit gate matrix
    pub fn single_qubit_matrix(&self) -> Option<[[Complex; 2]; 2]> {
        match self {
            QuantumGate::Hadamard => {
                let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
                Some([
                    [Complex::new(inv_sqrt2, 0.0), Complex::new(inv_sqrt2, 0.0)],
                    [Complex::new(inv_sqrt2, 0.0), Complex::new(-inv_sqrt2, 0.0)],
                ])
            }
            QuantumGate::PauliX => {
                Some([
                    [Complex::zero(), Complex::one()],
                    [Complex::one(), Complex::zero()],
                ])
            }
            QuantumGate::PauliY => {
                Some([
                    [Complex::zero(), Complex::new(0.0, -1.0)],
                    [Complex::new(0.0, 1.0), Complex::zero()],
                ])
            }
            QuantumGate::PauliZ => {
                Some([
                    [Complex::one(), Complex::zero()],
                    [Complex::zero(), Complex::new(-1.0, 0.0)],
                ])
            }
            QuantumGate::Phase { theta } => {
                Some([
                    [Complex::one(), Complex::zero()],
                    [Complex::zero(), Complex::polar(1.0, *theta)],
                ])
            }
            QuantumGate::RotationX { theta } => {
                let cos_half = (theta / 2.0).cos();
                let sin_half = (theta / 2.0).sin();
                Some([
                    [Complex::new(cos_half, 0.0), Complex::new(0.0, -sin_half)],
                    [Complex::new(0.0, -sin_half), Complex::new(cos_half, 0.0)],
                ])
            }
            QuantumGate::RotationY { theta } => {
                let cos_half = (theta / 2.0).cos();
                let sin_half = (theta / 2.0).sin();
                Some([
                    [Complex::new(cos_half, 0.0), Complex::new(-sin_half, 0.0)],
                    [Complex::new(sin_half, 0.0), Complex::new(cos_half, 0.0)],
                ])
            }
            QuantumGate::RotationZ { theta } => {
                Some([
                    [Complex::polar(1.0, -theta / 2.0), Complex::zero()],
                    [Complex::zero(), Complex::polar(1.0, theta / 2.0)],
                ])
            }
            _ => None,
        }
    }

    /// Check if this is a single-qubit gate
    pub fn is_single_qubit(&self) -> bool {
        matches!(
            self,
            QuantumGate::Hadamard
                | QuantumGate::PauliX
                | QuantumGate::PauliY
                | QuantumGate::PauliZ
                | QuantumGate::Phase { .. }
                | QuantumGate::RotationX { .. }
                | QuantumGate::RotationY { .. }
                | QuantumGate::RotationZ { .. }
                | QuantumGate::Measurement { .. }
        )
    }

    /// Check if this is a two-qubit gate
    pub fn is_two_qubit(&self) -> bool {
        matches!(
            self,
            QuantumGate::CNOT { .. }
                | QuantumGate::ControlledZ { .. }
                | QuantumGate::Swap { .. }
        )
    }

    /// Check if this is a three-qubit gate
    pub fn is_three_qubit(&self) -> bool {
        matches!(self, QuantumGate::Toffoli { .. })
    }

    /// Get the name of the gate
    pub fn name(&self) -> &str {
        match self {
            QuantumGate::Hadamard => "H",
            QuantumGate::PauliX => "X",
            QuantumGate::PauliY => "Y",
            QuantumGate::PauliZ => "Z",
            QuantumGate::Phase { .. } => "Phase",
            QuantumGate::RotationX { .. } => "RX",
            QuantumGate::RotationY { .. } => "RY",
            QuantumGate::RotationZ { .. } => "RZ",
            QuantumGate::CNOT { .. } => "CNOT",
            QuantumGate::ControlledZ { .. } => "CZ",
            QuantumGate::Swap { .. } => "SWAP",
            QuantumGate::Toffoli { .. } => "CCX",
            QuantumGate::Measurement { .. } => "Measure",
        }
    }
}

/// Complex number wrapper for quantum amplitudes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

impl Complex {
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn one() -> Self {
        Self::new(1.0, 0.0)
    }

    pub fn i() -> Self {
        Self::new(0.0, 1.0)
    }

    /// Create a complex number in polar form: r * exp(i*theta)
    pub fn polar(r: f64, theta: f64) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }

    /// Magnitude of the complex number
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }

    /// Squared magnitude (probability)
    pub fn magnitude_squared(&self) -> f64 {
        self.real * self.real + self.imag * self.imag
    }

    /// Phase of the complex number
    pub fn phase(&self) -> f64 {
        self.imag.atan2(self.real)
    }

    /// Complex conjugate
    pub fn conjugate(&self) -> Self {
        Self::new(self.real, -self.imag)
    }
}

impl std::ops::Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.real + other.real, self.imag + other.imag)
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.real - other.real, self.imag - other.imag)
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self::new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real,
        )
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self::new(self.real * scalar, self.imag * scalar)
    }
}

impl std::ops::Mul<Complex> for f64 {
    type Output = Complex;

    fn mul(self, complex: Complex) -> Complex {
        complex * self
    }
}

impl Serialize for Complex {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Complex", 2)?;
        state.serialize_field("real", &self.real)?;
        state.serialize_field("imag", &self.imag)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Complex {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserialize, Deserializer, Visitor};
        use std::fmt;

        struct ComplexVisitor;

        impl<'de> Visitor<'de> for ComplexVisitor {
            type Value = Complex;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a complex number")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Complex, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut real = None;
                let mut imag = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "real" => real = Some(map.next_value()?),
                        "imag" => imag = Some(map.next_value()?),
                        _ => {}
                    }
                }

                Ok(Complex::new(
                    real.ok_or_else(|| de::Error::missing_field("real"))?,
                    imag.ok_or_else(|| de::Error::missing_field("imag"))?,
                ))
            }
        }

        deserializer.deserialize_struct("Complex", &["real", "imag"], ComplexVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_arithmetic() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, 4.0);

        let sum = a + b;
        assert_eq!(sum.real, 4.0);
        assert_eq!(sum.imag, 6.0);

        let diff = a - b;
        assert_eq!(diff.real, -2.0);
        assert_eq!(diff.imag, -2.0);

        let prod = a * b;
        assert_eq!(prod.real, -5.0);
        assert_eq!(prod.imag, 10.0);
    }

    #[test]
    fn test_complex_magnitude() {
        let z = Complex::new(3.0, 4.0);
        assert!((z.magnitude() - 5.0).abs() < 1e-10);
        assert!((z.magnitude_squared() - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_complex_polar() {
        let z = Complex::polar(1.0, PI / 2.0);
        assert!(z.real.abs() < 1e-10);
        assert!((z.imag - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_hadamard_gate() {
        let gate = QuantumGate::Hadamard;
        assert!(gate.is_single_qubit());
        assert_eq!(gate.name(), "H");

        let matrix = gate.single_qubit_matrix().unwrap();
        assert!(matrix[0][0].magnitude() > 0.0);
    }

    #[test]
    fn test_pauli_gates() {
        let x = QuantumGate::PauliX;
        let y = QuantumGate::PauliY;
        let z = QuantumGate::PauliZ;

        assert_eq!(x.name(), "X");
        assert_eq!(y.name(), "Y");
        assert_eq!(z.name(), "Z");
        assert!(x.is_single_qubit());
        assert!(y.is_single_qubit());
        assert!(z.is_single_qubit());
    }

    #[test]
    fn test_cnot_gate() {
        let cnot = QuantumGate::CNOT { control: 0, target: 1 };
        assert!(cnot.is_two_qubit());
        assert_eq!(cnot.name(), "CNOT");
    }

    #[test]
    fn test_toffoli_gate() {
        let toffoli = QuantumGate::Toffoli {
            control1: 0,
            control2: 1,
            target: 2,
        };
        assert!(toffoli.is_three_qubit());
        assert_eq!(toffoli.name(), "CCX");
    }

    #[test]
    fn test_rotation_gates() {
        let rx = QuantumGate::RotationX { theta: PI / 2.0 };
        let ry = QuantumGate::RotationY { theta: PI / 4.0 };
        let rz = QuantumGate::RotationZ { theta: PI };

        assert_eq!(rx.name(), "RX");
        assert_eq!(ry.name(), "RY");
        assert_eq!(rz.name(), "RZ");
        assert!(rx.is_single_qubit());
        assert!(ry.is_single_qubit());
        assert!(rz.is_single_qubit());
    }
}
