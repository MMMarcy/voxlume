terraform {
  required_version = ">=1.9"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "6.1.0"
    }

    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.32.0"
    }

    docker = {
      source  = "kreuzwerker/docker"
      version = "3.0.2"
    }

    random = {
      source  = "hashicorp/random"
      version = "3.6.3"
    }

    null = {
      source  = "hashicorp/null"
      version = "3.2.3"
    }
  }
}

provider "google" {
  project = var.gcp_project
  region  = var.gcp_region
}

provider "kubernetes" {
  alias       = "gke"
  config_path = "~/.kube/config"
}

provider "docker" {
  alias = "local"
}

