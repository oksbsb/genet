import fs from 'fs'
import path from 'path'
import { promisify } from 'util'
import vm from 'vm'

const promiseReadFile = promisify(fs.readFile)
export default class Script {
  static async execute (file) {
    const code = await promiseReadFile(file, 'utf8')
    const wrapper =
      `(function(module, require, __filename, __dirname){ ${code} })`
    const options = {
      filename: file,
      displayErrors: true,
    }
    const dir = path.dirname(file)
    const func = vm.runInThisContext(wrapper, options)
    function isAvailable (name) {
      try {
        global.require.resolve(name)
        return true
      } catch (err) {
        return false
      }
    }
    function req (name) {
      if (name === 'genet') {
        return genet
      } else if (name.startsWith('./')) {
        const resolved = path.resolve(dir, name)
        if (isAvailable(resolved)) {
          return global.require(resolved)
        }
      } else {
        const resolved = path.resolve(dir, 'node_modules', name)
        if (isAvailable(resolved)) {
          return global.require(resolved)
        }
      }
      return global.require(name)
    }
    const module = {}
    func(module, req, file, dir)
    if (typeof module.exports !== 'function') {
      throw new TypeError('module.exports must be a function')
    }
    return module.exports
  }
}
