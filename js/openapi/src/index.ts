import loadCrate from "../../../crates/jsona-wasm-openapi/Cargo.toml";
import { ParseResult } from "./types";
export * as types from "./types";

export default class JsonaOpenapi {
  private static crate: any | undefined;
  private static guard: boolean = false;
  private constructor() {
    if (!JsonaOpenapi.guard) {
      throw new Error(
        `an instance of JsonaOpenapi can only be created by calling the "getInstance" static method`
      );
    }
  }

  public static async getInstance(): Promise<JsonaOpenapi> {
    if (typeof JsonaOpenapi.crate === "undefined") {
      JsonaOpenapi.crate = await loadCrate();
    }
    JsonaOpenapi.guard = true;
    const self = new JsonaOpenapi();
    JsonaOpenapi.guard = false;
    return self;
  }

  /**
   * Parse jsona doc as ast
   * @param jsona JSONA document.
   */
  public parse(jsona: string): ParseResult {
    try {
      return { value: JsonaOpenapi.crate.parse(jsona) }
    } catch (errors) {
      return { errors: errors }
    }
  }
}
