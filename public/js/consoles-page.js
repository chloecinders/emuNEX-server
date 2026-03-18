import { LitElement, html, css } from "lit";
import {
  baseTokens, devTokens, pageHostStyles, cardStyles,
  formStyles, buttonStyles, statusStyles,
} from "./components/shared-styles.js";

class EmunexConsolesPage extends LitElement {
  static properties = {
    _status: { type: String, state: true },
    _statusType: { type: String, state: true },
    _loading: { type: Boolean, state: true },
    _colorHex: { type: String, state: true },
  };

  static styles = [
    baseTokens, devTokens, pageHostStyles, cardStyles, formStyles, buttonStyles, statusStyles,
    css`
      .color-picker-row {
        display: flex;
        gap: var(--spacing-sm);
        align-items: center;
      }
      .color-hex {
        flex: 1;
        font-family: monospace;
        text-transform: uppercase;
      }
      .color-preview {
        width: 42px;
        height: 42px;
        border-radius: var(--radius-sm);
        border: 2px solid var(--color-border);
        cursor: pointer;
        flex-shrink: 0;
        transition: transform 0.1s;
      }
      .color-preview:active {
        transform: scale(0.95);
      }
      input[type="color"] {
        display: none;
      }
    `,
  ];

  constructor() {
    super();
    this._status = "";
    this._statusType = "";
    this._loading = false;
    this._colorHex = "";
  }

  render() {
    return html`
      <div class="auth-container">
        <div class="auth-card">
          <header class="card-header">
            <a href="/dev" class="back-link">Back</a>
            <h1>emuNEX</h1>
            <p class="tagline">Add or Update Console</p>
          </header>

          <div class="content">
            <form id="consoleForm" @submit=${this._handleSubmit}>
              <div class="section-hint">Console Library</div>

              <div class="form-group">
                <label for="name">Console Name</label>
                <input type="text" id="name" name="name" placeholder="GBA" required />
              </div>

              <div class="form-group">
                <label for="card_color">Card Color Hex</label>
                <div class="color-picker-row">
                  <input
                    type="text"
                    id="card_color"
                    name="card_color"
                    placeholder="#ff0000"
                    class="color-hex"
                    .value=${this._colorHex}
                    @input=${this._handleHexInput}
                  />
                  <input
                    type="color"
                    id="hiddenColorPicker"
                    .value=${this._colorHex || "#eeeeee"}
                    @input=${this._handlePickerInput}
                  />
                  <div
                    class="color-preview"
                    style="background-color: ${this._colorHex || "#eeeeee"}"
                    @click=${() => this.renderRoot.querySelector("#hiddenColorPicker").click()}
                  ></div>
                </div>
              </div>

              <button type="submit" class="popout-btn" ?disabled=${this._loading} style="margin-top: var(--spacing-lg)">
                <span class="btn-edge"></span>
                <span class="btn-front">${this._loading ? "Saving…" : "Save Console"}</span>
              </button>
            </form>

            ${this._status ? html`
              <div class="status-box ${this._statusType === "error" ? "status-error" : "status-success"}">
                ${this._status}
              </div>
            ` : ""}
          </div>
        </div>
      </div>
    `;
  }

  _handleHexInput(e) {
    const hex = e.target.value;
    if (/^#[0-9A-F]{6}$/i.test(hex)) {
      this._colorHex = hex.toUpperCase();
    }
  }

  _handlePickerInput(e) {
    this._colorHex = e.target.value.toUpperCase();
  }

  async _handleSubmit(e) {
    e.preventDefault();
    const token = localStorage.getItem("token");
    this._loading = true;
    this._status = "Saving console…";
    this._statusType = "success";

    const form = this.renderRoot.querySelector("#consoleForm");
    const formData = new FormData(form);

    try {
      const response = await fetch("/api/v1/consoles", {
        method: "POST",
        body: formData,
        headers: { Authorization: token },
      });

      const result = await response.json();
      if (response.ok) {
        this._status = `Success! Console ID: ${result.data}`;
        this._statusType = "success";
        form.reset();
        this._colorHex = "";
      } else {
        this._status = `Error: ${result.error || "Save failed"}`;
        this._statusType = "error";
      }
    } catch (err) {
      this._status = "A network error occurred.";
      this._statusType = "error";
    } finally {
      this._loading = false;
    }
  }
}

customElements.define("emunex-consoles-page", EmunexConsolesPage);
