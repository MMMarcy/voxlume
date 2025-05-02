locals {
  postgres_port = 5432
}


resource "docker_container" "postgres_container" {
  name = "postgres_container"

  image = docker_image.postgres_docker_image.image_id
  env = toset([
    "POSTGRES_USER=postgres",
    "POSTGRES_PASSWORD=${local.postgres_password}",
    "POSTGRES_DB=voxlume"
    ]
  )
  user = "root"
  mounts {
    target = "/var/lib/postgresql/data"
    source = "${abspath(path.root)}/../docker_volumes/postgres"
    type   = "bind"
  }
  restart      = "always"
  network_mode = "host"
  ports {
    internal = local.postgres_port
    external = local.postgres_port
    protocol = "TCP"
  }
  lifecycle {
    replace_triggered_by = [
      docker_image.postgres_docker_image.image_id
    ]
  }
}

