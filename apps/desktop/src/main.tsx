import ReactDOM from "react-dom/client";
import App from "@/src/app/App";
import { DetachedTabApp } from "@/src/app/DetachedTabApp";
import { readDetachedTabFromUrl } from "@/src/app/detachedWindow";

const detachedTab = readDetachedTabFromUrl();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  detachedTab ? <DetachedTabApp tab={detachedTab} /> : <App />
);
