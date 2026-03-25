import { LitElement, html, css } from "lit";
import {
  baseTokens, devTokens, pageHostStyles, cardStyles,
  formStyles, buttonStyles, statusStyles,
} from "./components/shared-styles.js";

class EmunexDevPage extends LitElement {
  static properties = {
    role: { type: String },
    _status: { type: String, state: true },
    _statusType: { type: String, state: true },
    _reports: { type: Array, state: true },
  };

  static styles = [
    baseTokens, devTokens, pageHostStyles, cardStyles, formStyles, buttonStyles, statusStyles,
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
      const res = await fetch("/api/v1/reports", {
        headers: { Authorization: localStorage.getItem("token") },
      });
      if (res.ok) {
        const json = await res.json();
        this._reports = (json.data || []).filter(r => r.status === 'open');
      }
    } catch (e) {
      console.error("Failed to fetch reports:", e);
    }
  }

render() {
  const isAdmin = this.role === "Admin";
  return html`
      <style>
        @import url("https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap");
      </style>
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
              ${isAdmin ? html`<a href="/search_sections" class="nav-btn"><span>Manage Search Sections</span></a>` : ""}
              ${isAdmin ? html`<a href="/roms/bulk_upload" class="nav-btn"><span>Bulk Import Games</span></a>` : ""}
            </nav>

            ${this._reports.length > 0 ? html`
              <div class="admin-section" style="margin-top: var(--spacing-xl);">
                <div class="section-hint" style="color: var(--color-error)">Open Reports</div>
                <div style="display: flex; flex-direction: column; gap: var(--spacing-sm);">
                  ${this._reports.map(report => html`
                    <div style="background: var(--color-surface-variant); padding: var(--spacing-md); border-radius: var(--radius-md); border: 1px solid var(--color-border); display: flex; justify-content: space-between; align-items: flex-start; gap: var(--spacing-md);">
                      <div>
                        <div style="font-weight: 700; margin-bottom: 4px;">${report.rom_title} <span style="font-size: 0.8em; color: var(--color-text-muted); font-family: monospace;">(${report.report_type})</span></div>
                        <div style="font-size: 0.85rem; color: var(--color-text-muted); margin-bottom: 4px;">Reported by: ${report.username}</div>
                        <div style="font-size: 0.9rem; background: rgba(0,0,0,0.1); padding: 8px; border-radius: 4px;">${report.description}</div>
                      </div>
                      <button class="popout-btn btn-fit" style="margin: 0; min-width: 100px; flex-shrink: 0;" @click=${() => this._resolveReport(report.id)}>
                        <span class="btn-edge"></span>
                        <span class="btn-front" style="padding: 6px 12px; font-size: 0.8rem;">Resolve</span>
                      </button>
                    </div>
                  `)}
                </div>
              </div>
            ` : html`
              <div class="admin-section" style="margin-top: var(--spacing-xl);">
                <div class="section-hint">Open Reports</div>
                <div style="color: var(--color-text-muted); font-size: 0.9rem; text-align: center; padding: var(--spacing-md); border: 1px dashed var(--color-border); border-radius: var(--radius-md);">
                  No open reports.
                </div>
              </div>
            `}

            ${isAdmin ? html`
              <div class="admin-section">
                <div class="section-hint">System Maintenance</div>
                <p class="role-label">Connected as: <strong>${this.role}</strong></p>
                <button class="popout-btn" @click=${this._triggerUpdate}>
                  <span class="btn-edge"></span>
                  <span class="btn-front">Pull Update &amp; Restart</span>
                </button>
                ${this._status ? html`
                  <div class="status-box ${this._statusType === "error" ? "status-error" : "status-success"}">
                    ${this._status}
                  </div>
                ` : ""}
              </div>
            ` : ""}
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

  async _resolveReport(id) {
  if (!confirm("Are you sure you resolved this report?")) return;

  try {
    const response = await fetch(`/api/v1/reports/${id}/resolve`, {
        method: "PATCH",
        headers: { Authorization: localStorage.getItem("token") },
      });

      if (response.ok) {
        this._reports = this._reports.filter(r => r.id !== id);
      } else {
        alert("Failed to resolve report");
      }
    } catch (e) {
      alert("Network error. Please try again.");
    }
  }
}

customElements.define("emunex-dev-page", EmunexDevPage);
