function DeviconLogo({ src, alt }: { src: string; alt: string }) {
  // eslint-disable-next-line @next/next/no-img-element
  return <img src={src} alt={alt} className="size-full object-contain" />
}

function CockroachDBLogo() {
  return (
    <svg viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
      <circle cx="64" cy="64" r="24" fill="#6933FF" />
      <g stroke="#6933FF" strokeWidth="7" strokeLinecap="round">
        <path d="M64 20v20M64 88v20M20 64h20M88 64h20" />
        <path d="M35 35l14 14M79 79l14 14M93 35L79 49M49 79l-14 14" />
      </g>
    </svg>
  )
}

function NeonLogo() {
  return (
    <svg viewBox="0 0 28 28" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M27.5421 0.00778666V28L16.7953 18.4918V27.8154H0V0L27.5421 0.00778666ZM3.3761 24.4393H13.4192V11.0836L24.1661 20.5916V3.38289L3.3761 3.37693V24.4393Z"
        fill="#34D59A"
      />
    </svg>
  )
}

export const ENGINE_LOGOS: { name: string; Logo: () => React.JSX.Element }[] = [
  { name: "PostgreSQL", Logo: () => <DeviconLogo src="/engine-logos/postgresql.svg" alt="PostgreSQL" /> },
  { name: "MySQL", Logo: () => <DeviconLogo src="/engine-logos/mysql.svg" alt="MySQL" /> },
  { name: "SQL Server", Logo: () => <DeviconLogo src="/engine-logos/sqlserver.svg" alt="SQL Server" /> },
  { name: "ClickHouse", Logo: () => <DeviconLogo src="/engine-logos/clickhouse.svg" alt="ClickHouse" /> },
  { name: "DuckDB", Logo: () => <DeviconLogo src="/engine-logos/duckdb.svg" alt="DuckDB" /> },
  { name: "MariaDB", Logo: () => <DeviconLogo src="/engine-logos/mariadb.svg" alt="MariaDB" /> },
  { name: "CockroachDB", Logo: CockroachDBLogo },
  { name: "SQLite", Logo: () => <DeviconLogo src="/engine-logos/sqlite.svg" alt="SQLite" /> },
  { name: "Neon", Logo: NeonLogo },
]
