import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SettingsPage } from "./SettingsPage";
import { TimerPickerPage } from "./TimerPickerPage";
import { useI18n } from "./i18n";
import { RegionPickerPage } from "./RegionPickerPage";

function App() {
  const { t } = useI18n();
  const [permissionRequired, setPermissionRequired] = useState(false);
  const [stayActiveOn, setStayActiveOn] = useState(false);
  const [windowLabel, setWindowLabel] = useState<string>("main");

  useEffect(() => {
    setWindowLabel(getCurrentWindow().label);
  }, []);

  useEffect(() => {
    const unlistenPermissionRequired = listen<{ message?: string }>(
      "permission_required",
      () => setPermissionRequired(true)
    );
    const unlistenPermissionGranted = listen("permission_granted", () =>
      setPermissionRequired(false)
    );
    const unlistenStateChanged = listen<{ active: boolean }>(
      "stay_active_state_changed",
      (e) => setStayActiveOn(e.payload.active)
    );
    return () => {
      unlistenPermissionRequired.then((u) => u());
      unlistenPermissionGranted.then((u) => u());
      unlistenStateChanged.then((u) => u());
    };
  }, []);

  if (windowLabel === "settings") {
    return <SettingsPage />;
  }

  if (windowLabel === "timer-picker") {
    return <TimerPickerPage />;
  }

  if (windowLabel === "region-picker") {
    return <RegionPickerPage />;
  }

  if (permissionRequired) {
    return (
      <div className="permission-guidance">
        <h1>{t("permission.title")}</h1>
        <p>{t("permission.description")}</p>
        <ol>
          <li dangerouslySetInnerHTML={{ __html: t("permission.step1") }} />
          <li>{t("permission.step2")}</li>
          <li>{t("permission.step3")}</li>
        </ol>
        <p>
          <strong>{t("permission.firstTime")}</strong>{" "}
          <span dangerouslySetInnerHTML={{ __html: t("permission.firstTimeHint") }} />
        </p>
        <button
          type="button"
          onClick={() => invoke("open_system_preferences_accessibility")}
        >
          {t("permission.openSettings")}
        </button>
      </div>
    );
  }

  return (
    <div className="app">
      <p>{t("app.status", { status: stayActiveOn ? t("common.on") : t("common.off") })}</p>
      <p>{t("app.menuHint")}</p>
    </div>
  );
}

export default App;
