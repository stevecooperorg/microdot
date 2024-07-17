variable "CURRENT_DOCKER_SEMVER_TAG" {
  # designed to be overridden by an environment variable
  # e.g., export $$(cat .env | xargs) && docker buildx bake
  default = "latest"
}

group "default" {
  targets = ["microdot", "live-server"]
}

target "microdot" {
  context    = "."
  dockerfile = "microdot.Dockerfile"
  tags = ["stevecooperorg/microdot:${CURRENT_DOCKER_SEMVER_TAG}", "stevecooperorg/microdot:latest"]
  cache-from = ["type=registry,ref=stevecooperorg/microdot:cache"]
  cache-to = ["type=inline"]
}

target "live-server" {
  context    = "."
  dockerfile = "live-server.Dockerfile"
  tags = ["stevecooperorg/live-server:${CURRENT_DOCKER_SEMVER_TAG}", "stevecooperorg/live-server:latest"]
  cache-from = ["type=registry,ref=stevecooperorg/live-server:cache"]
  cache-to = ["type=inline"]
}