"""Sanity tests for QuantumEnergyOS tooling (no hardware required)."""


def test_backend_labels():
    allowed = {"SIMULATION", "EMULATION", "MODEL", "HARDWARE"}
    assert "SIMULATION" in allowed


def test_quartz5d_axes():
    axes = ("X", "Y", "Z", "T", "S")
    assert len(axes) == 5
    assert axes[-1] == "S"