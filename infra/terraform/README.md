# bominal Terraform (GCP)

This folder is an IaC baseline for the GCP pieces used by `bominal`:

- GitHub Actions Workload Identity Federation (OIDC) + service account
- Minimal GitHub Actions IAM for deploy publish workflow
- Optional: single VM target (e2-micro) + firewall + static IP

## Quick Start

```bash
cd infra/terraform
terraform init

terraform apply \
  -var="project_id=bominal" \
  -var="github_owner=YOUR_GITHUB_ORG" \
  -var="github_repo=YOUR_REPO_NAME"
```

To also manage the VM/firewall/IP:

```bash
terraform apply \
  -var="project_id=bominal" \
  -var="github_owner=YOUR_GITHUB_ORG" \
  -var="github_repo=YOUR_REPO_NAME" \
  -var="enable_vm=true"
```

## Notes

- If resources already exist (likely), import them before `apply` to avoid conflicts.
- `enable_vm` is `false` by default to prevent accidental VM creation.
