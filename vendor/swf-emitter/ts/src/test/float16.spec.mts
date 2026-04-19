import { WritableStream } from "@open-flash/stream";
import chai from "chai";
import { Float16, Uint8 } from "semantic-types";

describe("stream", function () {
  describe("writeFloat16BE", function () {
    interface TestItem {
      input: Float16;
      expected: Uint8[];
    }

    const items: TestItem[] = [
      {input: -0, expected: [0b10000000, 0b00000000]},
      {input: 0, expected: [0b00000000, 0b00000000]},
      {input: 1 * 2 ** (-24), expected: [0b00000000, 0b00000001]},
      {input: 2 * 2 ** (-24), expected: [0b00000000, 0b00000010]},
      {input: 3 * 2 ** (-24), expected: [0b00000000, 0b00000011]},
      {input: 4 * 2 ** (-24), expected: [0b00000000, 0b00000100]},
      {input: 1023 * 2 ** (-24), expected: [0b00000011, 0b11111111]},
      {input: 2 ** (-14), expected: [0b00000100, 0b00000000]},
      {input: 2 ** (-13), expected: [0b00001000, 0b00000000]},
      {input: 2 ** (-12), expected: [0b00001100, 0b00000000]},
      {input: 2 ** (-12) + 1 * 2 ** (-22), expected: [0b00001100, 0b00000001]},
      {input: 2 ** (-12) + 2 * 2 ** (-22), expected: [0b00001100, 0b00000010]},
      {input: 2 ** (-12) + 3 * 2 ** (-22), expected: [0b00001100, 0b00000011]},
      {input: 0.5, expected: [0b00111000, 0b00000000]},
      {input: 1, expected: [0b00111100, 0b00000000]},
      {input: 1 + 2 ** (-10), expected: [0b00111100, 0b00000001]},
      {input: 1.0009765625, expected: [0b00111100, 0b00000001]},
      {input: 1.59375, expected: [0b00111110, 0b01100000]},
      {input: 2, expected: [0b01000000, 0b00000000]},
      {input: 2 ** 15, expected: [0b01111000, 0b00000000]},
      {input: 2 ** 15 + 1023 * 2 ** 5, expected: [0b01111011, 0b11111111]},
      {input: 2 ** 15 + 1024 * 2 ** 5, expected: [0b01111100, 0b00000000]},
      {input: 2 ** 16, expected: [0b01111100, 0b00000000]},
      {input: +Infinity, expected: [0b01111100, 0b00000000]},
      {input: NaN, expected: [0b11111111, 0b11111111]},
    ];

    for (const {input, expected} of items) {
      it(`should write ${input}`, function () {
        const s: WritableStream = new WritableStream();
        s.writeFloat16BE(input);
        const actual: Uint8Array = s.getBytes();
        const actualBits: string = [...actual].map(toBinary8).join("");
        const expectedBits: string = expected.map(toBinary8).join("");
        chai.assert.strictEqual(actualBits, expectedBits);
      });
    }
  });
});

function toBinary8(x: Uint8): string {
  return x.toString(2).padStart(8, "0");
}
