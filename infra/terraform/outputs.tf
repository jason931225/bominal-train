output "artifact_registry_repo" {
  description = "Artifact Registry repository name."
  value       = google_artifact_registry_repository.bominal.name
}

output "github_actions_service_account_email" {
  description = "Service account email that GitHub Actions impersonates."
  value       = google_service_account.github_actions.email
}

output "workload_identity_provider" {
  description = "Workload Identity Provider resource name to use in GitHub Actions auth."
  value       = google_iam_workload_identity_pool_provider.github.name
}

output "deploy_instance_external_ip" {
  description = "Deploy VM external IP (only when enable_vm=true)."
  value       = var.enable_vm ? google_compute_address.bominal_ip[0].address : null
}

