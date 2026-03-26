import { LitElement, html, css } from "lit";
import {
  baseTokens, devTokens, pageHostStyles, cardStyles,
  formStyles, buttonStyles, statusStyles,
} from "./components/shared-styles.js";

class EmunexBulkImportPage extends LitElement {
  static properties = {
    isUploading: { type: Boolean },
    statusMsg: { type: String },
    statusType: { type: String },
    _selectedFileName: { type: String },
  };

  static styles = [
    baseTokens, devTokens, pageHostStyles, cardStyles, formStyles, buttonStyles, statusStyles,
    css`
      .max-1200 { max-width: 1200px; }
      .file-drop {
        border: 2px dashed var(--color-border);
        border-radius: var(--radius-md);
        padding: var(--spacing-xl);
        text-align: center;
        margin-bottom: var(--spacing-md);
        background: var(--color-surface-variant);
        cursor: pointer;
        transition: border-color 0.2s;
        display: block;
      }
      .file-drop:hover {
        border-color: var(--color-primary);
      }
      .games-list {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-md);
        margin-top: var(--spacing-md);
        max-height: 60vh;
        overflow-y: auto;
        padding-right: var(--spacing-sm);
      }
      .game-item {
        display: flex;
        gap: var(--spacing-md);
        background: var(--color-surface);
        padding: var(--spacing-md);
        border-radius: var(--radius-md);
        border: 1px solid var(--color-border);
        align-items: center;
      }
      .game-cover {
        width: 60px;
        height: 80px;
        object-fit: cover;
        border-radius: var(--radius-sm);
        background: var(--color-surface-variant);
      }
      .game-details {
        flex: 1;
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-sm);
      }
      .game-details .form-group {
        margin: 0;
      }
      .game-status {
        font-size: 0.85rem;
        padding: 4px 8px;
        border-radius: var(--radius-sm);
        background: var(--color-surface-variant);
      }
      .game-status.success { color: #4ade80; }
      .game-status.error { color: #f87171; }
      .actions {
        display: flex;
        justify-content: flex-end;
        margin-top: var(--spacing-md);
      }
    `
  ];

  constructor() {
    super();
    this.isUploading = false;
    this.statusMsg = "";
    this.statusType = "";
    this._selectedFileName = "";
  }

  render() {
    return html`
      <div class="auth-container max-1200">
        <div class="auth-card" style="width: 100%">
          <header class="card-header">
            <a href="/roms" class="back-link">Back</a>
            <h1>emuNEX</h1>
            <p class="tagline">Bulk Import Games</p>
          </header>

          <div class="content">
            <p class="role-label" style="text-align: center; margin-bottom: var(--spacing-lg)">
              Upload a unified ZIP file containing folders for each game. Each folder must have an <code>info.json</code> file.
            </p>
            
            <label class="file-drop" for="zip-upload">
              <strong>${this._selectedFileName ? `Selected: ${this._selectedFileName}` : "Click to select a .ZIP file"}</strong>
              <input type="file" id="zip-upload" accept=".zip" style="display: none" @change=${this._handleFileSelect} ?disabled=${this.isUploading} />
            </label>

            ${this.statusMsg ? html`
              <div class="status-box status-${this.statusType}">
                ${this.statusMsg}
              </div>
            ` : ""}

            <div class="actions">
              <button class="popout-btn" @click=${this._uploadZip} ?disabled=${this.isUploading || !this._selectedFileName}>
                <span class="btn-edge"></span>
                <span class="btn-front">${this.isUploading ? 'Uploading & Processing...' : 'Upload & Process ZIP'}</span>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  _handleFileSelect(e) {
    const file = e.target.files[0];
    if (file) {
      this._selectedFileName = file.name;
    } else {
      this._selectedFileName = "";
    }
  }

  async _uploadZip() {
    const inputElement = this.renderRoot.querySelector("#zip-upload");
    const file = inputElement.files[0];
    if (!file) return;

    this.isUploading = true;
    this.statusMsg = "Uploading to server for processing... This may take a few minutes for large files.";
    this.statusType = "success";

    const formData = new FormData();
    formData.append("zip_file", file);

    try {
      const response = await fetch("/api/v1/roms/bulk_upload", {
        method: "POST",
        body: formData,
        headers: { Authorization: localStorage.getItem("token") },
      });

      const result = await response.json();

      if (response.ok) {
        this.statusMsg = result.data || "Processing completed successfully!";
        this.statusType = "success";
        this._selectedFileName = "";
        inputElement.value = "";
      } else {
        this.statusMsg = "Server error: " + (result.error || "Failed to process ZIP");
        this.statusType = "error";
      }
    } catch (err) {
      console.error(err);
      this.statusMsg = "Network error occurred during upload.";
      this.statusType = "error";
    } finally {
      this.isUploading = false;
    }
  }
}

customElements.define("emunex-bulk-import-page", EmunexBulkImportPage);
