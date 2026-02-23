locals {
  github_repo_full = "${var.github_owner}/${var.github_repo}"

  # Lock GitHub Actions -> GCP impersonation to this one repo.
  wif_attribute_condition = "assertion.repository==\"${local.github_repo_full}\""
}

# -----------------------------------------------------------------------------
# GitHub Actions Workload Identity Federation (OIDC)
# -----------------------------------------------------------------------------

resource "google_iam_workload_identity_pool" "github" {
  workload_identity_pool_id = "github-pool"
  display_name              = "GitHub Actions Pool"
  description               = "OIDC identities from GitHub Actions"
}

resource "google_iam_workload_identity_pool_provider" "github" {
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github-provider"
  display_name                       = "GitHub Actions Provider"

  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }

  attribute_mapping = {
    "google.subject"       = "assertion.sub"
    "attribute.repository" = "assertion.repository"
    "attribute.ref"        = "assertion.ref"
    "attribute.actor"      = "assertion.actor"
  }

  attribute_condition = local.wif_attribute_condition
}

# Service account GitHub Actions will impersonate.
resource "google_service_account" "github_actions" {
  account_id   = "github-actions"
  display_name = "GitHub Actions (bominal)"
  description  = "CI/CD service account for bominal (Workload Identity Federation)"
}

# Allow GitHub Actions (this repo) to impersonate the service account.
resource "google_service_account_iam_binding" "github_actions_wif" {
  service_account_id = google_service_account.github_actions.name
  role               = "roles/iam.workloadIdentityUser"

  members = [
    "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${local.github_repo_full}",
  ]
}

# -----------------------------------------------------------------------------
# Optional: Single VM deploy target (free-tier friendly)
# -----------------------------------------------------------------------------

resource "google_compute_address" "bominal_ip" {
  count  = var.enable_vm ? 1 : 0
  name   = "bominal-ip"
  region = var.region
}

resource "google_compute_firewall" "bominal_http_https" {
  count   = var.enable_vm ? 1 : 0
  name    = "bominal-http-https"
  network = "default"

  allow {
    protocol = "tcp"
    ports    = ["80", "443"]
  }

  # Cloudflare proxying still requires origin reachability. Tighten this to
  # Cloudflare IP ranges once you automate updates.
  source_ranges = ["0.0.0.0/0"]

  target_tags = ["bominal"]
}

resource "google_compute_instance" "deploy" {
  count        = var.enable_vm ? 1 : 0
  name         = var.deploy_instance_name
  machine_type = "e2-micro"
  zone         = var.zone

  tags = ["bominal"]

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-12"
      size  = 20
      type  = "pd-balanced"
    }
  }

  network_interface {
    network = "default"

    access_config {
      nat_ip = google_compute_address.bominal_ip[0].address
    }
  }

  metadata = {
    enable-oslogin = "TRUE"
  }
}
