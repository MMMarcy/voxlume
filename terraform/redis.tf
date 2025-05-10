locals {
  redis_port = 6379
}


resource "docker_container" "redis_container" {
  name = "redis_container"

  image        = docker_image.redis_docker_image.image_id
  user         = "root"
  restart      = "always"
  network_mode = "host"
  ports {
    internal = local.redis_port
    external = local.redis_port
    protocol = "TCP"
  }
  command = ["redis-server", "--requirepass", local.redis_password]
  lifecycle {
    replace_triggered_by = [
      docker_image.redis_docker_image.image_id
    ]
  }
}

