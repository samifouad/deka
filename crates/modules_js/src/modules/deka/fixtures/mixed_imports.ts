import def, { a as b } from "./mod"
import * as ns from "./ns"

export default function main() {
  return def(b, ns)
}
