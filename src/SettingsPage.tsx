import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "./i18n";
import type { MovementRegion } from "./types";
import { listen } from "@tauri-apps/api/event";
import { MOVEMENT_REGION_UI_ENABLED } from "./featureFlags";

export interface Settings {
  interval_min_sec: number;
  interval_max_sec: number;
  random_interval?: boolean;
  move_pixels_min?: number;
  move_pixels_max?: number;
  simulate_move?: boolean;
  movement_region?: MovementRegion;
  simulate_click?: boolean;
  click_button?: string;
  prevent_sleep?: boolean;
  language?: string;
}

type TabType = "general" | "advanced";

// Icon components
const GearIcon = ({ className }: { className?: string }) => (
  <svg className={className} width="16" height="16" viewBox="0 0 16 16" fill="none">
    <circle cx="8" cy="8" r="3" stroke="currentColor" strokeWidth="1.2" fill="none" />
    <path
      d="M8 1V3M8 13V15M15 8H13M3 8H1M13.364 2.636L11.95 4.05M4.05 11.95L2.636 13.364M13.364 13.364L11.95 11.95M4.05 4.05L2.636 2.636"
      stroke="currentColor"
      strokeWidth="1.2"
      strokeLinecap="round"
    />
  </svg>
);

const AdvancedIcon = ({ className }: { className?: string }) => (
  <svg className={className} width="16" height="16" viewBox="0 0 16 16" fill="none">
    <path
      d="M8 2L9.5 5.5L13.5 5.5L10.5 8L13.5 10.5L9.5 10.5L8 14L6.5 10.5L2.5 10.5L5.5 8L2.5 5.5L6.5 5.5L8 2Z"
      stroke="currentColor"
      strokeWidth="1.2"
      fill="none"
      strokeLinejoin="round"
    />
    <circle cx="8" cy="8" r="2" fill="currentColor" />
  </svg>
);

const ClockIcon = ({ className }: { className?: string }) => (
  <svg className={className} width="16" height="16" viewBox="0 0 16 16" fill="none">
    <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1.2" fill="none" />
    <path
      d="M8 4V8L10 10"
      stroke="currentColor"
      strokeWidth="1.2"
      strokeLinecap="round"
      strokeLinejoin="round"
    />
  </svg>
);

const MouseIcon = ({ className }: { className?: string }) => (
  <svg className={className} width="16" height="16" viewBox="0 0 16 16" fill="none">
    <rect x="5" y="2" width="6" height="10" rx="3" stroke="currentColor" strokeWidth="1.2" fill="none" />
    <path d="M8 2V0" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
    <path d="M8 12V14" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
    <circle cx="8" cy="7" r="1" fill="currentColor" />
  </svg>
);

const SystemIcon = ({ className }: { className?: string }) => (
  <svg className={className} width="16" height="16" viewBox="0 0 16 16" fill="none">
    <rect x="2" y="3" width="12" height="10" rx="1" stroke="currentColor" strokeWidth="1.2" fill="none" />
    <path d="M5 3V2C5 1.44772 5.44772 1 6 1H10C10.5523 1 11 1.44772 11 2V3" stroke="currentColor" strokeWidth="1.2" />
    <circle cx="8" cy="8" r="1.5" fill="currentColor" />
    <path d="M4 13H12" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
  </svg>
);

const LanguageIcon = ({ className }: { className?: string }) => (
  <svg className={className} width="16" height="16" viewBox="0 0 16 16" fill="none">
    <path
      d="M8 1C4.13401 1 1 4.13401 1 8C1 11.866 4.13401 15 8 15C11.866 15 15 11.866 15 8C15 4.13401 11.866 1 8 1Z"
      stroke="currentColor"
      strokeWidth="1.2"
      fill="none"
    />
    <path
      d="M5 8C5 9.65685 6.34315 11 8 11C9.65685 11 11 9.65685 11 8C11 6.34315 9.65685 5 8 5C6.34315 5 5 6.34315 5 8Z"
      stroke="currentColor"
      strokeWidth="1.2"
      fill="none"
    />
    <path d="M1 8H15M8 1V15" stroke="currentColor" strokeWidth="1.2" />
  </svg>
);

const InfoIcon = ({ className }: { className?: string }) => (
  <svg className={className} width="14" height="14" viewBox="0 0 14 14" fill="none">
    <circle cx="7" cy="7" r="6" stroke="currentColor" strokeWidth="1.2" fill="none" />
    <path d="M7 4V4.01" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    <path d="M7 6V10" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
  </svg>
);

const DEFAULTS: Settings = {
  interval_min_sec: 45,
  interval_max_sec: 90,
  random_interval: true,
  move_pixels_min: 1,
  move_pixels_max: 3,
  simulate_move: true,
  movement_region: { enabled: false },
  simulate_click: true,
  click_button: "left",
  prevent_sleep: true,
};

export function SettingsPage() {
  const { t, language, setLanguage } = useI18n();
  const [activeTab, setActiveTab] = useState<TabType>("general");
  const [settings, setSettings] = useState<Settings>(DEFAULTS);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => {
        const next = { ...DEFAULTS, ...s };
        // Region feature is WIP — keep UI/state on the default (disabled) path.
        if (!MOVEMENT_REGION_UI_ENABLED) {
          next.movement_region = { enabled: false };
        }
        setSettings(next);
        // Sync language if it's different
        if (s.language && (s.language === "en" || s.language === "zh") && s.language !== language) {
          setLanguage(s.language);
        }
      })
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (!MOVEMENT_REGION_UI_ENABLED) return;
    const unlisten = listen<MovementRegion>("movement_region_selected", (e) => {
      setSettings((s) => ({
        ...s,
        movement_region: { ...(s.movement_region ?? DEFAULTS.movement_region!), ...(e.payload ?? {}) },
      }));
    });
    const unlistenInvalid = listen<{ message?: string }>("movement_region_invalid", (e) => {
      setError(e.payload?.message ?? t("settings.movementRegionInvalid"));
    });
    return () => {
      unlisten.then((u) => u());
      unlistenInvalid.then((u) => u());
    };
  }, []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSaved(false);
    setSaving(true);
    invoke("set_settings", {
      s: {
        interval_min_sec: Number(settings.interval_min_sec),
        interval_max_sec:
          (settings.random_interval ?? DEFAULTS.random_interval)
            ? Number(settings.interval_max_sec)
            : Number(settings.interval_min_sec),
        random_interval: settings.random_interval ?? DEFAULTS.random_interval,
        move_pixels_min: Number(settings.move_pixels_min ?? DEFAULTS.move_pixels_min),
        move_pixels_max: Number(settings.move_pixels_max ?? DEFAULTS.move_pixels_max),
        simulate_move: settings.simulate_move ?? DEFAULTS.simulate_move,
        movement_region: MOVEMENT_REGION_UI_ENABLED
          ? settings.movement_region ?? DEFAULTS.movement_region
          : { enabled: false },
        simulate_click: settings.simulate_click ?? DEFAULTS.simulate_click,
        click_button: settings.click_button ?? DEFAULTS.click_button,
        prevent_sleep: settings.prevent_sleep ?? DEFAULTS.prevent_sleep,
        language: language,
      },
    })
      .then(() => {
        setSaved(true);
        setSaving(false);
        setTimeout(() => setSaved(false), 2000);
      })
      .catch((e) => {
        setError(String(e));
        setSaving(false);
      });
  };

  const renderGeneralTab = () => (
    <>
      <div className="preferences-section">
        <div className="section-header">
          <ClockIcon className="section-icon" />
          <label className="section-title">{t("settings.operationInterval")}</label>
        </div>
        <div className="section-row">
          <label className="checkbox-label">
            <input
              type="checkbox"
              className="mac-checkbox"
              checked={settings.random_interval ?? DEFAULTS.random_interval}
              onChange={(e) =>
                setSettings((s) => {
                  const nextRandom = e.target.checked;
                  return {
                    ...s,
                    random_interval: nextRandom,
                    interval_max_sec: nextRandom
                      ? (s.interval_max_sec ?? DEFAULTS.interval_max_sec)
                      : (s.interval_min_sec ?? DEFAULTS.interval_min_sec),
                  };
                })
              }
            />
            <span>{t("settings.randomInterval")}</span>
          </label>
        </div>
        <div className="section-row">
          <label className="field-label">
            <span>{t("settings.minimumInterval")}</span>
            <input
              type="number"
              className="number-input"
              min={1}
              max={600}
              value={settings.interval_min_sec}
              onChange={(e) =>
                setSettings((s) => {
                  const nextMin = Number(e.target.value) || 1;
                  const random = s.random_interval ?? DEFAULTS.random_interval;
                  return {
                    ...s,
                    interval_min_sec: nextMin,
                    interval_max_sec: random
                      ? Math.max(Number(s.interval_max_sec ?? DEFAULTS.interval_max_sec) || 1, nextMin)
                      : nextMin,
                  };
                })
              }
            />
            <span className="field-unit">{t("settings.seconds")}</span>
          </label>
        </div>
        {(settings.random_interval ?? DEFAULTS.random_interval) && (
          <div className="section-row">
            <label className="field-label">
              <span>{t("settings.maximumInterval")}</span>
              <input
                type="number"
                className="number-input"
                min={1}
                max={600}
                value={settings.interval_max_sec}
                onChange={(e) =>
                  setSettings((s) => {
                    const nextMax = Number(e.target.value) || 1;
                    const min = Number(s.interval_min_sec) || 1;
                    return { ...s, interval_max_sec: Math.max(nextMax, min) };
                  })
                }
              />
              <span className="field-unit">{t("settings.seconds")}</span>
            </label>
          </div>
        )}
        <div className="section-hint">
          <InfoIcon className="hint-icon" />
          <span>{t("settings.operationIntervalHint")}</span>
        </div>
      </div>

      <div className="preferences-separator"></div>

      {/* Click */}
      <div className="preferences-section">
        <div className="section-header">
          <MouseIcon className="section-icon" />
          <label className="section-title">{t("settings.inputSimulation")}</label>
        </div>

        <div className="section-row">
          <label className="checkbox-label">
            <input
              type="checkbox"
              className="mac-checkbox"
              checked={settings.simulate_click ?? DEFAULTS.simulate_click}
              onChange={(e) => setSettings((s) => ({ ...s, simulate_click: e.target.checked }))}
            />
            <span>{t("settings.simulateClick")}</span>
          </label>
        </div>

        {(settings.simulate_click ?? DEFAULTS.simulate_click) && (
          <div className="section-row">
            <label className="field-label">
              <span>{t("settings.clickButton")}</span>
              <div className="segmented-control">
                <button
                  type="button"
                  className={`segmented-option ${(settings.click_button ?? DEFAULTS.click_button) === "left" ? "active" : ""}`}
                  onClick={() => setSettings((s) => ({ ...s, click_button: "left" }))}
                >
                  {t("settings.leftClick")}
                </button>
                <button
                  type="button"
                  className={`segmented-option ${(settings.click_button ?? DEFAULTS.click_button) === "right" ? "active" : ""}`}
                  onClick={() => setSettings((s) => ({ ...s, click_button: "right" }))}
                >
                  {t("settings.rightClick")}
                </button>
              </div>
            </label>
          </div>
        )}

        <div className="section-hint">
          <InfoIcon className="hint-icon" />
          <span>{t("settings.simulationHint")}</span>
        </div>
      </div>

      <div className="preferences-separator"></div>

      {/* Movement */}
      <div className="preferences-section">
        <div className="section-header">
          <MouseIcon className="section-icon" />
          <label className="section-title">{t("settings.movementRange")}</label>
        </div>
        <div className="section-row">
          <label className="checkbox-label">
            <input
              type="checkbox"
              className="mac-checkbox"
              checked={settings.simulate_move ?? DEFAULTS.simulate_move}
              onChange={(e) => setSettings((s) => ({ ...s, simulate_move: e.target.checked }))}
            />
            <span>{t("settings.simulateMove")}</span>
          </label>
        </div>
        {/* Movement region UI is temporarily hidden (WIP). Flip MOVEMENT_REGION_UI_ENABLED to restore. */}
        {MOVEMENT_REGION_UI_ENABLED && (
          <>
            <div className="section-row">
              <label className="checkbox-label">
                <input
                  type="checkbox"
                  className="mac-checkbox"
                  checked={settings.movement_region?.enabled ?? DEFAULTS.movement_region?.enabled}
                  onChange={(e) =>
                    setSettings((s) => ({
                      ...s,
                      movement_region: {
                        ...(s.movement_region ?? DEFAULTS.movement_region!),
                        enabled: e.target.checked,
                      },
                    }))
                  }
                  disabled={!(settings.simulate_move ?? DEFAULTS.simulate_move)}
                />
                <span>{t("settings.useMovementRegion")}</span>
              </label>
            </div>

            {(() => {
              const region = settings.movement_region ?? DEFAULTS.movement_region!;
              const enabled = region.enabled;
              const configured =
                region.x_min != null &&
                region.y_min != null &&
                region.x_max != null &&
                region.y_max != null;
              if (!enabled) return null;
              return (
                <div className="section-row">
                  <label className="field-label">
                    <span>{t("settings.movementRegion")}</span>
                    <div className="segmented-control">
                      <button
                        type="button"
                        className="mac-button secondary"
                        onClick={() => invoke("start_region_selection")}
                      >
                        {configured ? t("settings.reselectRegion") : t("settings.selectRegion")}
                      </button>
                      {configured && (
                        <button
                          type="button"
                          className="mac-button secondary"
                          onClick={() => invoke("clear_movement_region")}
                        >
                          {t("settings.clearRegion")}
                        </button>
                      )}
                    </div>
                    {configured && (
                      <div className="section-hint" style={{ marginTop: 8 }}>
                        <InfoIcon className="hint-icon" />
                        <span>
                          {t("settings.regionSummary", {
                            x: String(region.x_min),
                            y: String(region.y_min),
                            width: String(Number(region.x_max) - Number(region.x_min)),
                            height: String(Number(region.y_max) - Number(region.y_min)),
                          })}
                        </span>
                      </div>
                    )}
                  </label>
                </div>
              );
            })()}
          </>
        )}

        {!(settings.simulate_move ?? DEFAULTS.simulate_move) ? (
          <div className="section-hint">
            <InfoIcon className="hint-icon" />
            <span>{t("settings.movementDisabledHint")}</span>
          </div>
        ) : MOVEMENT_REGION_UI_ENABLED &&
          (settings.movement_region?.enabled ?? DEFAULTS.movement_region?.enabled) ? (
          <div className="section-hint">
            <InfoIcon className="hint-icon" />
            <span>{t("settings.regionEnabledDisablesRangeHint")}</span>
          </div>
        ) : (
          <>
            <div className="section-row">
              <label className="field-label">
                <span>{t("settings.minimumPixels")}</span>
                <input
                  type="number"
                  className="number-input"
                  min={0}
                  max={20}
                  value={settings.move_pixels_min ?? DEFAULTS.move_pixels_min}
                  onChange={(e) =>
                    setSettings((s) => ({ ...s, move_pixels_min: Number(e.target.value) || 0 }))
                  }
                />
                <span className="field-unit">px</span>
              </label>
            </div>
            <div className="section-row">
              <label className="field-label">
                <span>{t("settings.maximumPixels")}</span>
                <input
                  type="number"
                  className="number-input"
                  min={0}
                  max={20}
                  value={settings.move_pixels_max ?? DEFAULTS.move_pixels_max}
                  onChange={(e) =>
                    setSettings((s) => ({ ...s, move_pixels_max: Number(e.target.value) || 0 }))
                  }
                />
                <span className="field-unit">px</span>
              </label>
            </div>
            <div className="section-hint">
              <InfoIcon className="hint-icon" />
              <span>{t("settings.rangeHint")}</span>
            </div>
          </>
        )}
      </div>
    </>
  );

  const renderAdvancedTab = () => (
    <>
      <div className="preferences-section">
        <div className="section-header">
          <SystemIcon className="section-icon" />
          <label className="section-title">{t("settings.systemBehavior")}</label>
        </div>
        <div className="section-row">
          <label className="checkbox-label">
            <input
              type="checkbox"
              className="mac-checkbox"
              checked={settings.prevent_sleep ?? DEFAULTS.prevent_sleep}
              onChange={(e) => setSettings((s) => ({ ...s, prevent_sleep: e.target.checked }))}
            />
            <span>{t("settings.preventSleep")}</span>
          </label>
        </div>
      </div>

      <div className="preferences-separator"></div>

      <div className="preferences-section">
        <div className="section-header">
          <LanguageIcon className="section-icon" />
          <label className="section-title">{t("settings.language")}</label>
        </div>
        <div className="section-row">
          <div className="language-selector">
            <button
              type="button"
              className={`language-option ${language === "en" ? "active" : ""}`}
              onClick={() => setLanguage("en")}
            >
              {t("settings.english")}
            </button>
            <button
              type="button"
              className={`language-option ${language === "zh" ? "active" : ""}`}
              onClick={() => setLanguage("zh")}
            >
              {t("settings.chinese")}
            </button>
          </div>
        </div>
      </div>
    </>
  );

  return (
    <div className="preferences-window">
      <div className="preferences-tabs">
        <div
          className={`preferences-tab ${activeTab === "general" ? "active" : ""}`}
          onClick={() => setActiveTab("general")}
        >
          <GearIcon className="tab-icon" />
          <span>{t("settings.general")}</span>
        </div>
        <div
          className={`preferences-tab ${activeTab === "advanced" ? "active" : ""}`}
          onClick={() => setActiveTab("advanced")}
        >
          <AdvancedIcon className="tab-icon" />
          <span>{t("settings.advanced")}</span>
        </div>
      </div>

      <div className="preferences-content-wrapper">
        <div className="preferences-content">
          {activeTab === "general" && renderGeneralTab()}
          {activeTab === "advanced" && renderAdvancedTab()}

          {error && (
            <div className="preferences-message error">
              {error}
            </div>
          )}
          {saved && (
            <div className="preferences-message success">
              {t("settings.saved")}
            </div>
          )}
        </div>

        <div className="preferences-actions">
          <button
            type="button"
            className="mac-button secondary"
            onClick={() => {
              invoke<Settings>("get_settings")
                .then((s) => setSettings({ ...DEFAULTS, ...s }))
                .catch((e) => setError(String(e)));
            }}
          >
            {t("common.reset")}
          </button>
          <button
            type="button"
            className="mac-button primary"
            onClick={handleSubmit}
            disabled={saving}
          >
            {saving ? t("common.saving") : t("common.save")}
          </button>
        </div>
      </div>
    </div>
  );
}

