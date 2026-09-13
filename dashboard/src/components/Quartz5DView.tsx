import { useState } from 'react';

// 5D data -> 3D projection visualization.
// X/Y/Z -> geometry, T -> animation (timeline), S -> intensity/scale/state.
// This component never claims a 3D cube is literally 5D.

type Point5D = { x: number; y: number; z: number; t: number; state: number };

const SAMPLE: Point5D[] = [
  { x: 0, y: 0, z: 0, t: 0, state: 0.1 },
  { x: 1, y: 0, z: 0, t: 1, state: 0.5 },
  { x: 1, y: 1, z: 0, t: 2, state: 0.9 },
  { x: 0, y: 1, z: 1, t: 3, state: 0.4 },
];

export function Quartz5DView() {
  const [time, setTime] = useState(3);
  const [stateFilter, setStateFilter] = useState(0);

  const visible = SAMPLE.filter((p) => p.t <= time && p.state >= stateFilter);

  return (
    <section>
      <h2>Quartz5D (X,Y,Z,T,S) — 5D → 3D Projection</h2>
      <p>S is a computational state-space dimension, not a physical spatial dimension.</p>
      <label>
        Timeline T ≤ {time}
        <input type="range" min={0} max={3} value={time} onChange={(e) => setTime(Number(e.target.value))} />
      </label>
      <br />
      <label>
        State S ≥ {stateFilter.toFixed(2)}
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={stateFilter}
          onChange={(e) => setStateFilter(Number(e.target.value))}
        />
      </label>
      <ul>
        {visible.map((p, i) => (
          <li key={i}>
            ({p.x},{p.y},{p.z}) t={p.t} s={p.state} → intensity {(p.state * 100).toFixed(0)}% scale{' '}
            {(1 + p.state).toFixed(2)}
          </li>
        ))}
      </ul>
    </section>
  );
}