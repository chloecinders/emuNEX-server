import { LitElement, css, html } from "lit";
import { CheckCircle, Download, ExternalLink, createIcons } from "lucide";
import "../components/navbar.js";
import {
    authTokens,
    baseTokens,
    buttonStyles,
    cardStyles,
    formStyles,
    pageHostStyles,
    statusStyles,
} from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexAuthPage extends LitElement {
    static properties = {
        authType: { type: String, attribute: "auth-type" },
        domain: { type: String },
        _error: { type: String, state: true },
        _loading: { type: Boolean, state: true },
        _successToken: { type: String, state: true },
        _role: { type: String, state: true },
    };

    static styles = [
        baseTokens,
        authTokens,
        pageHostStyles,
        cardStyles,
        formStyles,
        buttonStyles,
        statusStyles,
        css`
            .popout-btn.btn-admin {
                --btn-color-primary: #e66b1d;
                --btn-color-dark: #c45612;
            }
            .auth-footer {
                margin-top: var(--spacing-md);
                text-align: center;
                font-size: 0.85rem;
                font-weight: 600;
            }
            .auth-footer a {
                color: var(--color-primary);
                text-decoration: none;
                transition: color 0.2s;
            }
            .auth-footer a:hover {
                color: var(--color-primary-dark);
                text-decoration: underline;
            }
            .error-bubble {
                background: #fff5f5;
                border: 1px solid #ffccd1;
                padding: var(--spacing-md) var(--spacing-lg);
                border-radius: var(--radius-md);
                color: #e53e3e;
                font-weight: 700;
                font-size: 0.85rem;
                text-align: center;
                margin-bottom: var(--spacing-md);
            }
            .success-view {
                display: flex;
                flex-direction: column;
                align-items: center;
                text-align: center;
                gap: var(--spacing-md);
                padding: var(--spacing-lg) 0;
            }
            .success-icon {
                width: 64px;
                height: 64px;
                color: var(--color-primary);
                margin-bottom: var(--spacing-sm);
            }
            .success-title {
                font-size: 1.5rem;
                font-weight: 800;
                color: var(--color-text);
                margin: 0;
            }
            .success-text {
                font-size: 0.95rem;
                color: var(--color-text-muted);
                margin: 0 0 var(--spacing-md) 0;
                line-height: 1.5;
            }
            .success-actions {
                display: flex;
                flex-direction: column;
                gap: var(--spacing-md);
                width: 100%;
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

    constructor() {
        super();
        this.authType = "login";
        this.domain = "";
        this._error = "";
        this._loading = false;
        this._successToken = "";
        this._role = "";
    }

    async connectedCallback() {
        super.connectedCallback();
        const existingToken = (await cookieStore.get("token"))?.value;
        if (existingToken) {
            this._successToken = existingToken;
            this._fetchRole(existingToken);
        }
    }

    async _fetchRole(token) {
        try {
            const res = await fetch("/api/v1/users/@me", {
                headers: { Authorization: token },
            });
            if (res.ok) {
                const json = await res.json();
                this._role = json.data?.role || "";
            }
        } catch (e) {
            console.error("Failed to fetch role:", e);
        }
    }

    updated(changedProperties) {
        if (changedProperties.has("_successToken") && this._successToken) {
            createIcons({
                icons: { CheckCircle, Download, ExternalLink, Settings },
                nameAttr: "data-lucide",
                root: this.shadowRoot,
            });
        }
    }

    render() {
        const isRegister = this.authType === "register";
        return html`
            <emunex-navbar></emunex-navbar>
            <div class="auth-container">
                <div class="auth-card">
                    <header class="card-header">
                        <h1>emuNEX</h1>
                        <p class="tagline">Remote Emulation &amp; Library Management</p>
                    </header>

                    <div class="content">
                        ${this._successToken ? this._renderSuccess() : this._renderForm(isRegister)}
                    </div>
                </div>
            </div>
        `;
    }

    _renderForm(isRegister) {
        return html`
            ${this._error ? html`<div class="error-bubble">${this._error}</div>` : ""}

            <form @submit=${this._handleSubmit}>
                <div class="section-hint">${isRegister ? "Create Account" : "Authenticate"}</div>

                <div class="form-group">
                    <label for="username">Username</label>
                    <input
                        id="username"
                        type="text"
                        placeholder="Enter your username"
                        required
                        autocomplete="username"
                    />
                </div>

                <div class="form-group">
                    <label for="password">Password</label>
                    <input
                        id="password"
                        type="password"
                        placeholder="••••••••"
                        required
                        autocomplete="current-password"
                    />
                </div>

                ${isRegister
                    ? html`
                          <div class="form-group">
                              <label for="confirm">Confirm Password</label>
                              <input id="confirm" type="password" placeholder="••••••••" required />
                          </div>
                          <div class="form-group">
                              <label for="invite_code">Invite Code</label>
                              <input id="invite_code" type="text" placeholder="Your invite code" required />
                          </div>
                      `
                    : ""}

                <button type="submit" class="popout-btn" ?disabled=${this._loading}>
                    <span class="btn-edge"></span>
                    <span class="btn-front">
                        ${this._loading
                            ? isRegister
                                ? "Creating…"
                                : "Connecting…"
                            : isRegister
                              ? "Register"
                              : "Connect"}
                    </span>
                </button>
            </form>

            <div class="auth-footer">
                ${isRegister
                    ? html`Already have an account? <a href="/auth/login">Login instead</a>`
                    : html`Don't have an account? <a href="/auth/register">Register instead</a>`}
            </div>
        `;
    }

    _renderSuccess() {
        const deeplink = `emunex://login?token=${this._successToken}&domain=${encodeURIComponent(this.domain)}&storage_path=/storage`;

        return html`
            <div class="success-view">
                <h2 class="success-title">Authorized</h2>
                <p class="success-text">
                    You have been authorized. You can now open emuNEX or download the client to get started.
                </p>

                <div class="success-actions">
                    <a href="${deeplink}" class="popout-btn">
                        <span class="btn-edge"></span>
                        <span class="btn-front btn-front-flex">
                            <i data-lucide="external-link"></i>
                            CONNECT TO EMUNEX CLIENT
                        </span>
                    </a>

                    <a
                        href="https://github.com/chloecinders/emuNEX-client/releases"
                        class="popout-btn btn-secondary"
                        target="_blank"
                    >
                        <span class="btn-edge"></span>
                        <span class="btn-front btn-front-flex">
                            <i data-lucide="download"></i>
                            Download Client
                        </span>
                    </a>

                    ${this._role === "Admin" || this._role === "Moderator"
                        ? html`
                              <a href="/dev" class="popout-btn btn-admin">
                                  <span class="btn-edge"></span>
                                  <span class="btn-front btn-front-flex">
                                      <i data-lucide="settings"></i>
                                      Admin Panel
                                  </span>
                              </a>
                          `
                        : ""}
                </div>
            </div>
        `;
    }

    async _handleSubmit(e) {
        e.preventDefault();
        this._error = "";
        this._loading = true;

        const username = this.renderRoot.querySelector("#username").value;
        const password = this.renderRoot.querySelector("#password").value;
        let invite_code = "";

        if (this.authType === "register") {
            const confirm = this.renderRoot.querySelector("#confirm").value;
            invite_code = this.renderRoot.querySelector("#invite_code").value;
            if (password !== confirm) {
                this._error = "Passwords do not match";
                this._loading = false;
                return;
            }
        }

        try {
            const res = await fetch(`/api/v1/${this.authType}`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username, password, invite_code }),
            });

            const json = await res.json();
            if (!res.ok || !json.success) {
                throw new Error(json.message || json.error || `Server returned ${res.status}`);
            }

            const token = json.data.token;
            await cookieStore.set({
                name: "token",
                value: token,
                expires: Date.now() + 31536000000,
                path: "/",
            });

            window.dispatchEvent(new CustomEvent("auth-changed"));

            this._successToken = token;
            this._fetchRole(token);
            this._loading = false;

            setTimeout(() => {
                window.location.href = `emunex://login?token=${token}&domain=${encodeURIComponent(this.domain)}&storage_path=/storage`;
            }, 500);
        } catch (err) {
            this._error = err.message;
            this._loading = false;
        }
    }
}

customElements.define("emunex-auth-page", EmunexAuthPage);
