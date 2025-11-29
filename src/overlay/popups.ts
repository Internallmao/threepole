import { appWindow } from "@tauri-apps/api/window";
import "./popups.css";

export type Popup = {
    title: string,
    subtext: string,
};

let popupPanel = document.querySelector<HTMLElement>("#popup-panel");
if (!popupPanel) {
    popupPanel = document.createElement("div");
    popupPanel.id = "popup-panel";
    popupPanel.style.cssText = `
        position: fixed !important;
        bottom: 16px !important;
        right: 16px !important;
        width: 300px;
        max-height: 80vh;
        display: flex;
        flex-direction: column;
        justify-content: flex-end;
        pointer-events: none;
        z-index: 999999 !important;
    `;
    document.body.appendChild(popupPanel);
}

let queuedPopups: Popup[] = [];

function showPopup(queuedPopup: Popup) {
    console.log("Creating popup element for:", queuedPopup);
    
    let popup = document.createElement("div");
    popup.classList.add("popup");

    if (customBackgroundColor) {
        popup.style.background = customBackgroundColor;
        popup.style.setProperty('--notification-background-color', customBackgroundColor);
    }

    let contents = document.createElement("div");

    let title = document.createElement("p");
    title.classList.add("title");
    title.innerText = queuedPopup.title;

    let subtext = document.createElement("p");
    subtext.innerHTML = queuedPopup.subtext;

    contents.appendChild(title);
    contents.appendChild(subtext);

    popup.appendChild(contents);

    console.log("Appending popup to panel:", popupPanel);
    popupPanel.appendChild(popup);

    setTimeout(() => {
        popup.classList.add("fade-out");
    }, 7600);

    setTimeout(() => {
        if (popupPanel.contains(popup)) {
            popupPanel.removeChild(popup);
        }
    }, 8000);
}

function showQueuedPopups() {
    console.log("Showing queued popups:", queuedPopups.length);
    for (let queuedPopup of queuedPopups) {
        showPopup(queuedPopup);
    }
    queuedPopups = [];
}

appWindow.listen("show", showQueuedPopups);

let customBackgroundColor: string | null = null;

export function createPopup(popup: Popup, showImmediately: boolean, backgroundColor?: string) {
    console.log("createPopup called with:", popup, showImmediately, backgroundColor);
    
    if (backgroundColor) {
        customBackgroundColor = backgroundColor;
    }
    
    showPopup(popup);
}
