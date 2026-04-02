import { LitElement, css, html } from "lit";
import "../components/navbar.js";
import {
    baseTokens,
    buttonStyles,
    cardStyles,
    devTokens,
    formStyles,
    modalStyles,
    pageHostStyles,
    statusStyles,
    tableStyles
} from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexConsolesManagePage extends LitElement {
  static properties = {
    _consoles: { type: Array, state: true },
    _filtered: { type: Array, state: true },
    _loading: { type: Boolean, state: true },
    _error: { type: String, state: true },
    _status: { type: String, state: true },
    _statusType: { type: String, state: true },
    _editModalOpen: { type: Boolean, state: true },
    _editingConsole: { type: Object, state: true },
    _editColor: { type: String, state: true },
  };

  static styles = [
    baseTokens, devTokens, pageHostStyles, cardStyles, formStyles, 
    buttonStyles, tableStyles, statusStyles, modalStyles,
    css`
      .auth-container { max-width: 800px; }
      .header-actions {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-md);
      }
      .color-picker-row { display: flex; gap: var(--spacing-sm); align-items: center; }
      .color-hex { flex: 1; font-family: monospace; text-transform: uppercase; }
      .color-preview {
        width: 42px; height: 42px; border-radius: var(--radius-sm);
        border: 2px solid var(--color-border); cursor: pointer; flex-shrink: 0;
      }
      input[type="color"] { display: none; }
      .action-btns { white-space: nowrap; font-size: 0; }
      .action-btns .popout-btn { margin: 4px var(--spacing-xs) 0 0; display: inline-block; vertical-align: middle; }
      .action-btns .popout-btn:last-child { margin-right: 0; }
      .action-btns .btn-front { padding: 4px 10px; font-size: 0.75rem; min-width: 60px; }
    `
  ];

  constructor() {
    super();
    this._consoles = [];
    this._filtered = [];
    this._loading = true;
    this._error = "";
    this._status = "";
    this._statusType = "";
    this._editModalOpen = false;
    this._editingConsole = null;
    this._editColor = "";
    this._searchTimeout = null;
    this._currentQuery = "";
  }

  connectedCallback() {
    super.connectedCallback();
    this.fetchConsoles();
  }

  async fetchConsoles() {
    this._loading = true;
    try {
      const token = (await cookieStore.get("token"))?.value;
      const res = await fetch("/api/v1/consoles", {
        headers: { Authorization: token },
      });
      const json = await res.json();
      if (res.ok) {
        this._consoles = json.data || [];
        if (this._currentQuery && this._applyFilter) {
          this._applyFilter();
        } else {
          this._filtered = this._consoles;
        }
      } else {
        this._error = json.error || "Failed to load";
      }
    } catch (err) {
      this._error = "Network error";
    } finally {
      this._loading = false;
    }
  }

  _handleSearch(e) {
    const q = e.target.value.toLowerCase();
    this._currentQuery = q;
    clearTimeout(this._searchTimeout);
    this._searchTimeout = setTimeout(() => {
      this._applyFilter();
    }, 300);
  }

  _applyFilter() {
    const q = this._currentQuery;
    if (!q) {
      this._filtered = [...this._consoles];
      return;
    }
    this._filtered = this._consoles.filter(c => c.name.toLowerCase().includes(q));
  }

  render() {
    return html`
      <emunex-navbar></emunex-navbar>
      <div class="auth-container">
        <div class="auth-card">
          <header class="card-header">
            <a href="/dev" class="back-link">Back</a>
            <h1>emuNEX</h1>
          </header>

          <div class="content">
            <div class="header-actions">
              <a href="/consoles/upload" class="popout-btn btn-fit" style="text-decoration: none">
                <span class="btn-edge"></span>
                <span class="btn-front" style="padding: 6px 12px; font-size: 0.8rem; min-width: auto">+ Add Console</span>
              </a>
            </div>

            <div class="form-group" style="margin-bottom: var(--spacing-md)">
              <input type="text" placeholder="Search consoles by name..." style="font-weight: 700" @input=${this._handleSearch} />
            </div>

            <div style="overflow-x: auto">
              <table style="margin-top: 0">
                <thead>
                  <tr><th>Console Name</th><th>Card Accent</th><th>Actions</th></tr>
                </thead>
                <tbody>
                  ${this._loading ? html`<tr><td colspan="3" style="text-align: center">Loading...</td></tr>` : ""}
                  ${this._error ? html`<tr><td colspan="3" style="text-align: center; color: var(--color-error)">${this._error}</td></tr>` : ""}
                  ${!this._loading && !this._error && this._filtered.length === 0 ? html`<tr><td colspan="3" style="text-align: center">No consoles found.</td></tr>` : ""}
                  ${this._filtered.map(c => html`
                    <tr>
                      <td style="font-weight: 700;">${c.name}</td>
                      <td>
                        <div style="display: flex; align-items: center; gap: 8px;">
                          <div style="width: 20px; height: 20px; border-radius: 4px; background: ${c.card_color || "#333"}; border: 1px solid var(--color-border);"></div>
                          <code style="font-size: 0.8rem; opacity: 0.8">${c.card_color || "Default"}</code>
                        </div>
                      </td>
                      <td class="action-btns">
                        <button class="popout-btn btn-fit btn-info" @click=${() => this.openEdit(c)}>
                          <span class="btn-edge"></span><span class="btn-front">Edit</span>
                        </button>
                        <button class="popout-btn btn-fit btn-error" @click=${() => this.deleteConsole(c.name)}>
                          <span class="btn-edge"></span><span class="btn-front">Delete</span>
                        </button>
                      </td>
                    </tr>
                  `)}
                </tbody>
              </table>
            </div>

            ${this._status ? html`
              <div class="status-box ${this._statusType === "error" ? "status-error" : "status-success"}">${this._status}</div>
            ` : ""}
          </div>
        </div>
      </div>

      ${this._editModalOpen ? this.renderModal() : ""}
    `;
  }

  renderModal() {
    return html`
      <div class="modal-backdrop open">
        <div class="modal-card">
          <div class="content">
            <div class="section-hint">Edit Console Metadata</div>
            <form @submit=${this.submitEdit}>
              <div class="form-group">
                <label>Console Name (Read-only)</label>
                <input type="text" .value=${this._editingConsole?.name} readonly style="opacity: 0.7; cursor: not-allowed" />
              </div>
              <div class="form-group">
                <label>Card Accent Color</label>
                <div class="color-picker-row">
                  <input type="text" class="color-hex" .value=${this._editColor} @input=${e => this._editColor = e.target.value.toUpperCase()} />
                  <input type="color" id="edit-color-picker" .value=${this._editColor} @input=${e => this._editColor = e.target.value.toUpperCase()} />
                  <div class="color-preview" style="background-color: ${this._editColor}" @click=${() => this.renderRoot.querySelector("#edit-color-picker").click()}></div>
                </div>
              </div>
              <div style="display: flex; gap: var(--spacing-md); margin-top: var(--spacing-lg)">
                <button type="submit" class="popout-btn btn-fit btn-info" style="flex: 1">
                  <span class="btn-edge"></span><span class="btn-front" style="padding: 10px">Save Changes</span>
                </button>
                <button type="button" class="popout-btn btn-fit btn-cancel" style="flex: 1" @click=${this.closeModal}>
                  <span class="btn-edge"></span><span class="btn-front" style="padding: 10px">Cancel</span>
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    `;
  }

  showStatus(msg, type) {
    this._status = msg;
    this._statusType = type;
    setTimeout(() => this._status = "", 3000);
  }

  openEdit(c) {
    this._editingConsole = c;
    this._editColor = c.card_color || "#333333";
    this._editModalOpen = true;
  }

  closeModal() {
    this._editModalOpen = false;
  }

  async submitEdit(e) {
    e.preventDefault();
    const name = this._editingConsole.name;
    try {
      const token = (await cookieStore.get("token"))?.value;
      const res = await fetch(`/api/v1/consoles/${name}`, {
        method: "PUT",
        headers: { Authorization: token, "Content-Type": "application/json" },
        body: JSON.stringify({ card_color: this._editColor }),
      });
      if (res.ok) {
        this.showStatus("Console updated!", "success");
        this.closeModal();
        this.fetchConsoles();
      } else {
        const err = await res.json();
        this.showStatus(`Error: ${err.error}`, "error");
      }
    } catch {
      this.showStatus("Update failed.", "error");
    }
  }

  async deleteConsole(name) {
    if (!confirm(`Are you sure you want to delete the console '${name}'?`)) return;
    try {
      const token = (await cookieStore.get("token"))?.value;
      const res = await fetch(`/api/v1/consoles/${encodeURIComponent(name)}`, {
        method: "DELETE",
        headers: { Authorization: token },
      });
      if (res.ok) {
        this.showStatus(`Console ${name} deleted.`, "success");
        this.fetchConsoles();
      } else {
        const err = await res.json();
        this.showStatus(`Error: ${err.error}`, "error");
      }
    } catch {
      this.showStatus("Deletion failed.", "error");
    }
  }
}

customElements.define("emunex-consoles-manage-page", EmunexConsolesManagePage);
