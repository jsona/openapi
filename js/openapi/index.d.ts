import { OpenAPIV3 } from "openapi-types";

/**
 * Parse jsona doc as ast
 * @param input JSONA document.
 */
export function parse(input: string): OpenapiTypes.ParseResult;

export namespace OpenapiTypes {
  export import Spec = OpenAPIV3;
  export interface Range {
    start: Position;
    end: Position;
  }

  export interface Position {
    index: number;
    line: number;
    character: number;
  }

  export interface ErrorObject {
    kind: string,
    message: string,
    range?: Range,
  }

  export interface ParseResult {
    value?: Spec.Document,
    errors?: ErrorObject[],
  }
}
