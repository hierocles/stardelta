import incident from "incident";
import { SintSize, UintSize } from "semantic-types";

/**
 * Returns the minimum number of bits required to represent `value` using Stream#writeUintBits
 */
export function getUintBitCount(value: UintSize): UintSize {
  // TODO: Check why we are restricting to 2^31 instead of 2^32
  if (value < 0 || value >= 2 ** 31) {
    throw new incident.Incident("UnsupportedValue", {value});
  }
  return 32 - Math.clz32(value);
}

/**
 * Returns the minimum number of bits required to represent `value` using Stream#writeSintBits
 */
export function getSintBitCount(value: SintSize): UintSize {
  if (value < -(2 ** 31) || value >= 2 ** 31) {
    throw new incident.Incident("UnsupportedValue", {value});
  }
  if (value === 0) {
    return 0;
  }
  return value < 0 ? 33 - Math.clz32(~value) : 33 - Math.clz32(value);
}

/**
 * Returns the minimum number of bits required to represent all the `values` using Stream#writeUintBits
 */
export function getUintMinBitCount(...values: UintSize[]): UintSize {
  return values.map(getUintBitCount).reduce((a, b) => Math.max(a, b), 0);
}

/**
 * Returns the minimum number of bits required to represent all the `values` using Stream#writeSintBits
 */
export function getSintMinBitCount(...values: SintSize[]): UintSize {
  return values.map(getSintBitCount).reduce((a, b) => Math.max(a, b), 0);
}
