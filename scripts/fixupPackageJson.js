#!/usr/bin/env node

const { readFileSync, writeFileSync } = require("fs");
const { join } = require("path");

const main = () => {
  const file_path = join(__dirname, "..", "js", "package.json");
  package_data = JSON.parse(readFileSync(file_path));
  if (!package_data.files.includes("index_bg.js")) {
    package_data.files = [...package_data.files, "index_bg.js"];
  }
  if (!Object.keys(package_data).includes("type")) {
    package_data["type"] = "module";
  }

  writeFileSync(file_path, JSON.stringify(package_data, null, 2));
};

main();
