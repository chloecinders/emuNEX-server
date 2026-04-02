import { css, html, LitElement } from "lit";
import { createIcons, Home, LogIn, Moon, Save, Settings, Shield, Sun, User } from "lucide";
import { ThemeManager } from "../theme-manager.js";
import { baseTokens } from "./shared-styles.js";

export class EmunexNavbar extends LitElement {
    static properties = {
        isLoggedIn: { type: Boolean, state: true },
        isAdmin: { type: Boolean, state: true },
        _theme: { type: String, state: true },
    };

    static styles = [
        baseTokens,
        css`
            :host {
                display: block;
                width: 100%;
                position: absolute;
                top: 0;
                left: 0;
                z-index: 1000;
                font-family:
                    "Industry",
                    system-ui,
                    -apple-system,
                    BlinkMacSystemFont,
                    "Segoe UI",
                    sans-serif;
            }
            .status-bar {
                background: var(--glass-bg, rgba(255, 255, 255, 0.8));
                backdrop-filter: blur(12px);
                -webkit-backdrop-filter: blur(12px);
                padding: var(--spacing-sm) var(--spacing-md);
                display: flex;
                justify-content: space-between;
                align-items: center;
                border-bottom: 2px solid var(--color-border);
                min-height: 56px;
                box-sizing: border-box;
                transition:
                    background 0.25s ease,
                    border-color 0.25s ease;
            }
            .logo {
                display: flex;
                align-items: center;
                font-weight: 900;
                font-size: 1.25rem;
                color: var(--color-primary, #6b5cb1);
                text-decoration: none;
                letter-spacing: -0.5px;
            }
            .nav-links {
                display: flex;
                gap: var(--spacing-sm);
                align-items: center;
            }
            .nav-link {
                display: flex;
                align-items: center;
                padding: var(--spacing-xs) var(--spacing-md);
                text-decoration: none;
                color: var(--color-text-muted);
                font-weight: 700;
                border-radius: var(--radius-full);
                transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
                border: 1px solid transparent;
                white-space: nowrap;
            }
            .nav-link:hover {
                color: var(--color-text);
                background: var(--color-surface-variant, rgba(255, 255, 255, 0.05));
                border-color: var(--color-border);
            }
            .nav-link.active {
                background: var(--color-primary);
                color: white;
                border-color: var(--color-primary);
            }
            .nav-link.active .lucide {
                color: white;
            }
            .lucide {
                width: 16px;
                height: 16px;
                margin-right: var(--spacing-sm);
                stroke-width: 2.5px;
                stroke-linecap: round;
                stroke-linejoin: round;
            }
            .theme-btn {
                display: flex;
                align-items: center;
                justify-content: center;
                width: 36px;
                height: 36px;
                border: 1px solid var(--color-border);
                border-radius: var(--radius-full);
                background: var(--color-surface-variant, transparent);
                color: var(--color-text-muted);
                cursor: pointer;
                transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
                flex-shrink: 0;
            }
            .theme-btn:hover {
                color: var(--color-text);
                background: var(--color-surface);
                border-color: var(--color-border-hover);
                transform: scale(1.05);
            }
            .theme-btn .lucide {
                margin-right: 0;
                width: 16px;
                height: 16px;
            }
        `,
    ];

    constructor() {
        super();
        this.isLoggedIn = false;
        this.isAdmin = false;
        this._theme = ThemeManager.current;
    }

    async connectedCallback() {
        super.connectedCallback();
        this.refreshAuth();
        window.addEventListener("auth-changed", () => this.refreshAuth());
        window.addEventListener("theme-changed", (e) => {
            this._theme = e.detail.theme;
        });
    }

    disconnectedCallback() {
        super.disconnectedCallback();
        window.removeEventListener("auth-changed", () => this.refreshAuth());
        window.removeEventListener("theme-changed", (e) => {
            this._theme = e.detail.theme;
        });
    }

    async refreshAuth() {
        const token = (await cookieStore.get("token"))?.value;

        if (token) {
            this.isLoggedIn = true;
            try {
                const res = await fetch("/api/v1/users/@me", { headers: { Authorization: token } });
                if (res.ok) {
                    const json = await res.json();
                    this.isAdmin = json.data?.role === "Admin";
                } else {
                    this.isLoggedIn = false;
                    this.isAdmin = false;
                }
            } catch (e) {
                this.isLoggedIn = false;
                this.isAdmin = false;
            }
        } else {
            this.isLoggedIn = false;
            this.isAdmin = false;
        }
    }

    async _toggleTheme() {
        await ThemeManager.toggle();
        this._theme = ThemeManager.current;
    }

    firstUpdated() {
        createIcons({
            icons: {
                Home,
                Save,
                User,
                LogIn,
                Moon,
                Sun,
                Shield,
                Settings,
            },
            nameAttr: "data-lucide",
            root: this.shadowRoot,
        });
    }

    updated(changedProperties) {
        if (changedProperties.has("_theme")) {
            createIcons({
                icons: { Home, Save, User, LogIn, Moon, Sun, Shield, Settings },
                nameAttr: "data-lucide",
                root: this.shadowRoot,
            });
        }
    }

    render() {
        const isHome = window.location.pathname === "/";
        const isAdmin = window.location.pathname.startsWith("/dev") || window.location.pathname.startsWith("/users");
        const isProfile = window.location.pathname.startsWith("/profile");
        const isSaves = window.location.pathname.startsWith("/saves");
        const isSettings = window.location.pathname.startsWith("/settings");
        const isLogin = window.location.pathname.startsWith("/auth/login");
        const isDark = this._theme === "dark";

        return html`
            <header class="status-bar">
                <a href="/" class="logo">emuNEX</a>
                <nav class="nav-links">
                    <a href="/" class="nav-link ${isHome ? "active" : ""}">
                        <span>Home</span>
                    </a>
                    ${this.isAdmin
                        ? html`
                              <a href="/dev" class="nav-link ${isAdmin ? "active" : ""}">
                                  <span>Admin</span>
                              </a>
                          `
                        : ""}
                    ${this.isLoggedIn
                        ? html`
                              <a href="/profile" class="nav-link ${isProfile ? "active" : ""}">
                                  <span>Profile</span>
                              </a>
                              <a href="/settings" class="nav-link ${isSettings ? "active" : ""}">
                                  <span>Settings</span>
                              </a>
                              <a href="/saves" class="nav-link ${isSaves ? "active" : ""}">
                                  <span>Saves</span>
                              </a>
                          `
                        : html`
                              <a href="/auth/login" class="nav-link ${isLogin ? "active" : ""}">
                                  <span>Login</span>
                              </a>
                          `}

                    <button
                        class="theme-btn"
                        @click=${this._toggleTheme}
                        title="${isDark ? "Switch to light mode" : "Switch to dark mode"}"
                    >
                        <i data-lucide="${isDark ? "sun" : "moon"}"></i>
                    </button>
                </nav>
            </header>
        `;
    }
}

customElements.define("emunex-navbar", EmunexNavbar);
