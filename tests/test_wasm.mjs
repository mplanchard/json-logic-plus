/**
 * Test the WASM package using node
 */

import { readFileSync } from "fs";
import { dirname, join } from "path";
import { apply } from "../js/index.js";

const load_test_json = () => {
  const file_path = import.meta.url;

  let data_file;
  // Hack for windows CI
  if (file_path.search(/[A-Z]:/) != -1) {
    data_file = join(
      // import.meta.url will be something like file:///D:/whatever for some reason
      dirname(import.meta.url).split("file:///")[1],
      "data",
      "tests.json"
    );
  } else {
    data_file = join(
      // import.meta.url will be something like file://<absolute_path>
      dirname(import.meta.url).split("file://")[1],
      "data",
      "tests.json"
    );
  }
  const data = readFileSync(data_file);
  return JSON.parse(data);
};

const print_case = (c, res) => {
  console.log(`  Logic: ${JSON.stringify(c[0])}`);
  console.log(`  Data: ${JSON.stringify(c[1])}`);
  console.log(`  Expected: ${JSON.stringify(c[2])}`);
  console.log(`  Actual: ${res && JSON.stringify(res)}`);
};

const run_tests = (cases) => {
  cases
    .filter((i) => typeof i !== "string")
    .forEach((c) => {
      const logic = c[0];
      const data = c[1];
      const exp = c[2];

      let res;
      try {
        res = apply(logic, data);
      } catch (e) {
        console.log("Test errored!");
        console.log(`  Error: ${e}}`);
        print_case(c);
        process.exit(2);
      }

      if (JSON.stringify(res) !== JSON.stringify(exp)) {
        console.log("Failed Test!");
        print_case(c, res);
        process.exit(1);
      }
    });
};

const main = () => {
  run_tests(load_test_json());
};

main();
