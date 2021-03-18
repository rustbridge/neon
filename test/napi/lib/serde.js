"use strict";

const assert = require('assert');

const addon = require('..');
const pokedex = require('../fixtures/pokedex.json');

describe('serde', () => {
    it('should be able to parse a pokedex from a string', () => {
        const s = JSON.stringify(pokedex);
        const result = addon.parse_pokedex(s);

        assert.deepStrictEqual(result, pokedex);
    });

    it('should be able to stringify a pokedex', () => {
        const result = addon.stringify_pokedex(pokedex);

        assert.deepStrictEqual(JSON.parse(result), pokedex);
    });
});
