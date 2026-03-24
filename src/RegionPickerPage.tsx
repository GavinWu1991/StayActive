import { useEffect, useMemo, useRef, useState } from "react";
import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "./i18n";
import { emit } from "@tauri-apps/api/event";

type DragState =
  | { kind: "idle" }
  | { kind: "dragging"; startX: number; startY: number; currentX: number; currentY: number };

function normalizeRect(a: { x: number; y: number }, b: { x: number; y: number }) {
  const xMin = Math.min(a.x, b.x);
  const yMin = Math.min(a.y, b.y);
  const xMax = Math.max(a.x, b.x);
  const yMax = Math.max(a.y, b.y);
  return { xMin, yMin, xMax, yMax, width: xMax - xMin, height: yMax - yMin };
}

export function RegionPickerPage() {
  const { t } = useI18n();
  const win = useMemo(() => getCurrentWindow(), []);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [drag, setDrag] = useState<DragState>({ kind: "idle" });
  const [error, setError] = useState<string | null>(null);

  const rect = useMemo(() => {
    if (drag.kind !== "dragging") return null;
    return normalizeRect({ x: drag.startX, y: drag.startY }, { x: drag.currentX, y: drag.currentY });
  }, [drag]);

  useEffect(() => {
    const onKeyDown = async (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        await emit("movement_region_selection_cancelled", {});
        await win.close();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [win]);

  const toClientPoint = (e: React.MouseEvent) => {
    const el = containerRef.current;
    if (!el) return { x: e.clientX, y: e.clientY };
    const r = el.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  const onMouseDown = (e: React.MouseEvent) => {
    setError(null);
    const p = toClientPoint(e);
    setDrag({ kind: "dragging", startX: p.x, startY: p.y, currentX: p.x, currentY: p.y });
  };

  const onMouseMove = (e: React.MouseEvent) => {
    if (drag.kind !== "dragging") return;
    const p = toClientPoint(e);
    setDrag((s) => (s.kind === "dragging" ? { ...s, currentX: p.x, currentY: p.y } : s));
  };

  const onMouseUp = async (e: React.MouseEvent) => {
    if (drag.kind !== "dragging") return;
    const p = toClientPoint(e);
    const r = normalizeRect({ x: drag.startX, y: drag.startY }, { x: p.x, y: p.y });

    // Minimum rectangle size to avoid accidental clicks.
    if (r.width < 10 || r.height < 10) {
      setError(t("regionPicker.tooSmall"));
      setDrag({ kind: "idle" });
      return;
    }

    try {
      const scale = window.devicePixelRatio || 1;
      const payload: InvokeArgs = {
        enabled: true,
        display_ref: "unknown",
        x_min: r.xMin * scale,
        y_min: r.yMin * scale,
        x_max: r.xMax * scale,
        y_max: r.yMax * scale,
      };
      await invoke("set_movement_region", payload);
      await win.close();
    } catch (err) {
      setError(String(err));
      setDrag({ kind: "idle" });
    }
  };

  return (
    <div className="region-picker-root">
      <div className="region-picker-instructions">
        <div className="region-picker-title">{t("regionPicker.title")}</div>
        <div className="region-picker-subtitle">{t("regionPicker.subtitle")}</div>
        <div className="region-picker-actions">
          <button
            type="button"
            className="mac-button secondary"
            onClick={async () => {
              await emit("movement_region_selection_cancelled", {});
              await win.close();
            }}
          >
            {t("common.cancel")}
          </button>
        </div>
      </div>

      <div
        ref={containerRef}
        className="region-picker-overlay"
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onMouseUp}
      >
        {rect && (
          <div
            className="region-picker-rect"
            style={{
              left: rect.xMin,
              top: rect.yMin,
              width: rect.width,
              height: rect.height,
            }}
          />
        )}
      </div>

      {error && <div className="preferences-message error region-picker-error">{error}</div>}
    </div>
  );
}

