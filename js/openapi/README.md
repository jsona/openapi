# @jsona/openapi

This is a JavaScript wrapper for the JSONA openapi.

## Install

```
npm i @jsona/openapi
yarn add @jsona/openapi
```

## Usage

```js
import JsonaOpenapi from '@jsona/openapi';

const jsonaOpenapi = await JsonaOpenapi.getInstance();

// parse as openapi
jsonaOpenapi.parse(jsonaContent);
```