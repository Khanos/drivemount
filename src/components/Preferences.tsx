import { Gauge, HardDrive, SlidersHorizontal, Zap } from "lucide-react";
import type { Preferences } from "../lib/types";
import { presets } from "../lib/store";
import { Notice, PreferenceToggle } from "./common";
export function CachePresetSelector({
  value,
  onChange,
}: {
  value: Preferences["cache"];
  onChange: (cache: Preferences["cache"]) => void;
}) {
  const selected =
    Object.entries(presets).find(
      ([, v]) =>
        v.mode === value.mode &&
        v.sizeGib === value.sizeGib &&
        v.ageHours === value.ageHours,
    )?.[0] ?? "Custom";
  return (
    <>
      <div className="preset-grid">
        {Object.entries(presets).map(([name, preset], i) => (
          <button
            type="button"
            aria-pressed={selected === name}
            className={`preset ${selected === name ? "selected" : ""}`}
            key={name}
            onClick={() => onChange({ ...preset })}
          >
            {i === 0 ? (
              <HardDrive size={20} />
            ) : i === 1 ? (
              <Gauge size={20} />
            ) : (
              <Zap size={20} />
            )}
            <strong>{name}</strong>
            <span>
              {name === "Balanced"
                ? "Recommended"
                : `${preset.sizeGib} GiB temporary storage`}
            </span>
          </button>
        ))}
      </div>
      <p className="help">
        Drive temporarily stores parts of files on your computer to make them
        easier to open and edit.
      </p>
      <details className="advanced">
        <summary>
          <SlidersHorizontal size={16} />
          Advanced performance settings <span>{selected}</span>
        </summary>
        <div className="advanced-fields">
          <label>
            Temporary storage limit{" "}
            <span className="input-unit">
              <input
                aria-label="Temporary storage limit in GiB"
                type="number"
                min={1}
                max={1024}
                value={value.sizeGib}
                onChange={(e) =>
                  onChange({ ...value, sizeGib: Number(e.target.value) })
                }
              />
              <span>GiB</span>
            </span>
          </label>
          <label>
            Keep unused cached files for{" "}
            <span className="input-unit">
              <input
                aria-label="Cache age in hours"
                type="number"
                min={1}
                max={8760}
                value={value.ageHours}
                onChange={(e) =>
                  onChange({ ...value, ageHours: Number(e.target.value) })
                }
              />
              <span>hours</span>
            </span>
          </label>
          <label>
            File buffering
            <select
              value={value.mode}
              onChange={(e) =>
                onChange({
                  ...value,
                  mode: e.target.value as Preferences["cache"]["mode"],
                })
              }
            >
              <option value="full">Reads and writes</option>
              <option value="writes">Writes only</option>
              <option value="minimal">Minimal buffering</option>
            </select>
          </label>
        </div>
        <p className="help">
          The storage limit is a cleanup target, not a strict cap. Open files
          and pending uploads can temporarily use more space. Cached files are
          not a complete offline backup.
        </p>
      </details>
    </>
  );
}
export function ConnectionPreferences({
  value,
  onChange,
  freeBytes,
}: {
  value: Preferences;
  onChange: (v: Preferences) => void;
  freeBytes: number;
}) {
  return (
    <>
      <PreferenceToggle
        id="autostart"
        title="Connect when I sign in"
        description="Your Drive will connect when your desktop session starts."
        checked={value.autostart}
        onChange={(autostart) => onChange({ ...value, autostart })}
      />
      <div className="field-section">
        <h3>File access</h3>
        <div className="access-options">
          <label>
            <input
              type="radio"
              name="access"
              checked={!value.readOnly}
              onChange={() => onChange({ ...value, readOnly: false })}
            />
            <span>
              <strong>Read and write</strong>
              <small>Open, edit, and organize your files</small>
            </span>
          </label>
          <label>
            <input
              type="radio"
              name="access"
              checked={value.readOnly}
              onChange={() => onChange({ ...value, readOnly: true })}
            />
            <span>
              <strong>Read-only</strong>
              <small>Browse files without changing Drive</small>
            </span>
          </label>
        </div>
        {!value.readOnly && (
          <p className="help">
            Editing or deleting files in this folder can also change or delete
            files in Google Drive.
          </p>
        )}
      </div>
      <div className="field-section">
        <h3>Performance</h3>
        <CachePresetSelector
          value={value.cache}
          onChange={(cache) => onChange({ ...value, cache })}
        />
      </div>
      {freeBytes < 2 * 1024 ** 3 && (
        <Notice tone="warning">
          Your cache disk has less than 2 GiB free. Free some space before
          connecting Drive.
        </Notice>
      )}
    </>
  );
}
