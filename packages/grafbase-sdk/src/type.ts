import { Field } from './field'
import { ListDefinition } from './typedefs/list'
import { Interface } from './interface'
import {
  CacheDefinition,
  TypeCacheParams,
  TypeLevelCache
} from './typedefs/cache'
import { ReferenceDefinition } from './typedefs/reference'
import { ScalarDefinition } from './typedefs/scalar'
import { EnumDefinition } from './typedefs/enum'
import { ResolverDefinition } from './typedefs/resolver'
import { validateIdentifier } from './validation'
import { Query } from './query'
import { MapDefinition } from './typedefs/map'
import { FederationKey, FederationKeyParameters } from './federation'
import { JoinDefinition } from './typedefs/join'

/**
 * A collection of fields in a model.
 */
export type TypeFields = Record<string, TypeFieldShape>

/**
 * A combination of classes a field in a non-model type can be.
 */
export type TypeFieldShape =
  | ScalarDefinition
  | ListDefinition
  | ReferenceDefinition
  | CacheDefinition
  | MapDefinition
  | EnumDefinition<any, any>
  | ResolverDefinition
  | JoinDefinition

/**
 * A composite type definition (e.g. not a model).
 */
export class Type {
  private _name: string
  private fields: Field[]
  private interfaces: Interface[]
  private cacheDirective?: TypeLevelCache
  private keys: FederationKey[]

  constructor(name: string) {
    validateIdentifier(name)

    this._name = name
    this.fields = []
    this.interfaces = []
    this.keys = []
  }

  /**
   * The name of the type.
   */
  public get name(): string {
    return this._name
  }

  /**
   * Pushes a field to the type definition.
   *
   * @param name - The name of the field.
   * @param definition - The type definition with optional attributes.
   */
  public field(name: string, definition: TypeFieldShape): this {
    this.fields.push(new Field(name, definition))

    return this
  }

  /**
   * Pushes an interface implemented by the type.
   *
   * @param iface - The interface this type implements.
   */
  public implements(iface: Interface): this {
    this.interfaces.push(iface)

    return this
  }

  /**
   * Sets the type `@cache` directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: TypeCacheParams): this {
    this.cacheDirective = new TypeLevelCache(params)

    return this
  }

  /**
   * Marks this type as a federation entitiy with the given key
   *
   * @param fields The fields that make up this key, in FieldSet format
   * @param params The parameters for this key
   */
  public key(fields: string, params?: FederationKeyParameters): this {
    this.keys.push(new FederationKey(fields, params))
    return this
  }

  public toString(): string {
    const interfaces = this.interfaces.map((i) => i.name).join(' & ')
    const cache = this.cacheDirective ? ` ${this.cacheDirective}` : ''
    const keys =
      this.keys.length != 0
        ? ` ${this.keys.map((key) => key.toString()).join(' ')}`
        : ''
    const impl = interfaces ? ` implements ${interfaces}` : ''
    const header = `type ${this.name}${cache}${keys}${impl} {`

    const fields = distinct(
      (this.interfaces.flatMap((i) => i.fields) ?? []).concat(this.fields)
    )
      .map((field) => `  ${field}`)
      .join('\n')

    const footer = '}'

    return `${header}\n${fields}\n${footer}`
  }
}

export class TypeExtension {
  private name: string
  private queries: Query[]

  constructor(type: string | Type) {
    if (type instanceof Type) {
      this.name = type.name
    } else {
      validateIdentifier(type)
      this.name = type
    }

    this.queries = []
  }

  /**
   * Pushes a query to the extension.
   *
   * @param query - The query to be added.
   */
  public query(query: Query): this {
    this.queries.push(query)

    return this
  }

  public toString(): string {
    if (this.queries.length > 0) {
      const queries = this.queries.map(String).join('\n')

      return `extend type ${this.name} {\n${queries}\n}`
    } else {
      return ''
    }
  }
}

function distinct(fields: Field[]): Field[] {
  const found = new Set()

  return fields.filter((f) => {
    if (found.has(f.name)) {
      return false
    } else {
      found.add(f.name)
      return true
    }
  })
}
