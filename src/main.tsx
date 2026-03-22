import React, { Suspense } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./components/ThemeProvider";
import { initI18n } from "./i18n";
import "./index.css";

initI18n().then(() => {
    ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
        <React.StrictMode>
            <Suspense>
                <ThemeProvider defaultTheme="system" storageKey="appthere-loki-theme">
                    <App />
                </ThemeProvider>
            </Suspense>
        </React.StrictMode>
    );
});
