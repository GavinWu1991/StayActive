import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "./i18n";

export function TimerPickerPage() {
  const { t } = useI18n();
  const now = new Date();
  const [hour, setHour] = useState(now.getHours());
  const [minute, setMinute] = useState(now.getMinutes());
  const [error, setError] = useState<string | null>(null);

  async function handleOk() {
    setError(null);
    try {
      await invoke("set_timer_custom", { hour, minute });
      getCurrentWindow().close();
    } catch (e) {
      setError(String(e));
    }
  }

  function handleCancel() {
    getCurrentWindow().close();
  }

  return (
    <div className="timer-picker-page">
      <h2>{t("timer.title")}</h2>
      <div className="timer-picker-form">
        <div className="timer-picker-row">
          <label>{t("timer.hour")}</label>
          <input
            type="number"
            className="number-input"
            min={0}
            max={23}
            value={hour}
            onChange={(e) => setHour(Number(e.target.value) || 0)}
          />
          <span className="field-unit">:</span>
          <label>{t("timer.minute")}</label>
          <input
            type="number"
            className="number-input"
            min={0}
            max={59}
            value={minute}
            onChange={(e) => setMinute(Number(e.target.value) || 0)}
          />
        </div>
        {error && (
          <div className="preferences-message error">
            {error}
          </div>
        )}
        <div className="timer-picker-actions">
          <button type="button" className="mac-button secondary" onClick={handleCancel}>
            {t("common.cancel")}
          </button>
          <button type="button" className="mac-button primary" onClick={handleOk}>
            {t("common.ok")}
          </button>
        </div>
      </div>
    </div>
  );
}
