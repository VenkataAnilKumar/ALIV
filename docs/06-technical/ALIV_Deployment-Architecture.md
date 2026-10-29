# Deployment Architecture
## Desktop, Firm Server and private cloud

**Version:** 1.0 | **Date:** October 29, 2026 | **Status:** Desktop as built; Firm Server and private cloud planned (R4)

---

## 1. Deployment options

| Option | Who | Data location | Models | Release |
|---|---|---|---|---|
| **Desktop** | Solo, small teams | Each user's computer | Local on the device; cloud for Hybrid matters | ✅ R0 |
| **Desktop + Firm Server** | Firms of 10–500 | Firm's server (on-premises) | Server GPU models + desktop local + cloud by policy | R4 |
| **Private cloud** | Firms preferring their own cloud tenancy | Firm's VPC (AWS, Azure, GCP) | Self-hosted models in the VPC; cloud APIs by policy | R4+ |

## 2. Desktop (today)

```
┌──────────── User's computer ─────────────┐
│ ALIV.app / ALIV.exe / AppImage, .deb      │
│  ├─ Rust core + vault plugin              │
│  ├─ Web UI (bundled)                      │
│  ├─ llama.cpp / MLX engines               │
│  └─ app data: vault.db, models/, logs/    │
│ System: Tesseract, poppler (for OCR)      │
│ Documents: local disk or mapped share     │
└───────────────────────────────────────────┘
```

| Item | Detail |
|---|---|
| **Installers** | macOS `.dmg` (universal), Windows NSIS `.exe`, Linux AppImage and `.deb`. Built by the Tauri bundler (`tauri.*.conf.json`) |
| **Signing** | macOS Developer ID + notarisation; Windows Authenticode (`sign.ps1`). Required for GA |
| **Updates** | Tauri updater with a signed `latest.json` (template in `src-tauri/latest.json.template`) from ALIV's update server; firms can disable or mirror it |
| **OCR dependencies** | Tesseract and poppler are detected at runtime. Missing tools are reported in Vault health. Bundling is planned for GA (macOS/Windows) |
| **Models** | Downloaded on first run from the model catalogue, or side-loaded (GGUF) for air-gapped sites |
| **Air-gapped install** | Supported: offline installer, side-loaded models, updater disabled, Local only matters |
| **Managed install** | MDM (Intune, Jamf) deploys the installer and a policy file: defaults (Local only forced, allowed providers, updater URL) |
| **Backup** | `vault.db` can be backed up with the firm's existing tools while the app is closed; documents are backed up as usual |

## 3. Firm Server (R4)

```
 Desktops (ALIV app) ──HTTPS + OIDC──▶ ┌──────────── Firm Server (on-prem) ─────────────┐
                                       │ API (axum)  · Agent runtime · Scheduler        │
 Word add-in ─────────────────────────▶│ Vault service (same crate as desktop)          │
                                       │ DB: PostgreSQL (or SQLite for < 20 users)      │
                                       │ Passage index + vectors · Audit chain          │
                                       │ Model gateway ──▶ GPU model server (vLLM/llama)│
                                       │              └─▶ cloud APIs (Hybrid matters)   │
                                       └───┬───────────────────────────┬────────────────┘
                                           │ read-only mounts          │ backups
                                     Document shares / DMS        Firm backup system
```

| Concern | Design |
|---|---|
| **Identity** | OIDC/SAML (Entra ID, Okta, Google). Roles: admin, member, viewer. Ethical walls per matter |
| **Packaging** | Container images (Docker/Podman) and a Linux package; Helm chart for Kubernetes |
| **Sizing (guide)** | 20 users: 16 vCPU, 64 GB RAM, 1× 48 GB GPU for 70B-class Q4, 1 TB SSD |
| **High availability** | Single node with backups (default); active/passive with shared Postgres for larger firms |
| **Monitors** | Run server-side, so they work when laptops are closed |
| **Desktop offline mode** | Matters can be checked out to a desktop vault (Local only) and synced back; audit entries merge into the server chain |
| **Admin** | Users, matter access, policy defaults, provider keys, spend limits, audit export, health |

## 4. Private cloud (R4+)

Same images as the Firm Server, deployed into the firm's own tenancy:
- storage on the firm's encrypted volumes (KMS keys owned by the firm);
- models self-hosted in the VPC, or the cloud provider's private endpoints (e.g. Bedrock, Azure OpenAI) treated as "cloud" routes subject to policy;
- no ALIV-operated infrastructure in the data path. ALIV ships images and updates only.

## 5. Environments and release

| Environment | Purpose |
|---|---|
| Dev | Contributors (`make dev`, browser demo build with `demo-service`) |
| CI | GitHub Actions: lint, types, web tests, Rust tests (desktop, CLI, vault with OCR tools) |
| Beta channel | Signed builds for pilot firms |
| Stable channel | General availability |

Release steps: version bump → CI green → signed builds → beta → stable. Rollback = re-publish the previous `latest.json`. Vault migrations are additive, so an older app can still open the vault read-only.
