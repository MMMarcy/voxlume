terraform {
  required_version = ">=1.9"

  required_providers {
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

