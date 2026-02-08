variable "project_id" {
  type        = string
  description = "GCP project id (e.g. bominal)."
}

variable "region" {
  type        = string
  description = "GCP region for Artifact Registry + regional resources (e.g. us-central1)."
  default     = "us-central1"
}

variable "zone" {
  type        = string
  description = "GCP zone for the VM (e.g. us-central1-a)."
  default     = "us-central1-a"
}

variable "github_owner" {
  type        = string
  description = "GitHub org/user that owns the repository (for Workload Identity Federation)."
}

variable "github_repo" {
  type        = string
  description = "GitHub repository name (for Workload Identity Federation)."
}

variable "artifact_repo_id" {
  type        = string
  description = "Artifact Registry repository id for docker images."
  default     = "bominal"
}

variable "deploy_instance_name" {
  type        = string
  description = "Compute Engine instance name."
  default     = "bominal-deploy"
}

variable "enable_vm" {
  type        = bool
  description = "Whether to manage the deploy VM + firewall via Terraform."
  default     = false
}

