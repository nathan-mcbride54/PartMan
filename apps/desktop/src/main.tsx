import React from "react";
import ReactDOM from "react-dom/client";

import "../../../packages/design-tokens/src/generated.css";
import "../../../packages/ui/src/ui.css";
import "./app.css";
import { App } from "./App";

const root = document.getElementById("root");
if (!root) {
  throw new Error("PartMan cannot start because the root element is missing");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
