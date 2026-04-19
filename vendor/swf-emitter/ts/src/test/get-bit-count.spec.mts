import chai from "chai";
import { SintSize, UintSize } from "semantic-types";

import { getSintBitCount, getSintMinBitCount, getUintBitCount, getUintMinBitCount } from "../lib/get-bit-count.mjs";

// function toUint8Array(array: number[]): Uint8Array {
//   const result: Uint8Array = new Uint8Array(array.length);
//   result.set(array);
//   return result;
// }

describe("getBitCount", function () {
  describe("getUintBitCount", function () {
    interface TestItem {
      input: UintSize;
      expected: UintSize;
    }

    const items: TestItem[] = [
      {input: 0, expected: 0},
      {input: 1, expected: 1},
      {input: 2, expected: 2},
      {input: 3, expected: 2},
      {input: 4, expected: 3},
      {input: 5, expected: 3},
      {input: 7, expected: 3},
      {input: 8, expected: 4},
      {input: 15, expected: 4},
      {input: 16, expected: 5},
      {input: 2147483647, expected: 31},
    ];

    for (const {input, expected} of items) {
      it(`should return ${expected} for ${input}`, function () {
        chai.assert.strictEqual(getUintBitCount(input), expected);
      });
    }
  });

  describe("getSintBitCount", function () {
    interface TestItem {
      input: SintSize;
      expected: UintSize;
    }

    const items: TestItem[] = [
      {input: 0, expected: 0},
      {input: 1, expected: 2},
      {input: 2, expected: 3},
      {input: 3, expected: 3},
      {input: 4, expected: 4},
      {input: 5, expected: 4},
      {input: 7, expected: 4},
      {input: 8, expected: 5},
      {input: 15, expected: 5},
      {input: 16, expected: 6},
      {input: 2147483647, expected: 32},
      {input: -1, expected: 1},
      {input: -2, expected: 2},
      {input: -3, expected: 3},
      {input: -4, expected: 3},
      {input: -5, expected: 4},
      {input: -2147483648, expected: 32},
    ];

    for (const {input, expected} of items) {
      it(`should return ${expected} for ${input}`, function () {
        chai.assert.strictEqual(getSintBitCount(input), expected);
      });
    }
  });

  describe("getUintMinBitCount", function () {
    interface TestItem {
      input: UintSize[];
      expected: UintSize;
    }

    const items: TestItem[] = [
      {input: [], expected: 0},
      {input: [0], expected: 0},
      {input: [0, 0], expected: 0},
      {input: [1], expected: 1},
      {input: [1, 1], expected: 1},
      {input: [0, 1], expected: 1},
      {input: [1, 0], expected: 1},
      {input: [0, 0, 2, 3], expected: 2},
      {input: [4, 0, 2, 3], expected: 3},
      {input: [2, 1, 2], expected: 2},
      {input: [2147483647, 3, 0, 1000], expected: 31},
    ];

    for (const {input, expected} of items) {
      it(`should return ${expected} for ${input}`, function () {
        chai.assert.strictEqual(getUintMinBitCount(...input), expected);
      });
    }
  });

  describe("getSintBitCount", function () {
    interface TestItem {
      input: SintSize[];
      expected: UintSize;
    }

    const items: TestItem[] = [
      {input: [], expected: 0},
      {input: [0], expected: 0},
      {input: [-1], expected: 1},
      {input: [0, -1], expected: 1},
      {input: [-1, 0], expected: 1},
      {input: [16, 0, -5], expected: 6},
      {input: [2147483647, -2147483648], expected: 32},
    ];

    for (const {input, expected} of items) {
      it(`should return ${expected} for ${input}`, function () {
        chai.assert.strictEqual(getSintMinBitCount(...input), expected);
      });
    }
  });
});
