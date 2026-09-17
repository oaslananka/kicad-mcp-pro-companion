export default function SettingsScreen() {
  return (
    <div>
      <h2>Settings</h2>
      <div className="card">
        <p>
          Companion telemetry is <strong>off by default</strong> and nothing in this build sends project files,
          schematics, board contents, or audit data anywhere. See the README's Privacy section.
        </p>
        <p className="mono">
          Configuration (data directory, log level, core bridge endpoint, transport mode) is set via CLI
          flags/environment variables/config file — see docs/protocol and the top-level README. A settings editor
          UI is planned but not implemented in this build.
        </p>
      </div>
    </div>
  );
}
