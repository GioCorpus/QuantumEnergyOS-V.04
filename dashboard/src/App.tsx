import { useState } from 'react';
import { Quartz5DView } from './components/Quartz5DView';
import { ServiceHealth } from './components/ServiceHealth';
import { EnergyPanel } from './components/EnergyPanel';

type Tab = 'overview' | 'quantum' | 'energy' | 'quartz5d';

export default function App() {
  const [tab, setTab] = useState<Tab>('overview');

  return (
    <div style={{ fontFamily: 'system-ui, sans-serif', padding: 24, maxWidth: 1100, margin: '0 auto' }}>
      <header>
        <h1>QuantumEnergyOS Dashboard</h1>
        <p>
          Classical + Quantum-runtime + Energy telemetry. Quantum backends are explicitly labeled
          SIMULATION / EMULATION / REMOTE / PHYSICAL. No hardware control is claimed without a
          documented interface.
        </p>
        <nav style={{ display: 'flex', gap: 8 }}>
          {(['overview', 'quantum', 'energy', 'quartz5d'] as Tab[]).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              style={{ fontWeight: tab === t ? 700 : 400, padding: '6px 12px' }}
            >
              {t}
            </button>
          ))}
        </nav>
      </header>
      <main style={{ marginTop: 16 }}>
        {tab === 'overview' && <ServiceHealth />}
        {tab === 'quantum' && (
          <section>
            <h2>Quantum Runtime (SIMULATION)</h2>
            <p>
              Reference backend: state-vector simulator. Majorana / physical adapters are
              capability-gated stubs until a documented hardware API exists.
            </p>
          </section>
        )}
        {tab === 'energy' && <EnergyPanel />}
        {tab === 'quartz5d' && <Quartz5DView />}
      </main>
    </div>
  );
}