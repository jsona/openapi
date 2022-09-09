const { parse } = require("./pkg");
const assert = require("assert");

assert.deepEqual(parse("{}"), {
  value: {
    openapi: '3.0.0',
    info: { version: '0.1.0', title: 'openapi' },
    paths: {},
    components: {}
  },
  errors: null
})
