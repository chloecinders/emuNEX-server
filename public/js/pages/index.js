import { css, html, LitElement } from "lit";
import { Code, createIcons, Download } from "lucide";
import "../components/navbar.js";
import { authTokens, baseTokens, buttonStyles } from "../components/shared-styles.js";
import { ThemeManager } from "../theme-manager.js";

export class EmunexIndexPage extends LitElement {
    static properties = {
        _theme: { type: String, state: true },
    };

    constructor() {
        super();
        this._theme = ThemeManager.current;
    }

    connectedCallback() {
        super.connectedCallback();
        window.addEventListener("theme-changed", this._handleThemeChange);
    }

    disconnectedCallback() {
        super.disconnectedCallback();
        window.removeEventListener("theme-changed", this._handleThemeChange);
    }

    _handleThemeChange = (e) => {
        this._theme = e.detail.theme;
    };

    static styles = [
        baseTokens,
        authTokens,
        buttonStyles,
        css`
            :host {
                display: flex;
                flex-direction: column;
                min-height: 100vh;
                width: 100%;
                font-family:
                    "Industry",
                    system-ui,
                    -apple-system,
                    BlinkMacSystemFont,
                    "Segoe UI",
                    sans-serif;
                color: var(--color-text);
                box-sizing: border-box;
            }

            *,
            *::before,
            *::after {
                box-sizing: inherit;
            }

            .main-content {
                flex: 1;
                display: flex;
                align-items: center;
                justify-content: center;
                padding: var(--spacing-xl);
                margin-top: 56px;
            }

            .hero-layout {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: var(--spacing-xl);
                max-width: 1000px;
                width: 100%;
                align-items: center;
            }

            @media (max-width: 768px) {
                .hero-layout {
                    grid-template-columns: 1fr;
                    text-align: center;
                }
            }

            .hero-image-container {
                width: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
            }

            .hero-image-container img {
                width: 100%;
                height: auto;
                box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
            }

            .hero-text h1 {
                font-size: 4rem;
                font-weight: 900;
                margin: 0;
                letter-spacing: -2px;
                color: var(--color-primary);
            }

            .hero-text p {
                font-size: 1.1rem;
                color: var(--color-text-muted);
                margin: var(--spacing-sm) 0 var(--spacing-xl) 0;
            }

            .hero-buttons {
                display: flex;
                flex-direction: column;
                gap: var(--spacing-md);
            }

            @media (max-width: 768px) {
                .hero-buttons {
                    margin: 0 auto;
                    max-width: 300px;
                }
            }

            .btn-front-flex {
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 8px;
                width: 100%;
            }
            .lucide {
                width: 18px;
                height: 18px;
                flex-shrink: 0;
            }
            .popout-btn.btn-secondary {
                --btn-color-primary: var(--color-surface, #ffffff);
                --btn-color-dark: var(--color-border-hover, #d0d0e0);
            }
            :host-context([data-theme="dark"]) .popout-btn.btn-secondary {
                --btn-color-primary: #4a4a5a;
                --btn-color-dark: #2b2b36;
            }
            .popout-btn.btn-secondary .btn-front {
                color: var(--color-text, #2d2d3a);
                text-shadow: none;
                border: 1px solid var(--color-border, #e2e2ec);
            }
            :host-context([data-theme="dark"]) .popout-btn.btn-secondary .btn-front {
                color: #ffffff;
                border-color: #5b5b6a;
            }
        `,
    ];

    firstUpdated() {
        createIcons({
            icons: {
                Download,
                Code,
            },
            nameAttr: "data-lucide",
            root: this.shadowRoot,
        });
    }

    render() {
        return html`
            <emunex-navbar></emunex-navbar>

            <main class="main-content">
                <div class="hero-layout">
                    <div class="hero-image-container">
                        <img
                            src="/public/images/emuNEX-${this._theme === "dark" ? "dark" : "light"}.png"
                            alt="emuNEX Preview"
                        />
                    </div>

                    <div class="hero-text">
                        <h1>emuNEX</h1>
                        <p>Remote Emulation & Library Management</p>

                        <div class="hero-buttons">
                            <a
                                href="https://github.com/chloecinders/emuNEX-client/releases"
                                class="popout-btn"
                                target="_blank"
                            >
                                <span class="btn-edge"></span>
                                <span class="btn-front btn-front-flex">
                                    <i data-lucide="download"></i>
                                    Download Client
                                </span>
                            </a>

                            <a
                                href="https://github.com/chloecinders/emuNEX-client"
                                class="popout-btn btn-secondary"
                                target="_blank"
                            >
                                <span class="btn-edge"></span>
                                <span class="btn-front btn-front-flex">
                                    <i data-lucide="code"></i>
                                    Client Repository
                                </span>
                            </a>

                            <a
                                href="https://github.com/chloecinders/emuNEX-server"
                                class="popout-btn btn-secondary"
                                target="_blank"
                            >
                                <span class="btn-edge"></span>
                                <span class="btn-front btn-front-flex">
                                    <i data-lucide="code"></i>
                                    Server Repository
                                </span>
                            </a>
                        </div>
                    </div>
                </div>
            </main>
        `;
    }
}

customElements.define("emunex-index-page", EmunexIndexPage);
