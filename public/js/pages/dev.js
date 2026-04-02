import { LitElement, css, html } from "lit";
import "../components/navbar.js";
import {
    baseTokens,
    buttonStyles,
    cardStyles,
    devTokens,
    formStyles,
    pageHostStyles,
    statusStyles,
} from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexDevPage extends LitElement {
    static properties = {
        role: { type: String },
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _reports: { type: Array, state: true },
    };

    static styles = [
        baseTokens,
        devTokens,
        pageHostStyles,
        cardStyles,
        formStyles,
        buttonStyles,
        statusStyles,
        css`
            .nav-links {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
                gap: var(--spacing-md);
            }
            .nav-btn {
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: var(--spacing-sm);
                padding: var(--spacing-lg);
                background: var(--color-surface-variant);
                border: 2px solid var(--color-border);
                border-radius: var(--radius-md);
                color: var(--color-text);
                text-decoration: none;
                font-weight: 700;
                transition: all 0.2s;
                text-align: center;
            }
            .nav-btn:hover {
                border-color: var(--color-primary);
                transform: translateY(-2px);
                box-shadow: var(--shadow-md);
            }
            .admin-section {
                margin-top: var(--spacing-xl);
                padding-top: var(--spacing-lg);
                border-top: 1px solid var(--color-border);
            }
            .role-label {
                font-size: 0.85rem;
                color: var(--color-text-muted);
                margin-bottom: var(--spacing-md);
            }
        `,
    ];

    constructor() {
        super();
        this.role = "";
        this._status = "";
        this._statusType = "";
        this._reports = [];
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchReports();
    }

    async fetchReports() {
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/reports", {
                headers: { Authorization: token },
            });
            if (res.ok) {
                const json = await res.json();
                this._reports = (json.data || []).filter((r) => r.status === "open");
            }
        } catch (e) {
            console.error("Failed to fetch reports:", e);
        }
    }

    render() {
        const isAdmin = this.role === "Admin";
        return html`
            <emunex-navbar></emunex-navbar>
            <div class="auth-container">
                <div class="auth-card">
                    <header class="card-header">
                        <h1>emuNEX</h1>
                    </header>

                    <div class="content">
                        <div class="section-hint">Management Actions</div>

                        <nav class="nav-links">
                            <a href="/roms" class="nav-btn"><span>Manage ROM</span></a>
                            <a href="/emulators" class="nav-btn"><span>Manage Emulator</span></a>
                            <a href="/consoles" class="nav-btn"><span>Manage Consoles</span></a>
                            ${isAdmin ? html`<a href="/users" class="nav-btn"><span>Manage Users</span></a>` : ""}
                            ${isAdmin
                                ? html`<a href="/search_sections" class="nav-btn"
                                      ><span>Manage Search Sections</span></a
                                  >`
                                : ""}
                            <a href="/reports" class="nav-btn">
                                <span>Manage Reports</span>
                                ${this._reports.length > 0
                                    ? html`
                                          <span
                                              style="background: var(--color-error, #e53e3e); color: var(--color-text-on-primary, white); border-radius: 12px; padding: 2px 8px; font-size: 0.75rem;"
                                              >${this._reports.length} Open</span
                                          >
                                      `
                                    : ""}
                            </a>
                        </nav>

                        ${isAdmin
                            ? html`
                                  <div class="admin-section">
                                      <div class="section-hint">System Maintenance</div>
                                      <p class="role-label">Connected as: <strong>${this.role}</strong></p>
                                      <button class="popout-btn" @click=${this._triggerUpdate}>
                                          <span class="btn-edge"></span>
                                          <span class="btn-front">Pull Update &amp; Restart</span>
                                      </button>
                                      ${this._status
                                          ? html`
                                                <div
                                                    class="status-box ${this._statusType === "error"
                                                        ? "status-error"
                                                        : "status-success"}"
                                                >
                                                    ${this._status}
                                                </div>
                                            `
                                          : ""}
                                  </div>
                              `
                            : ""}
                    </div>
                </div>
            </div>
        `;
    }

    async _triggerUpdate() {
        if (!confirm("Are you sure you want to restart the server for an update?")) return;

        const btn = this.renderRoot.querySelector(".popout-btn");
        btn.disabled = true;
        this._status = "Triggering update…";
        this._statusType = "success";

        try {
            const response = await fetch("/admin/update", { method: "POST" });
            const text = await response.text();
            if (text && text.length > 0) {
                this._status = "Error: " + text;
                this._statusType = "error";
                btn.disabled = false;
            } else {
                this._status = "Error: Received empty response from server.";
                this._statusType = "error";
                btn.disabled = false;
            }
        } catch {
            this._status = "Server is restarting…";
            this._statusType = "success";
            setTimeout(() => location.reload(), 8000);
        }
    }
}

customElements.define("emunex-dev-page", EmunexDevPage);
