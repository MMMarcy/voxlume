variable "namespaces_definition" {
  description = <<EOF
    The definition of the namespaces we need for running aistudy-helper.

    At the moment this is provided with a list of objects that only require a name attribute.

  EOF
  type = list(object({
    name = string
  }))
  default = [
    {
      name = "ingestion"
    },
    {
      name = "metrics"
    },
    {
      name = "serving"
    },
    {
      name = "infra"
    }
  ]
}

variable "gcp_region" {
  description = "Region where all regional cloud resources will live."
  type        = string
}

variable "gcp_zone" {
  description = "Zone where all zonal cloud resources will live."
  type        = string
}

variable "gcp_project" {
  description = "Project id where everything will be set up."
  type        = string
}

variable "gcp_or_minikube" {
  description = "Whether the deployment targets gcp or minikube."
  type        = string
}
variable "postgres_password" {
  type        = string
  description = "Postgres password. If empty, a random one will be generated."
  default     = "password"
}
variable "postgres_docker_image_data" {
  type = object({
    image_name = string
    tag        = string
  })
  description = "Object with the information about the pgmq docker image."
  default = {
    image_name = "postgres"
    tag        = "17"
  }
}
