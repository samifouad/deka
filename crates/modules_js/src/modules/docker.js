/**
 * deka/docker - Docker operations
 */

function listContainers() {
  return Deno.core.ops.op_docker_list_containers()
}

function createContainer(config) {
  return Deno.core.ops.op_docker_create_container(config || {})
}

globalThis.__dekaDocker = { listContainers, createContainer }

export { listContainers, createContainer }
