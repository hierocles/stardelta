import { WritableStream, WritableByteStream } from "@open-flash/stream";
import chai from "chai";
import fs from "fs";
import { IoType } from "kryo";
import { JSON_READER } from "kryo-json/json-reader";
import sysPath from "path";
import { $Tag,Tag } from "swf-types/tag";

import { emitTag } from "../lib/emitters/tags.mjs";
import meta from "./meta.mjs";
import { prettyPrintBytes, readFile, readTextFile } from "./utils.mjs";

const PROJECT_ROOT: string = sysPath.join(meta.dirname, "..");
const TAG_SAMPLES_ROOT: string = sysPath.join(PROJECT_ROOT, "..", "tests", "tags");
// `BLACKLIST` can be used to forcefully skip some tests.
const BLACKLIST: ReadonlySet<string> = new Set([
  // "define-shape/shape1-squares",
]);
// `WHITELIST` can be used to only enable a few tests.
const WHITELIST: ReadonlySet<string> = new Set([
  // "place-object2/place-id-1",
  // "place-object3/update-depth-1",
]);

describe("tags", function () {
  for (const group of getSampleGroups()) {
    describe(group.name, function () {
      for (const sample of getSamplesFromGroup(group.name)) {
        it(sample.name, async function () {
          const valueJson: string = await readTextFile(sample.valuePath);
          const value: Tag = group.type.read(JSON_READER, valueJson);
          const s: WritableByteStream = new WritableStream();

          let swfVersion: number;
          switch (`${group.name}/${sample.name}`) {
            case "place-object/po2-swf5":
              swfVersion = 5;
              break;
            default:
              swfVersion = 10;
              break;
          }

          emitTag(s, value, swfVersion);
          const actualBytes: Uint8Array = s.getBytes();

          const expectedBytes: Uint8Array = await readFile(sample.outputPath);

          try {
            chai.assert.deepEqual(actualBytes, expectedBytes);
          } catch (err) {
            chai.assert.strictEqual(prettyPrintBytes(actualBytes), prettyPrintBytes(expectedBytes));
            throw err;
          }
        });
      }
    });
  }
});

interface SampleGroup {
  name: string;
  type: IoType<Tag>;
}

function* getSampleGroups(): IterableIterator<SampleGroup> {
  for (const dirEnt of fs.readdirSync(TAG_SAMPLES_ROOT, {withFileTypes: true})) {
    if (!dirEnt.isDirectory()) {
      continue;
    }
    const name: string = dirEnt.name;
    yield {
      name,
      type: $Tag,
    };
  }
}

interface Sample {
  name: string;
  outputPath: string;
  valuePath: string;
}

function* getSamplesFromGroup(group: string): IterableIterator<Sample> {
  const groupPath: string = sysPath.join(TAG_SAMPLES_ROOT, group);
  for (const dirEnt of fs.readdirSync(groupPath, {withFileTypes: true})) {
    if (!dirEnt.isDirectory()) {
      continue;
    }
    const testName: string = dirEnt.name;
    const testPath: string = sysPath.join(groupPath, testName);

    if (BLACKLIST.has(`${group}/${testName}`)) {
      continue;
    } else if (WHITELIST.size > 0 && !WHITELIST.has(`${group}/${testName}`)) {
      continue;
    }

    const outputPath: string = fs.existsSync(sysPath.join(testPath, "output.bytes"))
      ? sysPath.join(testPath, "output.bytes")
      : sysPath.join(testPath, "input.bytes");
    const valuePath: string = sysPath.join(testPath, "value.json");

    yield {name: testName, outputPath, valuePath};
  }
}
