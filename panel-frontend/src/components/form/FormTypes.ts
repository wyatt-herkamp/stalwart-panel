export interface DropDownOption {
  name: string
  value: string
}

export function enumToOptions(e: any): DropDownOption[] {
  return Object.keys(e).map((k) => {
    return { name: k, value: e[k] }
  })
}
