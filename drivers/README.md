# Drivers (V.04 §12–§13)

Thin, documented kernel/userland drivers bound to `hardware/` traits.
Prefer upstream Linux drivers; custom modules require a documented reason
(performance, telemetry, power, QPU adapter).

No reverse-engineered proprietary protocols. QPU drivers are adapters over
documented APIs only.