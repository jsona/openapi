import { OpenAPIV3 as Openapi } from "openapi-types";

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
  value?: Openapi.Document,
  errors?: ErrorObject[],
}

export { Openapi }