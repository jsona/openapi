import loadCrate from "../../../crates/jsona-wasm-openapi/Cargo.toml";
import { ParseResult } from "./types";
export * as types from "./types";

export class JsonaOpenapi {
  private static crate: any | undefined;
  private static initializing: boolean = false;
  private constructor() {
    if (!JsonaOpenapi.initializing) {
      throw new Error(
        `an instance of JsonaOpenapi can only be created by calling the "initialize" static method`
      );
    }
  }

  public static async init(): Promise<JsonaOpenapi> {
    if (typeof JsonaOpenapi.crate === "undefined") {
      JsonaOpenapi.crate = await loadCrate();
    }
    JsonaOpenapi.initializing = true;
    const self = new JsonaOpenapi();
    JsonaOpenapi.initializing = false;
    return self;
  }

  /**
   * Parse jsona doc as ast
   * @param jsona JSONA document.
   */
  public parse(jsona: string): ParseResult {
    try {
      return { openapi: JsonaOpenapi.crate.parse(jsona) }
    } catch (errors) {
      return { errors: errors }
    }
  }
}
