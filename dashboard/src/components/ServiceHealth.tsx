export function ServiceHealth() {
  const services = [
    'identity',
    'policy',
    'quantum',
    'qpu (SIMULATION)',
    'telemetry',
    'energy',
    'browser',
    'dashboard',
    'storage',
    'hardware',
  ];
  return (
    <section>
      <h2>Services</h2>
      <ul>
        {services.map((s) => (
          <li key={s}>
            {s}: <code>unknown (connect API for live health)</code>
          </li>
        ))}
      </ul>
    </section>
  );
}