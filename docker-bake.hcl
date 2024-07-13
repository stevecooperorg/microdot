variable "CURRENT_DOCKER_SEMVER_TAG" {
  # designed to be overridden by an environment variable
  # e.g., CURRENT_DOCKER_SEMVER_TAG=$(cat CURRENT_DOCKER_SEMVER_TAG) docker buildx bake
  default = "latest"
}

variable "rust_image" {
  default = "docker-image://rust:1-bookworm"
}

variable "runtime_image" {
  default = "docker-image://debian:bookworm-slim"
}

group "default" {
  targets = ["microdot"]
}

target "microdot" {
  context    = "."
  dockerfile = "Dockerfile"
  tags = ["stevecooperorg/microdot:${CURRENT_DOCKER_SEMVER_TAG}", "stevecooperorg/microdot:latest"]
  cache-from = ["type=registry,ref=stevecooperorg/microdot:cache"]
  cache-to = ["type=inline"]
  contexts = {
    runtime_image = runtime_image
    rust_image    = rust_image
  }
}