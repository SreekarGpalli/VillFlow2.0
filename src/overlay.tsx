import React, { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import type { PillState, PillPayload } from "./types";
import "./index.css";

const FADE_TIMEOUT_MS: number = 1200;

function PillOverlay() {
  const [state, setState] = useState<PillState | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [fadingOut, setFadingOut] = useState(false);

  useEffect(() => {
    const unlistenPromise = listen<PillPayload>("pill-state", (event) => {
      setState(event.payload.state);
      setMessage(event.payload.message || null);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    if (state === "success" || state === "error") {
      const timer = setTimeout(() => {
        setFadingOut(true);
      }, FADE_TIMEOUT_MS);
      return () => clearTimeout(timer);
    } else {
      setFadingOut(false);
    }
  }, [state]);

  if (!state) return null;

  return (
    <div className="w-full h-screen flex items-center justify-center p-2 bg-transparent font-sans">
      <div
        className={`
          flex items-center gap-3 px-5 py-2.5 rounded-full
          bg-surface-alt/90 border border-border/80 shadow-2xl backdrop-blur-md
          transition-all duration-300 transform
          ${fadingOut ? "opacity-0 scale-95 duration-300" : "animate-zoom-in"}
        `}
        aria-live="polite"
      >
        {state === "recording" && (
          <>
            <span className="relative flex h-3 w-3">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-danger opacity-75"></span>
              <span className="relative inline-flex rounded-full h-3 w-3 bg-danger"></span>
            </span>
            <svg
              className="w-5 h-5 text-danger animate-pulse"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
              />
            </svg>
            <span className="text-sm font-semibold tracking-wide text-text-primary">
              Recording...
            </span>
          </>
        )}

        {state === "processing" && (
          <>
            <svg
              className="animate-spin h-5 w-5 text-secondary"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              ></circle>
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            <span className="text-sm font-semibold tracking-wide text-secondary">
              Processing...
            </span>
          </>
        )}

        {state === "success" && (
          <>
            <div className="flex items-center justify-center w-5 h-5 rounded-full bg-success/20 text-success">
              <svg
                className="w-[18px] h-[18px]"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2.5}
                  d="M5 13l4 4L19 7"
                />
              </svg>
            </div>
            <span className="text-sm font-semibold tracking-wide text-success">
              Done!
            </span>
          </>
        )}

        {state === "error" && (
          <>
            <div className="flex items-center justify-center w-5 h-5 rounded-full bg-danger/20 text-danger">
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2.5}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </div>
            <span className="text-sm font-semibold tracking-wide text-danger">
              {message || "Error"}
            </span>
          </>
        )}
      </div>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("pill-root") as HTMLElement).render(
  <React.StrictMode>
    <PillOverlay />
  </React.StrictMode>
);

