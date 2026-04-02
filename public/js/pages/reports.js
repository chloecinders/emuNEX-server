import { LitElement, css, html } from "lit";
import "../components/navbar.js";
import "../components/rom-edit-form.js";
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

class EmunexReportsPage extends LitElement {
    static properties = {
        role: { type: String },
        userid: { type: String },
        _reports: { type: Array, state: true },
        _selectedReportId: { type: String, state: true },
        _editingRom: { type: Object, state: true },
        _consoles: { type: Array, state: true },
        _loading: { type: Boolean, state: true },
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
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
            .app-layout {
                width: 100%;
                max-width: 1400px;
                margin: 0 auto;
                padding: var(--spacing-lg);
                box-sizing: border-box;
                display: grid;
                grid-template-columns: 400px 1fr;
                gap: var(--spacing-xl);
                height: calc(100vh - 80px);
            }
            .panel {
                background: var(--color-surface);
                border-radius: var(--radius-lg);
                box-shadow: var(--shadow-md);
                border: 1px solid var(--color-border);
                display: flex;
                flex-direction: column;
                overflow: hidden;
            }
            .panel-header {
                padding: var(--spacing-md) var(--spacing-lg);
                font-weight: 800;
                display: flex;
                justify-content: space-between;
                align-items: center;
                min-height: 64px;
            }
            .panel-content {
                flex: 1;
                overflow-y: auto;
                padding: var(--spacing-lg);
            }
            .report-card {
                padding: var(--spacing-md);
                border: 2px solid var(--color-border);
                border-radius: var(--radius-md);
                margin-bottom: var(--spacing-md);
                cursor: pointer;
                transition: all 0.2s;
                background: var(--color-surface-variant);
            }
            .report-card:hover {
                border-color: var(--color-primary);
            }
            .report-card.selected {
                border-color: var(--color-primary);
                box-shadow: 0 0 0 4px rgba(230, 107, 29, 0.1);
            }

            .report-meta {
                display: flex;
                justify-content: space-between;
                font-size: 0.8rem;
                color: var(--color-text-muted);
                margin-bottom: 4px;
                font-weight: 700;
            }
            .report-title {
                font-weight: 800;
                font-size: 1rem;
                margin-bottom: 4px;
                color: var(--color-text);
            }
            .report-desc {
                font-size: 0.9rem;
                margin-bottom: var(--spacing-md);
                background: rgba(0, 0, 0, 0.05);
                padding: 8px;
                border-radius: 4px;
            }

            .claim-badge {
                display: inline-block;
                padding: 2px 6px;
                border-radius: 4px;
                font-size: 0.75rem;
                font-weight: 800;
                margin-bottom: 8px;
            }
            .badge-open {
                background: rgba(59, 130, 246, 0.1);
                color: #3b82f6;
            }
            .badge-claimed-me {
                background: rgba(16, 185, 129, 0.1);
                color: #10b981;
            }
            .badge-claimed-other {
                background: rgba(245, 158, 11, 0.1);
                color: #f59e0b;
            }

            .empty-state {
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                height: 100%;
                color: var(--color-text-muted);
                text-align: center;
            }
        `,
    ];

    constructor() {
        super();
        this._reports = [];
        this._selectedReportId = null;
        this._editingRom = null;
        /** @type {never[]} we don't use typescript here so why does lit complain anyway?? */
        this._consoles = [];
        this._loading = false;
        this._status = "";
        this._statusType = "";
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchConsoles();
        this.fetchReports();
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 3000);
    }

    async fetchConsoles() {
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/consoles", { headers: { Authorization: token } });
            const json = await res.json();
            this._consoles = json.data || [];
        } catch {}
    }

    async fetchReports() {
        this._loading = true;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/reports", { headers: { Authorization: token } });
            if (res.ok) {
                const json = await res.json();
                this._reports = (json.data || []).filter((r) => r.status === "open");

                if (this._selectedReportId && !this._reports.find((r) => r.id === this._selectedReportId)) {
                    this._selectedReportId = null;
                    this._editingRom = null;
                }
            }
        } catch (e) {
            this.showStatus("Failed to fetch reports", "error");
        } finally {
            this._loading = false;
        }
    }

    async selectReport(report) {
        this._selectedReportId = report.id;
        this._editingRom = null;
        if (report.claimed_by !== this.userid) return;

        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/roms/${encodeURIComponent(report.rom_id)}`, {
                headers: { Authorization: token },
            });
            if (res.ok) {
                const json = await res.json();
                this._editingRom = json.data;
            }
        } catch (e) {
            this.showStatus("Failed to load ROM data", "error");
        }
    }

    async claimReport(e, id) {
        e.stopPropagation();
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/reports/${id}/claim`, {
                method: "PATCH",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Report claimed", "success");
                await this.fetchReports();
                const report = this._reports.find((r) => r.id === id);
                if (report) this.selectReport(report);
            } else {
                this.showStatus("Failed to claim report (may be claimed already)", "error");
            }
        } catch (e) {
            this.showStatus("Network error claiming report", "error");
        }
    }

    async unclaimReport(e, id) {
        e.stopPropagation();
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/reports/${id}/unclaim`, {
                method: "PATCH",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Report unclaimed", "success");
                if (this._selectedReportId === id) {
                    this._editingRom = null;
                }
                await this.fetchReports();
            } else {
                this.showStatus("Failed to unclaim report", "error");
            }
        } catch (e) {
            this.showStatus("Network error", "error");
        }
    }

    async resolveReport(id) {
        if (!confirm("Are you sure you want to resolve this report?")) return;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/reports/${id}/resolve`, {
                method: "PATCH",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Report resolved", "success");
                this._selectedReportId = null;
                this._editingRom = null;
                await this.fetchReports();
            } else {
                this.showStatus("Failed to resolve", "error");
            }
        } catch (e) {
            this.showStatus("Network error", "error");
        }
    }

    async forceDeleteReport(e, id) {
        e.stopPropagation();
        if (!confirm("ADMIN OVRRIDE: Force delete this report permanently?")) return;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/reports/${id}`, {
                method: "DELETE",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Report deleted forcefully", "success");
                if (this._selectedReportId === id) {
                    this._selectedReportId = null;
                    this._editingRom = null;
                }
                await this.fetchReports();
            } else {
                this.showStatus("Failed to delete", "error");
            }
        } catch (e) {
            this.showStatus("Network error", "error");
        }
    }

    render() {
        const isAdmin = this.role === "Admin";
        const selectedReport = this._reports.find((r) => r.id === this._selectedReportId);

        return html`
            <emunex-navbar></emunex-navbar>
            <div class="app-layout">
                <div class="panel" style="position: relative;">
                    <a href="/dev" class="back-link">Back</a>
                    <div class="panel-header" style="padding-left: 90px;">
                        <span>Open Reports (${this._reports.length})</span>
                        <button class="popout-btn btn-fit" style="margin: 0;" @click=${this.fetchReports}>
                            <span class="btn-edge"></span
                            ><span class="btn-front" style="padding: 4px 8px; font-size: 0.8rem;">Refresh</span>
                        </button>
                    </div>
                    <div class="panel-content">
                        ${this._loading ? html`<div class="empty-state">Loading...</div>` : ""}
                        ${!this._loading && this._reports.length === 0
                            ? html`<div class="empty-state">No open reports.<br />Queue is clear!</div>`
                            : ""}
                        ${this._reports.map((report) => {
                            const isMine = report.claimed_by === this.userid;
                            const isClaimedByOther = report.claimed_by && !isMine;
                            return html`
                                <div
                                    class="report-card ${this._selectedReportId === report.id ? "selected" : ""}"
                                    @click=${() => this.selectReport(report)}
                                >
                                    ${report.claimed_by
                                        ? html`
                                              <div
                                                  class="claim-badge ${isMine
                                                      ? "badge-claimed-me"
                                                      : "badge-claimed-other"}"
                                              >
                                                  ${isMine
                                                      ? "Claimed by You"
                                                      : `Claimed by: ${report.claimed_by_username}`}
                                              </div>
                                          `
                                        : html` <div class="claim-badge badge-open">Open - Unclaimed</div> `}

                                    <div class="report-meta">
                                        <span>${report.report_type}</span>
                                        <span>By: ${report.username}</span>
                                    </div>
                                    <div class="report-title">${report.rom_title}</div>
                                    <div class="report-desc">${report.description}</div>

                                    <div style="display: flex; gap: var(--spacing-sm); margin-top: var(--spacing-sm);">
                                        ${!report.claimed_by
                                            ? html`
                                                  <button
                                                      class="popout-btn btn-fit btn-info"
                                                      @click=${(e) => this.claimReport(e, report.id)}
                                                  >
                                                      <span class="btn-edge"></span
                                                      ><span class="btn-front" style="padding: 6px; font-size: 0.8rem;"
                                                          >Claim Report</span
                                                      >
                                                  </button>
                                              `
                                            : isMine || isAdmin
                                              ? html`
                                                    <button
                                                        class="popout-btn btn-fit"
                                                        @click=${(e) => this.unclaimReport(e, report.id)}
                                                    >
                                                        <span class="btn-edge"></span
                                                        ><span
                                                            class="btn-front"
                                                            style="padding: 6px; font-size: 0.8rem;"
                                                            >Unclaim</span
                                                        >
                                                    </button>
                                                `
                                              : ""}
                                        ${isAdmin
                                            ? html`
                                                  <button
                                                      class="popout-btn btn-fit btn-error"
                                                      @click=${(e) => this.forceDeleteReport(e, report.id)}
                                                  >
                                                      <span class="btn-edge"></span
                                                      ><span class="btn-front" style="padding: 6px; font-size: 0.8rem;"
                                                          >Delete</span
                                                      >
                                                  </button>
                                              `
                                            : ""}
                                    </div>
                                </div>
                            `;
                        })}
                    </div>
                </div>

                <div class="panel">
                    ${selectedReport
                        ? html`
                              <div class="panel-header">
                                  <span
                                      ><span style="color: var(--color-primary); margin-right: var(--spacing-xs);"
                                          >Viewing Ticket:</span
                                      >
                                      ${selectedReport.id.slice(-8)}</span
                                  >
                                  ${selectedReport.claimed_by === this.userid
                                      ? html`
                                            <button
                                                class="popout-btn btn-fit btn-info"
                                                style="margin: 0; --btn-color-primary: #10b981; --btn-color-dark: #059669;"
                                                @click=${() => this.resolveReport(selectedReport.id)}
                                            >
                                                <span class="btn-edge"></span
                                                ><span class="btn-front" style="padding: 6px 12px;"
                                                    >RESOLVE TICKET</span
                                                >
                                            </button>
                                        `
                                      : ""}
                              </div>
                              <div class="panel-content">
                                  ${this._status
                                      ? html`<div
                                            class="status-box ${this._statusType === "error"
                                                ? "status-error"
                                                : "status-success"}"
                                            style="margin-bottom: var(--spacing-md);"
                                            >${this._status}</div
                                        >`
                                      : ""}
                                  ${selectedReport.claimed_by === this.userid
                                      ? html`
                                            ${this._editingRom
                                                ? html`
                                                      <emunex-rom-edit-form
                                                          embedded
                                                          .rom=${this._editingRom}
                                                          .consoles=${this._consoles}
                                                          @saved=${this.fetchReports}
                                                      ></emunex-rom-edit-form>
                                                  `
                                                : html` <div class="empty-state">Loading ROM Data...</div> `}
                                        `
                                      : html`
                                            <div class="empty-state">
                                                <h3 style="margin-bottom: var(--spacing-sm);">Read Only</h3>
                                                <p>You must claim this report to edit its metadata and resolve it.</p>
                                            </div>
                                        `}
                              </div>
                          `
                        : html`
                              <div class="panel-content empty-state">
                                  Select a report from the queue<br />to view and resolve it.
                              </div>
                          `}
                </div>
            </div>
        `;
    }
}

customElements.define("emunex-reports-page", EmunexReportsPage);
