#! /usr/bin/env bash

cargo build --profile=wasm-release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "gameoflife" \
    ./target/wasm32-unknown-unknown/wasm-release/gameoflife.wasm

cat >./out/index.html <<EOL
<!doctype html>
<html lang="en">

<head>
  <style>
    html,
    body,
    canvas {
      height: 100% !important;
      width: 100% !important;
    }
  </style>
</head>

<body style="margin: 0px;">
  <script type="module">
    import init from './gameoflife.js'

    init().catch((error) => {
      if (!error.message.startsWith("Using exceptions for control flow, don't mind me. This isn't actually an error!")) {
        throw error;
      }
    });
  </script>
</body>

</html>
EOL
