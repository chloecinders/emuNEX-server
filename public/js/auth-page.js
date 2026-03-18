import { LitElement, html, css } from "lit";
import {
  baseTokens, authTokens, pageHostStyles, cardStyles,
  formStyles, buttonStyles, statusStyles,
} from "./components/shared-styles.js";

class EmunexAuthPage extends LitElement {
  static properties = {
    authType: { type: String, attribute: "auth-type" },
    domain:   { type: String },
    _error:   { type: String, state: true },
    _loading: { type: Boolean, state: true },
  };

  static styles = [
    baseTokens, authTokens, pageHostStyles, cardStyles, formStyles, buttonStyles, statusStyles,
    css`
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
    `,
  ];

  constructor() {
    super();
    this.authType = "login";
    this.domain = "";
    this._error = "";
    this._loading = false;
  }

  render() {
    const isRegister = this.authType === "register";
    return html`
      <style>
        @import url("https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap");
      </style>
      <div class="auth-container">
        <div class="auth-card">
          <header class="card-header">
            <h1>emuNEX</h1>
            <p class="tagline">Remote Emulation &amp; Library Management</p>
          </header>

          <div class="content">
            ${this._error ? html`<div class="error-bubble">${this._error}</div>` : ""}

            <form @submit=${this._handleSubmit}>
              <div class="section-hint">${isRegister ? "Create Account" : "Authenticate"}</div>

              <div class="form-group">
                <label for="username">Username</label>
                <input id="username" type="text" placeholder="Enter your username" required autocomplete="username" />
              </div>

              <div class="form-group">
                <label for="password">Password</label>
                <input id="password" type="password" placeholder="••••••••" required autocomplete="current-password" />
              </div>

              ${isRegister ? html`
                <div class="form-group">
                  <label for="confirm">Confirm Password</label>
                  <input id="confirm" type="password" placeholder="••••••••" required />
                </div>
                <div class="form-group">
                  <label for="invite_code">Invite Code</label>
                  <input id="invite_code" type="text" placeholder="Your invite code" required />
                </div>
              ` : ""}

              <button type="submit" class="popout-btn" ?disabled=${this._loading}>
                <span class="btn-edge"></span>
                <span class="btn-front">
                  ${this._loading
                    ? (isRegister ? "Creating…" : "Connecting…")
                    : (isRegister ? "Register" : "Connect")}
                </span>
              </button>
            </form>

            <div class="auth-footer">
              ${isRegister
                ? html`Already have an account? <a href="/auth/login">Login instead</a>`
                : html`Don't have an account? <a href="/auth/register">Register instead</a>`}
            </div>
          </div>
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
      localStorage.setItem("token", token);

      if (window.cookieStore) {
        await cookieStore.set({ name: "token", value: token, expires: Date.now() + 86400000, path: "/" });
      } else {
        document.cookie = `token=${token}; path=/; max-age=86400; SameSite=Lax`;
      }

      setTimeout(() => {
        window.location.href = `emunex://login?token=${token}&domain=${encodeURIComponent(this.domain)}&storage_path=/storage`;
      }, 100);
    } catch (err) {
      this._error = err.message;
      this._loading = false;
    }
  }
}

customElements.define("emunex-auth-page", EmunexAuthPage);
