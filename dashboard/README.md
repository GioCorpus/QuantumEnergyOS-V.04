# QuantumEnergyOS Dashboard (V.04 §19)

Vite + React + TypeScript UI. Requires `index.html` entry, explicit
`@vitejs/plugin-react`, and pinned `packageManager: pnpm@9.12.0`.

- `pnpm install --frozen-lockfile`
- `pnpm build` runs `tsc --noEmit && vite build`
- Compatible surfaces: Grafana, Prometheus, JupyterLab, Supabase Studio, plus this dashboard.
- Quartz5D view: 5D → 3D projection with timeline + state slider.