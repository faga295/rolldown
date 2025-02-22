import { InputOptions as RollupInputOptions } from '../rollup-types'
import { InputOptions as BindingInputOptions } from '@rolldown/node-binding'
import path from 'path'
import { arraify, normalizePluginOption } from '../utils'
import { createBuildPluginAdapter } from './create-build-plugin-adapter'

export interface InputOptions extends RollupInputOptions {
  // --- NotGoingToSupports

  /**
   * @deprecated
   * Rolldown use SWC to parse code
   */
  acorn?: never
  /**
   * @deprecated
   * Rolldown use SWC to parse code
   */
  acornInjectPlugins?: never
  /**
   * @deprecated
   * TODO: Rolldown might supports this in a long term. Need to investigate.
   */
  cache?: never
  /**
   * @deprecated
   * TODO: Rolldown might supports this in a long term. Need to investigate.
   */
  experimentalCacheExpiry?: never
  /**
   * @deprecated
   * deprecated by Rollup
   */
  inlineDynamicImports?: never
  /**
   * @deprecated
   * deprecated by Rollup
   */
  manualChunks?: never
  /**
   * @deprecated
   * TODO: Need to investigate.
   */
  maxParallelFileOps?: never
  /**
   * @deprecated
   * deprecated by Rollup
   */
  maxParallelFileReads?: never
  /**
   * @deprecated
   * TODO: Need to investigate.
   */
  onwarn?: never
  /**
   * @deprecated
   * TODO: Need to investigate.
   */
  perf?: never
  /**
   * @deprecated
   * TODO: Need to investigate.
   */
  preserveEntrySignatures?: never
  /**
   * @deprecated
   * deprecated by Rollup
   */
  preserveModules?: never
  /**
   * @deprecated
   * TODO: Need to investigate.
   */
  preserveSymlinks?: never
  /**
   * @deprecated
   * TODO: Need to investigate.
   */
  strictDeprecations?: never
  /**
   * @deprecated
   * TODO: Need to investigate.
   */
  watch?: never

  // --- ToBeSupported

  context?: never
  makeAbsoluteExternalsRelative?: never
  moduleContext?: never
  shimMissingExports?: never

  // --- Rewritten

  treeshake?: boolean

  // --- Extra

  cwd?: string
}

function normalizeInput(
  input: InputOptions['input'],
): BindingInputOptions['input'] {
  if (input == null) {
    return {}
  } else if (typeof input === 'string') {
    return {
      main: input,
    }
  } else if (Array.isArray(input)) {
    return Object.fromEntries(
      input.map((src) => {
        const name = path.parse(src).name
        return [name, src]
      }),
    )
  } else {
    return input
  }
}

export function normalizeExternal(
  option: InputOptions['external'],
): BindingInputOptions['external'] {
  if (option == null) {
    return { string: [] }
  }

  if (option instanceof Function) {
    return {
      function: (specifier, importer, isResolved) => {
        return Boolean(option(specifier, importer, isResolved))
      },
      string: [],
    }
  }

  const opts = arraify(option)
  const regexs: RegExp[] = []
  const strings: string[] = []
  for (const item of opts) {
    if (item instanceof RegExp) {
      regexs.push(item)
    } else {
      strings.push(item)
    }
  }

  return {
    function: (specifier, _importer, _isResolved) =>
      regexs.some((regex) => regex.test(specifier)),
    string: strings,
  }
}

async function normalizePlugins(
  option: InputOptions['plugins'],
): Promise<BindingInputOptions['plugins']> {
  const plugins = await normalizePluginOption(option)
  const adapters = plugins.map((plugin) => createBuildPluginAdapter(plugin))
  return adapters
}

export async function normalizeInputOptions(
  input_opts: InputOptions,
): Promise<BindingInputOptions> {
  const { input, treeshake, external, plugins, cwd, ...rest } = input_opts

  // Make sure all fields of RollupInputOptions are handled.
  // @ts-expect-error
  const _empty: never = undefined as unknown as NonNullable<
    (typeof rest)[keyof typeof rest]
  >
  return {
    input: normalizeInput(input),
    treeshake: treeshake,
    external: normalizeExternal(external),
    plugins: await normalizePlugins(plugins),
    cwd: cwd,
  }
}
