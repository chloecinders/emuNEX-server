import { LitElement, css, html } from "lit";
import { baseTokens, buttonStyles, formStyles, statusStyles, uploadZoneStyles } from "./shared-styles.js";
import "../libs/spark-md5.min.js";

export class EmunexRomEditForm extends LitElement {
    static properties = {
        rom: { type: Object },
        consoles: { type: Array },
        _editRomFileName: { type: String, state: true },
        _editImageFileName: { type: String, state: true },
        _dragoverRom: { type: Boolean, state: true },
        _dragoverImage: { type: Boolean, state: true },
        _previewImageUrl: { type: String, state: true },
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _zipped: { type: Boolean, state: true },
        _zippedEntry: { type: String, state: true },
        embedded: { type: Boolean },
    };

    static styles = [
        baseTokens,
        formStyles,
        buttonStyles,
        statusStyles,
        uploadZoneStyles,
        css`
            :host {
                display: block;
                box-sizing: border-box;
            }
            *,
            *::before,
            *::after {
                box-sizing: inherit;
            }
            .grid-3 {
                display: grid;
                grid-template-columns: 1fr 1fr 1fr;
                gap: var(--spacing-md);
            }
            .grid-2 {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: var(--spacing-md);
            }
            .section-hint {
                font-size: 0.85rem;
                font-weight: 700;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                color: var(--color-primary);
                margin-bottom: var(--spacing-sm);
                margin-top: var(--spacing-lg);
            }
            .section-hint:first-child {
                margin-top: 0;
            }
        `,
    ];

    constructor() {
        super();
        this.rom = {};
        this.consoles = [];
        this.embedded = false;
        this._editRomFileName = "";
        this._editImageFileName = "";
        this._dragoverRom = false;
        this._dragoverImage = false;
        this._previewImageUrl = "";
        this._status = "";
        this._statusType = "";
        this._zipped = false;
        this._zippedEntry = "";
    }

    updated(changedProps) {
        if (changedProps.has("rom") && this.rom) {
            this._editRomFileName = "";
            this._editImageFileName = "";
            this._zipped = this.rom.zipped || false;
            this._zippedEntry = this.rom.zipped_entry || "";
            this._previewImageUrl = this.rom.image_path
                ? this.rom.image_path.startsWith("blob:")
                    ? this.rom.image_path
                    : `/storage${this.rom.image_path}`
                : "";

            setTimeout(() => {
                const sel = this.renderRoot.querySelector("#edit-console");
                if (sel && this.rom.console) sel.value = this.rom.console;
            }, 0);
        }
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 3000);
    }

    async submitEdit(e) {
        e.preventDefault();
        if (!this.rom || !this.rom.id) return;
        const id = this.rom.id;
        const root = this.renderRoot;

        const reqData = {
            title: root.querySelector("#edit-title").value,
            realname: root.querySelector("#edit-realname").value || null,
            console: root.querySelector("#edit-console").value,
            category: root.querySelector("#edit-category").value,
            release_year: parseInt(root.querySelector("#edit-year").value) || null,
            region: root.querySelector("#edit-region").value || null,
            serial: root.querySelector("#edit-serial").value || null,
            md5_hash: root.querySelector("#edit-md5").value || null,
            zipped: this._zipped,
            zipped_entry: this._zipped ? this._zippedEntry : null,
            multi_disc_disclaimer: root.querySelector("#edit-multi-disc-disclaimer").checked,
        };

        const token = (await cookieStore.get("token"))?.value;
        try {
            const res = await fetch(`/api/v1/roms/${encodeURIComponent(id)}`, {
                method: "PUT",
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify(reqData),
            });

            if (!res.ok) throw new Error((await res.json()).error || "Metadata update failed");

            const romInput = root.querySelector("#edit-rom");
            if (romInput.files.length) {
                const romFile = romInput.files[0];
                this.showStatus("Computing MD5 for new ROM...", "success");
                
                const md5 = await new Promise((resolve, reject) => {
                    const chunkSize = 2097152;
                    const chunks = Math.ceil(romFile.size / chunkSize);
                    let currentChunk = 0;
                    const spark = new window.SparkMD5.ArrayBuffer();
                    const fileReader = new FileReader();
                    fileReader.onload = (e) => {
                        spark.append(e.target.result);
                        currentChunk++;
                        if (currentChunk < chunks) loadNext();
                        else resolve(spark.end());
                    };
                    fileReader.onerror = () => reject("File reading failed");
                    const loadNext = () => {
                        const start = currentChunk * chunkSize;
                        const end = start + chunkSize >= romFile.size ? romFile.size : start + chunkSize;
                        fileReader.readAsArrayBuffer(romFile.slice(start, end));
                    };
                    loadNext();
                });

                let fileExt = romFile.name.split('.').pop().toLowerCase() || "bin";
                if (this._zipped) fileExt = "zip";

                this.showStatus("Requesting secure upload URL...", "success");
                const signRes = await fetch(`/api/v1/roms/${encodeURIComponent(id)}/file/sign`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json", Authorization: token },
                    body: JSON.stringify({
                        md5_hash: md5,
                        file_extension: fileExt
                    })
                });
                const signJson = await signRes.json();
                if (!signRes.ok) throw new Error(signJson.error || "Failed to sign upload URL");

                this.showStatus("Uploading ROM directly to S3...", "success");
                const putRes = await fetch(signJson.data.upload_url, {
                    method: "PUT",
                    body: romFile,
                    headers: { "Content-Type": "application/octet-stream" }
                });
                if (!putRes.ok) throw new Error("S3 upload failed");

                this.showStatus("Finalizing file update with server...", "success");
                const d = new FormData();
                d.append("md5_hash", md5);
                d.append("file_extension", fileExt);
                d.append("file_size_bytes", romFile.size.toString());
                const r2 = await fetch(`/api/v1/roms/${encodeURIComponent(id)}/file`, {
                    method: "POST",
                    headers: { Authorization: token },
                    body: d,
                });
                if (!r2.ok) throw new Error("ROM file update failed");
            }

            const imgInput = root.querySelector("#edit-image");
            if (imgInput.files.length) {
                const d = new FormData();
                d.append("image", imgInput.files[0]);
                const r3 = await fetch(`/api/v1/roms/${encodeURIComponent(id)}/image`, {
                    method: "POST",
                    headers: { Authorization: token },
                    body: d,
                });
                if (!r3.ok) throw new Error("Cover image update failed");
            }

            this.showStatus("ROM updated successfully!", "success");
            this.dispatchEvent(new CustomEvent("saved", { detail: { id }, bubbles: true, composed: true }));
        } catch (err) {
            this.showStatus(err.message || "Update failed.", "error");
        }
    }

    _dragEnter(e, z) {
        e.preventDefault();
        if (z === "rom") this._dragoverRom = true;
        else this._dragoverImage = true;
    }
    _dragLeave(e, z) {
        e.preventDefault();
        if (z === "rom") this._dragoverRom = false;
        else this._dragoverImage = false;
    }
    _drop(e, id, prop) {
        e.preventDefault();
        this._dragoverRom = false;
        this._dragoverImage = false;
        const input = this.renderRoot.querySelector("#" + id);
        input.files = e.dataTransfer.files;
        if (input.files.length) {
            const fileName = input.files[0].name;
            this[`_${prop}`] = fileName;
            if (prop === "editRomFileName") {
                if (fileName.toLowerCase().endsWith(".zip")) {
                    this._zipped = true;
                }
            }
            if (prop === "editImageFileName") {
                if (this._previewImageUrl && this._previewImageUrl.startsWith("blob:"))
                    URL.revokeObjectURL(this._previewImageUrl);
                this._previewImageUrl = URL.createObjectURL(input.files[0]);
            }
        }
    }
    _fileChange(e, prop) {
        if (e.target.files.length) {
            const fileName = e.target.files[0].name;
            this[`_${prop}`] = fileName;
            if (prop === "editRomFileName") {
                if (fileName.toLowerCase().endsWith(".zip")) {
                    this._zipped = true;
                }
            }
            if (prop === "editImageFileName") {
                if (this._previewImageUrl && this._previewImageUrl.startsWith("blob:"))
                    URL.revokeObjectURL(this._previewImageUrl);
                this._previewImageUrl = URL.createObjectURL(e.target.files[0]);
            }
        }
    }

    render() {
        if (!this.rom || !this.rom.id)
            return html`
                <div style="padding: 1rem; text-align: center; color: var(--color-text-muted);">No ROM selected...</div>
            `;
        return html`
            <form id="editForm" @submit=${this.submitEdit}>
                <div class="form-group">
                    <label>ID (Read-only)</label>
                    <input type="text" .value=${this.rom.id} readonly style="opacity: 0.7; cursor: not-allowed" />
                </div>
                <div class="grid-2">
                    <div class="form-group">
                        <label>Title</label>
                        <input type="text" id="edit-title" .value=${this.rom.title} required />
                    </div>
                    <div class="form-group">
                        <label>Real Name</label>
                        <input type="text" id="edit-realname" .value=${this.rom.realname || ""} />
                    </div>
                </div>

                <div class="form-group">
                    <label>MD5 Hash (Hex)</label>
                    <input type="text" id="edit-md5" .value=${this.rom.md5_hash || ""} placeholder="1a2b3c4d..." />
                </div>

                <div class="grid-3">
                    <div class="form-group">
                        <label>Console</label>
                        <select id="edit-console" required>
                            <option value="">select console</option>
                            ${this.consoles.map(
                                (c) => html`
                                    <option value=${c.name}>${c.name.toUpperCase()}</option>
                                `,
                            )}
                        </select>
                    </div>
                    <div class="form-group">
                        <label>Category</label>
                        <input type="text" id="edit-category" .value=${this.rom.category || ""} required />
                    </div>
                    <div class="form-group">
                        <label>Serial Number</label>
                        <input type="text" id="edit-serial" .value=${this.rom.serial || ""} />
                    </div>
                </div>

                <div class="grid-2">
                    <div class="form-group">
                        <label>Region</label>
                        <input type="text" id="edit-region" .value=${this.rom.region || ""} />
                    </div>
                    <div class="form-group">
                        <label>Release Year</label>
                        <input type="number" id="edit-year" .value=${this.rom.release_year || ""} />
                    </div>
                </div>

                <div
                    class="form-group"
                    style="display: flex; align-items: center; gap: 8px; margin-top: var(--spacing-sm);">
                    <input
                        type="checkbox"
                        id="edit-multi-disc-disclaimer"
                        ?checked=${this.rom.multi_disc_disclaimer}
                        style="width: auto; margin: 0;" />
                    <label for="edit-multi-disc-disclaimer" style="margin: 0; font-weight: 700; cursor: pointer;">
                        Display Multi-Disc Swap Disclaimer
                    </label>
                </div>

                <div
                    class="form-group"
                    style="display: flex; align-items: center; gap: 8px; margin-top: var(--spacing-sm);">
                    <input
                        type="checkbox"
                        id="edit-zipped"
                        ?checked=${this._zipped}
                        style="width: auto; margin: 0;"
                        @change=${(e) => (this._zipped = e.target.checked)} />
                    <label for="edit-zipped" style="margin: 0; font-weight: 700; cursor: pointer;">Zipped ROM</label>
                </div>

                ${this._zipped
                    ? html`
                          <div class="form-group">
                              <label for="edit-zipped-entry">Starting ROM Entry File (inside ZIP, e.g. game.cue)</label>
                              <input
                                  type="text"
                                  id="edit-zipped-entry"
                                  placeholder="game.cue"
                                  .value=${this._zippedEntry || ""}
                                  @input=${(e) => (this._zippedEntry = e.target.value)}
                                  required />
                          </div>
                      `
                    : ""}

                <div class="section-hint">Update Assets (Leave blank to keep current)</div>

                <div class="form-group">
                    <label>Replace ROM File</label>
                    <label
                        class="upload-zone ${this._dragoverRom ? "dragover" : ""}"
                        @dragenter=${(e) => this._dragEnter(e, "rom")}
                        @dragover=${(e) => this._dragEnter(e, "rom")}
                        @dragleave=${(e) => this._dragLeave(e, "rom")}
                        @drop=${(e) => this._drop(e, "edit-rom", "editRomFileName")}>
                        <div class="upload-icon">↑</div>
                        <div class="upload-info">
                            <div class="upload-text">Upload new ROM</div>
                            <div class="file-name">${this._editRomFileName}</div>
                        </div>
                        <input
                            type="file"
                            id="edit-rom"
                            style="display: none"
                            @change=${(e) => this._fileChange(e, "editRomFileName")} />
                    </label>
                </div>

                <div class="form-group">
                    <label>Replace Cover Image</label>
                    <div style="display: flex; gap: var(--spacing-md); align-items: start;">
                        ${this._previewImageUrl
                            ? html`
                                  <img
                                      src=${this._previewImageUrl}
                                      style="width: 100px; height: 140px; object-fit: cover; border-radius: var(--radius-sm); border: 1px solid var(--color-border); flex-shrink: 0;" />
                              `
                            : ""}
                        <label
                            class="upload-zone ${this._dragoverImage ? "dragover" : ""}"
                            style="flex: 1; min-height: 140px; margin: 0;"
                            @dragenter=${(e) => this._dragEnter(e, "image")}
                            @dragover=${(e) => this._dragEnter(e, "image")}
                            @dragleave=${(e) => this._dragLeave(e, "image")}
                            @drop=${(e) => this._drop(e, "edit-image", "editImageFileName")}>
                            <div class="upload-icon">↑</div>
                            <div class="upload-info">
                                <div class="upload-text">Upload new cover</div>
                                <div class="file-name">${this._editImageFileName}</div>
                            </div>
                            <input
                                type="file"
                                id="edit-image"
                                style="display: none"
                                @change=${(e) => this._fileChange(e, "editImageFileName")} />
                        </label>
                    </div>
                </div>

                ${this._status
                    ? html`
                          <div
                              class="status-box ${this._statusType === "error" ? "status-error" : "status-success"}"
                              style="margin-top: var(--spacing-md);">
                              ${this._status}
                          </div>
                      `
                    : ""}

                <div style="display: flex; gap: var(--spacing-md); margin-top: var(--spacing-lg)">
                    <button type="submit" class="popout-btn btn-fit" style="flex: 1">
                        <span class="btn-edge"></span>
                        <span class="btn-front" style="padding: 10px">Save Changes</span>
                    </button>
                    ${!this.embedded
                        ? html`
                              <button
                                  type="button"
                                  class="popout-btn btn-fit btn-cancel"
                                  style="flex: 1"
                                  @click=${() =>
                                      this.dispatchEvent(new CustomEvent("cancel", { bubbles: true, composed: true }))}>
                                  <span class="btn-edge"></span>
                                  <span class="btn-front" style="padding: 10px">Cancel</span>
                              </button>
                          `
                        : ""}
                </div>
            </form>
        `;
    }
}

customElements.define("emunex-rom-edit-form", EmunexRomEditForm);
