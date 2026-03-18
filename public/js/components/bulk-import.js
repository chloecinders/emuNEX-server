import { LitElement, html, css } from "lit";
import {
  baseTokens, devTokens, pageHostStyles, cardStyles,
  formStyles, buttonStyles, statusStyles,
} from "./shared-styles.js";

class EmunexBulkImport extends LitElement {
  static properties = {
    isUploading: { type: Boolean },
    statusMsg: { type: String },
    statusType: { type: String },
    _selectedFileName: { type: String },
  };

  static styles = [
    baseTokens, devTokens, pageHostStyles, cardStyles, formStyles, buttonStyles, statusStyles,
    css`
      .import-section {
        margin-top: var(--spacing-xl);
        padding-top: var(--spacing-lg);
        border-top: 1px solid var(--color-border);
      }
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
      <div class="import-section">
        <div class="section-hint">Bulk Import Games</div>
        <p class="role-label">Upload a unified ZIP file containing folders for each game. Each folder must have an <code>info.json</code> file.</p>
        
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
          </button>
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

customElements.define("emunex-bulk-import", EmunexBulkImport);
